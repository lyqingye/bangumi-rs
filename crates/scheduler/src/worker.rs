use anyhow::Result;
use model::sea_orm_active_enums::{ResourceType, State};
use model::{bangumi, episodes, file_name_parse_record, subscriptions, torrents};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info};

use crate::db::Db;
use crate::metrics::{WorkerMetrics, WorkerState};
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
    pub(crate) task_manager: TaskManager,
    // 命令发送器
    cmd_tx: broadcast::Sender<WorkerCommand>,
    selector: TorrentSelector,
    pub(crate) bangumi: bangumi::Model,
    pub(crate) sub: subscriptions::Model,
    pub metrics: Arc<RwLock<WorkerMetrics>>,
    pub recommended_resource_type: ResourceType,
    client: reqwest::Client,
}

impl BangumiWorker {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sub: subscriptions::Model,
        bangumi: bangumi::Model,
        db: Db,
        parser: parser::worker::Worker,
        metadata: metadata::worker::Worker,
        task_manager: TaskManager,
        recommended_resource_type: ResourceType,
        client: reqwest::Client,
    ) -> Self {
        let (cmd_tx, _) = broadcast::channel(16);
        let selector = TorrentSelector::new(&sub);
        let metrics = Arc::new(RwLock::new(WorkerMetrics::new(
            bangumi.name.clone(),
            WorkerState::Idle,
        )));
        Self {
            sub,
            bangumi,
            db,
            parser,
            metadata,
            task_manager,
            cmd_tx,
            selector,
            metrics,
            recommended_resource_type,
            client,
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
        let mut episode_torrents: Vec<_> = torrent_pairs
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
                    if self.sub.enforce_torrent_release_after_broadcast == 1 {
                        if let Some(episode) = episodes.get(&actual_ep) {
                            if let Some(air_date) = episode.air_date {
                                if torrent.pub_date < air_date.into() {
                                    return false;
                                }
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
        loop {
            let best = self.selector.select(&episode_torrents);
            if let Some(ref best) = best {
                // 如果推荐资源类型为种子，则尝试获取种子数据
                if self.recommended_resource_type == ResourceType::Torrent {
                    let torrent = self
                        .db
                        .get_torrent_by_info_hash(&best.info_hash)
                        .await?
                        .ok_or_else(|| anyhow::anyhow!("种子不存在"))?;
                    if torrent.data.is_none() || torrent.data.unwrap().is_empty() {
                        // 如果提供种子下载地址，则尝试
                        if let Some(download_url) = &torrent.download_url {
                            match self.download_torrent(download_url).await {
                                Ok(data) => {
                                    // 下载成功，那么则选择该种子
                                    self.db.update_torrent_data(&best.info_hash, data).await?;
                                    return Ok(Some(best.clone()));
                                }
                                Err(e) => {
                                    error!("下载种子失败: {}", e);
                                }
                            }
                        }

                        // 如果无法获取种子数据，那么则移除该种子
                        episode_torrents.retain(|(torrent, _)| torrent.info_hash == best.info_hash);
                        continue;
                    }
                    return Ok(Some(best.clone()));
                } else {
                    // 如果推荐资源类型为磁力链接，则直接返回
                    return Ok(Some(best.clone()));
                }
            } else {
                // 如果无法选择到种子，则返回 None
                return Ok(None);
            }
        }
    }

    pub async fn download_torrent(&self, download_url: &str) -> Result<Vec<u8>> {
        let response = self.client.get(download_url).send().await?;
        let data = response.bytes().await?;
        Ok(data.to_vec())
    }

    /// 获取当前 worker metrics
    pub fn get_metrics(&self) -> WorkerMetrics {
        self.metrics.read().unwrap().clone()
    }

    /// 收集并解析种子，然后为任务选择合适的种子
    async fn collect_and_process_torrents(&self) -> Result<()> {
        // 1. 收集种子
        info!("开始收集番剧 {} 的种子", self.bangumi.name);
        self.metadata
            .request_refresh_torrents_and_wait(self.bangumi.id)
            .await?;

        // 2. 获取并解析种子
        let torrents_file_names = self
            .db
            .get_bangumi_torrents_file_names(self.bangumi.id)
            .await?;
        if !torrents_file_names.is_empty() {
            info!("开始解析番剧 {} 的种子文件名", self.bangumi.name);
            self.parser.parse_file_names(torrents_file_names).await?;
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

        if tasks.is_empty() {
            info!(
                "番剧 {} 所有任务剧集下载已经完成, 标记订阅状态为完成, Worker 将自动停止",
                self.bangumi.name
            );
            self.db
                .update_subscription_as_downloaded(self.bangumi.id)
                .await?;
            self.shutdown_no_wait();
            return Ok(());
        }

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

        // 创建三个定时器
        let mut collector_interval = tokio::time::interval(std::time::Duration::from_secs(
            worker.sub.collector_interval.unwrap_or(60 * 30) as u64,
        ));

        let mut metadata_interval = tokio::time::interval(std::time::Duration::from_secs(
            worker.sub.metadata_interval.unwrap_or(60 * 60 * 24) as u64,
        ));

        loop {
            tokio::select! {
                _ = collector_interval.tick() => {
                    {
                        let mut metrics = worker.metrics.write().unwrap();
                        metrics.set_state(WorkerState::Collecting);
                    }
                    if let Err(e) = worker.collect_and_process_torrents().await {
                        error!("番剧 {} 处理种子失败: {}", worker.bangumi.name, e);
                    }
                    {
                        let mut metrics = worker.metrics.write().unwrap();
                        metrics.set_state(WorkerState::Idle);
                    }
                }
                _ = metadata_interval.tick() => {
                    if let Err(e) = worker
                        .metadata
                        .request_refresh_metadata(worker.bangumi.id, false)
                    {
                        error!("番剧 {} 刷新元数据失败: {}", worker.bangumi.name, e);
                    }
                }
                Ok(cmd) = cmd_rx.recv() => {
                    match cmd {
                        WorkerCommand::Shutdown(tx) => {
                            info!("停止番剧 {} 的后台处理", worker.bangumi.name);
                            let _ = tx.send(()).await;
                            break;
                        }
                        WorkerCommand::TriggerCollection => {
                            info!("手动触发番剧 {} 的种子收集", worker.bangumi.name);
                            {
                                let mut metrics = worker.metrics.write().unwrap();
                                metrics.set_state(WorkerState::Collecting);
                            }
                            if let Err(e) = worker.collect_and_process_torrents().await {
                                error!("番剧 {} 处理种子失败: {}", worker.bangumi.name, e);
                            }
                            {
                                let mut metrics = worker.metrics.write().unwrap();
                                metrics.set_state(WorkerState::Idle);
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

    fn shutdown_no_wait(&self) {
        // 创建一个 mpsc 通道来等待 worker 完全停止
        let (tx, _) = mpsc::channel(1);
        // 发送停止命令
        let _ = self.cmd_tx.send(WorkerCommand::Shutdown(tx));
    }

    /// 触发种子收集
    pub fn trigger_collection(&self) {
        // 发送触发收集命令
        let _ = self.cmd_tx.send(WorkerCommand::TriggerCollection);
    }
}
