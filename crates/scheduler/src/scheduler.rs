use anyhow::{Context, Result};
use downloader::Downloader;
use model::sea_orm_active_enums::ResourceType;
use model::subscriptions;
use sea_orm::DatabaseConnection;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::db::Db;
use crate::download_torrent;
use crate::metrics::Metrics;
use crate::tasks::TaskManager;
use crate::worker::BangumiWorker;

/// 调度器，负责管理所有番剧的下载任务
#[derive(Clone)]
pub struct Scheduler {
    pub(crate) db: Db,
    pub(crate) parser: parser::worker::Worker,
    pub(crate) metadata: metadata::worker::Worker,
    pub(crate) downloader: Arc<Box<dyn Downloader>>,
    pub(crate) task_manager: TaskManager,
    pub(crate) workers: Arc<Mutex<HashMap<i32, BangumiWorker>>>, // 存储 worker 实例以便管理生命周期
    pub(crate) notify: notify::worker::Worker,
    pub(crate) client: reqwest::Client,
}

impl Scheduler {
    pub fn new(
        db: Db,
        parser: parser::worker::Worker,
        metadata: metadata::worker::Worker,
        downloader: Arc<Box<dyn Downloader>>,
        notify: notify::worker::Worker,
        client: reqwest::Client,
    ) -> Self {
        let task_manager = TaskManager::new(db.clone(), downloader.clone(), notify.clone());
        Self {
            db,
            parser,
            metadata,
            downloader,
            task_manager,
            workers: Arc::new(Mutex::new(HashMap::new())),
            notify,
            client,
        }
    }

    pub fn new_with_conn(
        conn: Arc<DatabaseConnection>,
        parser: parser::worker::Worker,
        metadata: metadata::worker::Worker,
        downloader: Arc<Box<dyn Downloader>>,
        notify: notify::worker::Worker,
        client: reqwest::Client,
    ) -> Self {
        let db = Db::new(conn);
        Self::new(db, parser, metadata, downloader, notify, client)
    }

    /// 创建并启动 worker 或重启 worker
    pub(crate) async fn spawn_new_or_restart_worker(
        &self,
        sub: subscriptions::Model,
    ) -> Result<()> {
        let bangumi_id = sub.bangumi_id;
        let mut workers = self.workers.lock().await;

        // 如果已存在worker，先停止它
        if let Some(existing_worker) = workers.remove(&bangumi_id) {
            info!("停止番剧 {} 的现有下载任务处理器", bangumi_id);
            if let Err(e) = existing_worker.shutdown().await {
                error!("停止番剧 {} 的下载任务处理器失败: {}", bangumi_id, e);
            }
        }

        // 创建新的worker
        let bangumi = self.db.get_bangumi_by_id(bangumi_id).await?;
        if bangumi.is_none() {
            return Err(anyhow::anyhow!("未找到番剧记录"));
        }
        let bangumi = bangumi.unwrap();

        let worker = BangumiWorker::new(
            sub,
            bangumi,
            self.db.clone(),
            self.parser.clone(),
            self.metadata.clone(),
            self.task_manager.clone(),
            self.downloader.recommended_resource_type(),
            self.client.clone(),
        );
        let worker_clone = worker.clone();
        worker.spawn();
        workers.insert(bangumi_id, worker_clone);
        info!("已为番剧 {} 创建下载任务处理器", bangumi_id);

        Ok(())
    }

    pub async fn spawn(&mut self) -> Result<()> {
        self.task_manager.spawn()?;

        // 获取所有已订阅的番剧
        let subscriptions = self.db.get_active_subscriptions().await?;

        // 为新订阅的番剧创建并启动 worker
        for subscription in subscriptions {
            self.spawn_new_or_restart_worker(subscription).await?;
        }

        info!("启动下载调度器");
        Ok(())
    }

    pub fn get_downloader(&self) -> Arc<Box<dyn Downloader>> {
        self.downloader.clone()
    }

    /// 优雅停机
    pub async fn shutdown(&self) -> Result<()> {
        info!("开始调度器优雅停机...");

        // 1. 停止所有 worker
        let mut workers = self.workers.lock().await;
        for (bangumi_id, worker) in workers.iter() {
            info!("停止番剧 {} 的下载任务处理器", bangumi_id);
            if let Err(e) = worker.shutdown().await {
                error!("停止番剧 {} 的下载任务处理器失败: {}", bangumi_id, e);
            }
        }
        workers.clear();

        // 2. 停止相关组件
        if let Err(e) = self.parser.shutdown().await {
            error!("停止解析器时发生错误: {}", e);
        }
        if let Err(e) = self.metadata.shutdown().await {
            error!("停止元数据服务时发生错误: {}", e);
        }
        if let Err(e) = self.notify.shutdown().await {
            error!("停止通知服务时发生错误: {}", e);
        }
        if let Err(e) = self.task_manager.stop().await {
            error!("停止任务管理器时发生错误: {}", e);
        }

        info!("调度器优雅停机完成");
        Ok(())
    }

    /// 获取调度器的 metrics
    pub async fn metrics(&self) -> Metrics {
        let workers = self.workers.lock().await;
        let mut metrics = Metrics::default();

        for worker in workers.values() {
            metrics.workers.push(worker.get_metrics());
        }

        metrics
    }
}

impl Scheduler {
    pub async fn manual_select_episode_torrent(
        &self,
        bangumi_id: i32,
        episode_number: i32,
        info_hash: &str,
    ) -> Result<()> {
        let torrent = self
            .db
            .get_torrent_by_info_hash(info_hash)
            .await?
            .context("未找到种子信息")?;
        let subscribe = self
            .db
            .get_subscription(bangumi_id)
            .await?
            .context("你需要先订阅番剧")?;

        // 如果推荐资源类型为种子，则尝试下载种子
        #[allow(clippy::collapsible_if)]
        if self
            .task_manager
            .use_torrent_to_download(&subscribe, &torrent)
        {
            if torrent.data.is_none() && torrent.download_url.is_some() {
                match download_torrent(&self.client, &torrent.download_url.unwrap()).await {
                    Ok(data) => {
                        info!("下载种子成功: {}, 更新种子数据", info_hash);
                        self.db.update_torrent_data(info_hash, data).await?;
                    }
                    Err(e) => {
                        warn!("下载种子失败: {}, 尝试使用磁力链接下载", e);
                    }
                }
            }
        }

        let task = self
            .db
            .get_episode_task_by_bangumi_id_and_episode_number(bangumi_id, episode_number)
            .await?;

        // 如果任务存在，则取消之前的任务
        if let Some(old_task) = task {
            if let Some(ref_info_hash) = old_task.ref_torrent_info_hash {
                self.downloader.cancel_task(&ref_info_hash).await?;
            }
        } else {
            // 插入新的剧集下载任务
            self.task_manager
                .batch_create_tasks(bangumi_id, vec![episode_number])
                .await?;
        }

        self.task_manager
            .update_task_ready(bangumi_id, episode_number, info_hash)
            .await?;
        Ok(())
    }

    pub async fn retry_task(&self, bangumi_id: i32, episode_number: i32) -> Result<()> {
        self.task_manager
            .retry_task(bangumi_id, episode_number)
            .await?;
        Ok(())
    }

    pub async fn trigger_collection(&self, bangumi_id: i32) -> Result<()> {
        let mut workers = self.workers.lock().await;
        if let Some(worker) = workers.get_mut(&bangumi_id) {
            worker.trigger_collection();
        }
        Ok(())
    }
}
