use anyhow::{Context, Result};
use downloader::{Downloader, Event, Resource};
use model::sea_orm_active_enums::State;
use model::{episode_download_tasks, sea_orm_active_enums::DownloadStatus};
use sea_orm::Set;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info, warn};

use crate::db::Db;

/// 任务缓存管理器
#[derive(Clone)]
pub struct TaskManager {
    db: Db,
    downloader: Arc<Box<dyn Downloader>>,
    notify: notify::worker::Worker,
    cmd_tx: Option<mpsc::UnboundedSender<Cmd>>,
}

enum Cmd {
    StateUpdate((i32, i32)),
    Stop(oneshot::Sender<()>),
}

impl TaskManager {
    pub fn new(
        db: Db,
        downloader: Arc<Box<dyn Downloader>>,
        notify: notify::worker::Worker,
    ) -> Self {
        Self {
            db,
            downloader,
            notify,
            cmd_tx: None,
        }
    }

    pub fn spawn(&mut self) -> Result<()> {
        let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel();
        self.cmd_tx = Some(cmd_tx);

        let task_manager = self.clone();

        tokio::spawn(async move {
            let downloader = task_manager.downloader.clone();
            let mut task_processor_interval =
                tokio::time::interval(std::time::Duration::from_secs(60));

            let mut dl_event_rx = downloader.subscribe().await;

            loop {
                tokio::select! {
                    _ = task_processor_interval.tick() => {
                        if let Err(e) = task_manager.process_tasks().await {
                            error!("处理下载任务失败: {}", e);
                        }
                    }
                    Some(cmd) = cmd_rx.recv() => {
                        match cmd {
                            Cmd::Stop(tx) => {
                                let _ = tx.send(());
                                break;
                            }
                            _ => {
                                if let Err(e) = task_manager.on_cmd(cmd).await {
                                    error!("处理命令失败: {}", e);
                                }
                            }
                        }
                    }
                    Ok(event) = dl_event_rx.recv() => {
                        if let Err(e) = task_manager.on_download_event(event).await {
                            error!("处理下载事件失败: {}", e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .as_ref()
            .expect("tasks unspawn")
            .send(Cmd::Stop(tx))?;
        let _ = rx.await;
        Ok(())
    }

    /// 获取指定番剧的所有未完成任务
    pub async fn get_unfinished_tasks(
        &self,
        bangumi_id: i32,
    ) -> Result<Vec<episode_download_tasks::Model>> {
        self.db.get_unfinished_tasks_by_bangumi(bangumi_id).await
    }

    /// 更新任务状态为就绪，并设置选中的种子
    pub async fn update_task_ready(
        &self,
        bangumi_id: i32,
        episode_number: i32,
        info_hash: &str,
    ) -> Result<()> {
        // 先更新数据库
        self.db
            .update_task_ready(bangumi_id, episode_number, info_hash)
            .await?;

        // 发送状态变化命令
        self.send_state_update_cmd(bangumi_id, episode_number)?;
        Ok(())
    }

    /// 更新任务状态为就绪，并设置选中的种子
    pub async fn retry_task(&self, bangumi_id: i32, episode_number: i32) -> Result<()> {
        // 先更新数据库
        self.db
            .update_task_state(bangumi_id, episode_number, State::Retrying)
            .await?;

        // 发送状态变化命令
        self.send_state_update_cmd(bangumi_id, episode_number)?;
        Ok(())
    }

    fn send_state_update_cmd(&self, bangumi_id: i32, episode_number: i32) -> Result<()> {
        self.cmd_tx
            .as_ref()
            .expect("tasks unspawn")
            .send(Cmd::StateUpdate((bangumi_id, episode_number)))?;

        Ok(())
    }

    /// 批量创建下载任务
    pub async fn batch_create_tasks(
        &self,
        bangumi_id: i32,
        episode_numbers: Vec<i32>,
    ) -> Result<()> {
        use model::episode_download_tasks::ActiveModel;

        // 构造批量任务
        let tasks: Vec<ActiveModel> = episode_numbers
            .into_iter()
            .map(|episode_number| ActiveModel {
                bangumi_id: Set(bangumi_id),
                episode_number: Set(episode_number),
                state: Set(State::Missing),
                ..Default::default()
            })
            .collect();

        // 批量插入数据库
        self.db.batch_create_tasks(tasks).await?;
        Ok(())
    }
}

impl TaskManager {
    /// 处理命令
    async fn on_cmd(&self, cmd: Cmd) -> Result<()> {
        match cmd {
            Cmd::StateUpdate((bangumi_id, episode_number)) => {
                self.on_state_update(bangumi_id, episode_number).await
            }
            _ => unreachable!(),
        }
    }

    /// 处理状态变化命令
    async fn on_state_update(&self, bangumi_id: i32, episode_number: i32) -> Result<()> {
        let task = self
            .db
            .get_episode_task_by_bangumi_id_and_episode_number(bangumi_id, episode_number)
            .await?;
        if let Some(task) = task {
            self.process_task(task).await?;
        }
        Ok(())
    }

    /// 处理下载事件
    async fn on_download_event(&self, event: Event) -> Result<()> {
        match event {
            Event::TaskUpdated((info_hash, _, _)) => {
                info!("监听到下载状态变更事件: {}", info_hash);
                let task = self.db.get_episode_task_by_info_hash(&info_hash).await?;
                if let Some(task) = task {
                    self.process_task(task).await?;
                }
            }
        }
        Ok(())
    }

    /// 处理所有任务的状态转换
    async fn process_tasks(&self) -> Result<()> {
        debug!("开始处理剧集下载任务");

        let tasks = self.db.get_all_unfinished_tasks().await?;

        // 处理所有任务
        for task in tasks {
            if let Err(e) = self.process_task(task).await {
                error!("处理下载任务失败: {}", e);
            }
        }

        debug!("剧集下载任务处理完成");
        Ok(())
    }

    /// 处理单个任务的状态转换
    async fn process_task(&self, task: episode_download_tasks::Model) -> Result<()> {
        let bangumi = self
            .db
            .get_bangumi_by_id(task.bangumi_id)
            .await?
            .context("番剧不存在")?;
        match task.state {
            State::Ready => {
                if let Some(ref info_hash) = task.ref_torrent_info_hash {
                    info!(
                        "开始下载番剧 {} 第 {} 集",
                        bangumi.name, task.episode_number
                    );
                    // 获取种子信息
                    let torrent = self
                        .db
                        .get_torrent_by_info_hash(info_hash)
                        .await?
                        .context("种子不存在")?;

                    // 创建下载任务
                    self.downloader
                        .add_task(
                            Resource::from_info_hash(torrent.info_hash)?,
                            PathBuf::from(bangumi.name),
                        )
                        .await?;

                    // 更新状态为下载中
                    self.db
                        .update_task_state(task.bangumi_id, task.episode_number, State::Downloading)
                        .await?;
                }
            }
            State::Downloading => {
                if let Some(ref info_hash) = task.ref_torrent_info_hash {
                    // 检查下载状态
                    let tasks = self.downloader.list_tasks(&[info_hash.clone()]).await?;
                    if let Some(download_task) = tasks.first() {
                        match download_task.download_status {
                            DownloadStatus::Completed => {
                                info!(
                                    "番剧 {} 第 {} 集下载完成",
                                    bangumi.name, task.episode_number
                                );
                                self.db
                                    .update_task_state(
                                        task.bangumi_id,
                                        task.episode_number,
                                        State::Downloaded,
                                    )
                                    .await?;
                                self.notify
                                    .notify(
                                        notify::worker::Topic::Download,
                                        "下载完成",
                                        format!(
                                            "番剧 [{}] 第 [{}] 集下载完成",
                                            bangumi.name, task.episode_number
                                        ),
                                    )
                                    .await?;
                            }
                            DownloadStatus::Failed => {
                                warn!(
                                    "番剧 {} 第 {} 集下载失败, 尝试重新选择种子",
                                    bangumi.name, task.episode_number
                                );
                                self.db
                                    .update_task_state(
                                        task.bangumi_id,
                                        task.episode_number,
                                        State::Missing,
                                    )
                                    .await?;
                            }
                            _ => {}
                        }
                    }
                }
            }
            State::Retrying => {
                // 重置状态到 Missing，重新开始整个流程
                warn!(
                    "番剧 {} 第 {} 集重试下载",
                    bangumi.name, task.episode_number
                );

                // 之前已经有过任务,
                if let Some(ref info_hash) = task.ref_torrent_info_hash {
                    self.downloader.retry(info_hash).await?;
                    self.db
                        .update_task_state(task.bangumi_id, task.episode_number, State::Downloading)
                        .await?;
                } else {
                    // 之前没有任务, 直接尝试新的种子
                    self.db
                        .update_task_state(task.bangumi_id, task.episode_number, State::Missing)
                        .await?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
