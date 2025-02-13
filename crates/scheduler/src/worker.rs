use anyhow::{Context, Result};
use downloader::Downloader;
use metadata;
use model::sea_orm_active_enums::{DownloadStatus, State};
use model::{bangumi, episode_download_tasks, file_name_parse_record, subscriptions, torrents};
use parser;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info};

use crate::db::Db;
use crate::selector::TorrentSelector;
use crate::tasks::TaskManager;

/// 负责单个番剧的下载任务处理
#[derive(Clone)]
pub struct BangumiWorker {
    pub(crate) db: Db,
    pub(crate) parser: parser::worker::Worker,
    pub(crate) metadata: metadata::worker::Worker,
    pub(crate) downloader: Arc<Box<dyn Downloader>>,
    pub(crate) task_manager: TaskManager,
    // 停止信号发送器，缓冲区设为2以确保能发送给两个线程
    stop_tx: broadcast::Sender<()>,
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
        // 创建容量为2的通道，确保可以发送给两个线程
        let (stop_tx, _) = broadcast::channel(2);
        let selector = TorrentSelector::new(&sub);
        Self {
            sub,
            bangumi,
            db,
            parser,
            metadata,
            downloader,
            task_manager,
            stop_tx,
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
    ) -> Result<Option<torrents::Model>> {
        // 过滤出当前集数的种子
        let episode_torrents: Vec<_> = torrent_pairs
            .iter()
            .filter(|(_, parse_result)| {
                if let Some(ep) = parse_result.episode_number {
                    // 剧集修复:
                    // 例如: 某些番剧第二季可能从第13集开始,但种子标记为第1集
                    // ep_start_number = 13, ep = 1 时:
                    // actual_ep = 1 + 13 - 1 = 13,修正为实际的第13集
                    if ep_start_number > 1 && ep < ep_start_number {
                        let actual_ep = ep + ep_start_number - 1;
                        actual_ep == episode_number
                    } else {
                        ep == episode_number
                    }
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

        info!("开始为番剧 {} 选择合适的种子", self.bangumi.name);

        // 6. 为每个 Missing 任务选择合适的种子
        for task in missing_tasks {
            if let Some(torrent) = self
                .select_episode_torrent(
                    task.episode_number,
                    self.bangumi.ep_start_number,
                    &torrent_pairs,
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

    /// 启动收集器，用于收集种子和刷新元数据
    async fn run_collector(worker: BangumiWorker, mut stop_rx: broadcast::Receiver<()>) {
        info!("启动番剧 {} 的种子收集处理", worker.bangumi.name);

        // 默认30分钟收集一次种子
        let mut collector_interval = tokio::time::interval(std::time::Duration::from_secs(
            worker.sub.collector_interval.unwrap_or(60 * 30) as u64,
        ));

        // 默认24小时刷新一次元数据
        let mut metadata_interval = tokio::time::interval(std::time::Duration::from_secs(
            worker.sub.metadata_interval.unwrap_or(60 * 60 * 24) as u64,
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
                _ = stop_rx.recv() => {
                    info!("停止番剧 {} 的种子收集处理", worker.bangumi.name);
                    break;
                }
            }
        }
    }

    /// 启动任务处理循环
    async fn run_task_processor(worker: BangumiWorker, mut stop_rx: broadcast::Receiver<()>) {
        info!("启动番剧 {} 的下载任务处理", worker.bangumi.name);

        // 初始化任务缓存
        if let Err(e) = worker
            .task_manager
            .init_bangumi_tasks(worker.sub.bangumi_id)
            .await
        {
            error!("初始化番剧 {} 的任务缓存失败: {}", worker.bangumi.name, e);
            return;
        }

        // 默认30秒处理一次任务
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(
            worker.sub.task_processor_interval.unwrap_or(5) as u64,
        ));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = worker.process_tasks().await {
                        error!("番剧 {} 处理下载任务失败: {}", worker.bangumi.name, e);
                    }
                }
                _ = stop_rx.recv() => {
                    info!("停止番剧 {} 的下载任务处理", worker.bangumi.name);
                    // 清除任务缓存
                    worker.task_manager.clear_bangumi_tasks(worker.sub.bangumi_id).await;
                    break;
                }
            }
        }
    }

    /// 启动 worker，开始定时处理任务
    pub fn spawn(self) {
        // 创建两个克隆，分别用于两个循环
        let worker_clone1 = self.clone();
        let worker_clone2 = self.clone();

        // 为两个线程分别创建停止信号接收器
        let stop_rx1 = self.stop_tx.subscribe();
        let stop_rx2 = self.stop_tx.subscribe();

        // 启动种子收集和处理循环
        tokio::spawn(async move {
            Self::run_collector(worker_clone1, stop_rx1).await;
        });

        // 启动任务状态处理循环
        tokio::spawn(async move {
            Self::run_task_processor(worker_clone2, stop_rx2).await;
        });
    }

    /// 停止 worker
    pub fn stop(&self) {
        // 发送停止信号，由于通道容量为2，这一次发送会被两个接收者接收到
        let _ = self.stop_tx.send(());
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
                info!(
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
