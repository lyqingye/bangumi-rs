use anyhow::{Context, Result};
use downloader::Downloader;
use metadata;
use model::sea_orm_active_enums::{DownloadStatus, State};
use model::{
    bangumi, episode_download_tasks, episodes, file_name_parse_record, subscriptions, torrents,
};
use parser;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, error, info, warn};

use crate::db::Db;
use crate::selector::TorrentSelector;
use crate::tasks::TaskManager;

/// Worker 命令枚举
#[derive(Debug, Clone)]
pub enum WorkerCommand {
    /// 停止 worker
    Shutdown(mpsc::Sender<()>),
    /// 触发收集种子
    TriggerCollection,
}

/// 负责单个番剧的下载任务处理
#[derive(Clone)]
pub struct BangumiWorker {
    pub(crate) db: Db,
    pub(crate) parser: parser::worker::Worker,
    pub(crate) metadata: metadata::worker::Worker,
    pub(crate) downloader: Arc<Box<dyn Downloader>>,
    pub(crate) task_manager: TaskManager,
    // 命令发送器
    cmd_tx: broadcast::Sender<WorkerCommand>,
    notify: notify::worker::Worker,
    selector: TorrentSelector,
    pub(crate) bangumi: bangumi::Model,
    pub(crate) sub: subscriptions::Model,
}

impl BangumiWorker {
    pub fn new(
        sub: subscriptions::Model,
        bangumi: bangumi::Model,
        db: Db,
        parser: parser::worker::Worker,
        metadata: metadata::worker::Worker,
        downloader: Arc<Box<dyn Downloader>>,
        task_manager: TaskManager,
        notify: notify::worker::Worker,
    ) -> Self {
        // 创建命令通道
        let (cmd_tx, _) = broadcast::channel(16);
        let selector = TorrentSelector::new(&sub);
        Self {
            sub,
            bangumi,
            db,
            parser,
            metadata,
            downloader,
            task_manager,
            cmd_tx,
            notify,
            selector,
        }
    }

    /// 为指定集数选择最合适的种子
    async fn select_episode_torrent(
        &self,
        episode_number: i32,
        ep_start_number: i32,
        torrent_pairs: &[(torrents::Model, file_name_parse_record::Model)],
        episodes: &HashMap<i32, episodes::Model>,
    ) -> Result<Option<torrents::Model>> {
        // 过滤出当前集数的种子
        let episode_torrents: Vec<_> = torrent_pairs
            .iter()
            .filter(|(torrent, parse_result)| {
                if let Some(ep) = parse_result.episode_number {
                    // 剧集修复:
                    // 例如: 某些番剧第二季可能从第13集开始,但种子标记为第1集
                    // ep_start_number = 13, ep = 1 时:
                    // actual_ep = 1 + 13 - 1 = 13,修正为实际的第13集
                    let mut actual_ep = ep;
                    if ep_start_number > 1 && ep < ep_start_number {
                        actual_ep = ep + ep_start_number - 1;
                    }

                    // 确保种子发布时间在番剧集数发布时间之后
                    if let Some(episode) = episodes.get(&actual_ep) {
                        if let Some(air_date) = episode.air_date {
                            if torrent.pub_date < air_date.into() {
                                return false;
                            }
                        }
                    }

                    actual_ep == episode_number
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        // 使用 TorrentSelector 选择最合适的种子
        self.selector.select(episode_torrents)
    }

    /// 收集并解析种子，然后为任务选择合适的种子
    async fn collect_and_process_torrents(&self) -> Result<()> {
        // 1. 收集种子
        info!("开始收集番剧 {} 的种子", self.bangumi.name);
        self.metadata
            .request_refresh(
                Some(self.bangumi.id),
                metadata::worker::RefreshKind::Torrents,
            )
            .await?;

        // 2. 获取并解析种子
        let torrents = self.db.get_bangumi_torrents(self.bangumi.id).await?;
        if !torrents.is_empty() {
            info!("开始解析番剧 {} 的种子文件名", self.bangumi.name);
            let file_names: Vec<String> = torrents.iter().map(|t| t.title.clone()).collect();
            self.parser.parse_file_names(file_names).await?;
        }

        // 3. 获取所有种子及其解析结果
        let torrent_pairs = self
            .db
            .get_bangumi_torrents_with_parse_results(self.bangumi.id)
            .await?;

        // 5. 获取所有 Missing 状态的任务
        let tasks = self
            .task_manager
            .get_unfinished_tasks(self.bangumi.id)
            .await?;
        let missing_tasks: Vec<_> = tasks
            .into_iter()
            .filter(|t| t.state == State::Missing)
            .collect();

        let episodes: HashMap<i32, episodes::Model> = self
            .db
            .get_bangumi_episodes(self.bangumi.id)
            .await?
            .into_iter()
            .map(|e| (e.number, e))
            .collect();

        // 这里需要筛选未被使用过的种子, TODO 考虑上面的Sql做处理？
        let info_hashes = torrent_pairs
            .iter()
            .map(|t| t.0.info_hash.clone())
            .collect::<Vec<String>>();
        let already_used_info_hashes = self
            .db
            .list_torrent_download_tasks_by_info_hashes(&info_hashes)
            .await?;

        let mut unused_torrents = Vec::new();
        for (torrent, ps) in torrent_pairs {
            if already_used_info_hashes.contains(&torrent.info_hash) {
                debug!("种子 {} 已被使用过，跳过", torrent.info_hash);
                continue;
            }
            unused_torrents.push((torrent, ps));
        }

        info!("开始为番剧 {} 选择合适的种子", self.bangumi.name);

        // 6. 为每个 Missing 任务选择合适的种子
        for task in missing_tasks {
            if let Some(torrent) = self
                .select_episode_torrent(
                    task.episode_number,
                    self.bangumi.ep_start_number,
                    &unused_torrents,
                    &episodes,
                )
                .await?
            {
                info!(
                    "已为番剧 {} 第 {} 集选择合适的种子",
                    self.bangumi.name, task.episode_number
                );
                self.task_manager
                    .update_task_ready(task.bangumi_id, task.episode_number, &torrent.info_hash)
                    .await?;
            }
        }

        info!("番剧 {} 种子收集处理完成", self.bangumi.name);

        Ok(())
    }

    /// 统一的 worker 运行循环，处理所有定时任务
    async fn run_worker(worker: BangumiWorker, mut cmd_rx: broadcast::Receiver<WorkerCommand>) {
        info!("启动番剧 {} 的后台处理", worker.bangumi.name);

        // 初始化任务缓存
        if let Err(e) = worker
            .task_manager
            .init_bangumi_tasks(worker.sub.bangumi_id)
            .await
        {
            error!("初始化番剧 {} 的任务缓存失败: {}", worker.bangumi.name, e);
            return;
        }

        // 创建三个定时器
        let mut collector_interval = tokio::time::interval(std::time::Duration::from_secs(
            worker.sub.collector_interval.unwrap_or(60 * 30) as u64,
        ));

        let mut metadata_interval = tokio::time::interval(std::time::Duration::from_secs(
            worker.sub.metadata_interval.unwrap_or(60 * 60 * 24) as u64,
        ));

        let mut task_processor_interval = tokio::time::interval(std::time::Duration::from_secs(
            worker.sub.task_processor_interval.unwrap_or(5) as u64,
        ));

        loop {
            tokio::select! {
                _ = collector_interval.tick() => {
                    if let Err(e) = worker.collect_and_process_torrents().await {
                        error!("番剧 {} 处理种子失败: {}", worker.bangumi.name, e);
                    }
                }
                _ = metadata_interval.tick() => {
                    if let Err(e) = worker
                        .metadata
                        .request_refresh(Some(worker.bangumi.id), metadata::worker::RefreshKind::Metadata)
                        .await
                    {
                        error!("番剧 {} 刷新元数据失败: {}", worker.bangumi.name, e);
                    }
                }
                _ = task_processor_interval.tick() => {
                    if let Err(e) = worker.process_tasks().await {
                        error!("番剧 {} 处理下载任务失败: {}", worker.bangumi.name, e);
                    }
                }
                Ok(cmd) = cmd_rx.recv() => {
                    match cmd {
                        WorkerCommand::Shutdown(tx) => {
                            info!("停止番剧 {} 的后台处理", worker.bangumi.name);
                            // 清除任务缓存
                            worker.task_manager.clear_bangumi_tasks(worker.sub.bangumi_id).await;
                            let _ = tx.send(()).await;
                            break;
                        }
                        WorkerCommand::TriggerCollection => {
                            info!("手动触发番剧 {} 的种子收集", worker.bangumi.name);
                            if let Err(e) = worker.collect_and_process_torrents().await {
                                error!("番剧 {} 处理种子失败: {}", worker.bangumi.name, e);
                            }
                        }
                    }
                }
            }
        }
    }

    /// 启动 worker，开始定时处理任务
    pub fn spawn(self) {
        // 创建命令接收器
        let cmd_rx = self.cmd_tx.subscribe();

        // 启动统一的后台处理循环
        tokio::spawn(async move {
            Self::run_worker(self, cmd_rx).await;
        });
    }

    /// 停止 worker
    pub async fn shutdown(&self) -> Result<()> {
        // 创建一个 mpsc 通道来等待 worker 完全停止
        let (tx, mut rx) = mpsc::channel(1);
        // 发送停止命令
        let _ = self.cmd_tx.send(WorkerCommand::Shutdown(tx));
        // 等待 worker 确认停止
        rx.recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("worker 停止失败"))?;
        Ok(())
    }

    /// 触发种子收集
    pub fn trigger_collection(&self) {
        // 发送触发收集命令
        let _ = self.cmd_tx.send(WorkerCommand::TriggerCollection);
    }

    /// 处理所有任务的状态转换
    async fn process_tasks(&self) -> Result<()> {
        debug!("开始处理番剧 {} 的所有任务", self.bangumi.name);

        // 从缓存获取该番剧所有未完成的任务
        let tasks = self
            .task_manager
            .get_unfinished_tasks(self.sub.bangumi_id)
            .await?;

        // 处理所有任务
        for task in tasks {
            if let Err(e) = self.process_task(task).await {
                error!("处理下载任务失败: {}", e);
            }
        }

        debug!("番剧 {} 所有任务处理完成", self.bangumi.name);
        Ok(())
    }

    /// 处理单个任务的状态转换
    async fn process_task(&self, task: episode_download_tasks::Model) -> Result<()> {
        match task.state {
            State::Ready => {
                if let Some(info_hash) = task.ref_torrent_info_hash {
                    info!(
                        "开始下载番剧 {} 第 {} 集",
                        self.bangumi.name, task.episode_number
                    );
                    // 获取种子信息
                    let torrent = self
                        .db
                        .get_torrent_by_info_hash(&info_hash)
                        .await?
                        .context("种子不存在")?;
                    let bangumi = self
                        .db
                        .get_bangumi_by_id(task.bangumi_id)
                        .await?
                        .context("番剧不存在")?;
                    // 创建下载任务
                    self.downloader
                        .add_task(&torrent.info_hash, PathBuf::from(bangumi.name))
                        .await?;
                    // 更新状态为下载中
                    self.task_manager
                        .update_task_state(task.bangumi_id, task.episode_number, State::Downloading)
                        .await?;
                }
            }
            State::Downloading => {
                if let Some(info_hash) = task.ref_torrent_info_hash {
                    // 检查下载状态
                    let tasks = self.downloader.list_tasks(&[info_hash.clone()]).await?;
                    if let Some(download_task) = tasks.first() {
                        match download_task.download_status {
                            DownloadStatus::Completed => {
                                info!(
                                    "番剧 {} 第 {} 集下载完成",
                                    self.bangumi.name, task.episode_number
                                );
                                self.task_manager
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
                                            self.bangumi.name, task.episode_number
                                        ),
                                    )
                                    .await?;
                            }
                            DownloadStatus::Failed => {
                                error!(
                                    "番剧 {} 第 {} 集下载失败",
                                    self.bangumi.name, task.episode_number
                                );
                                self.task_manager
                                    .update_task_state(
                                        task.bangumi_id,
                                        task.episode_number,
                                        State::Failed,
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
                    self.bangumi.name, task.episode_number
                );
                self.task_manager
                    .update_task_state(task.bangumi_id, task.episode_number, State::Missing)
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}
