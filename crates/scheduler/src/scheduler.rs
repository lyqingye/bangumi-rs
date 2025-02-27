use anyhow::Result;
use downloader::Downloader;
use model::subscriptions;
use sea_orm::DatabaseConnection;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::db::Db;
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
}

impl Scheduler {
    pub fn new(
        db: Db,
        parser: parser::worker::Worker,
        metadata: metadata::worker::Worker,
        downloader: Arc<Box<dyn Downloader>>,
        notify: notify::worker::Worker,
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
        }
    }

    pub fn new_with_conn(
        conn: Arc<DatabaseConnection>,
        parser: parser::worker::Worker,
        metadata: metadata::worker::Worker,
        downloader: Arc<Box<dyn Downloader>>,
        notify: notify::worker::Worker,
    ) -> Self {
        let db = Db::new(conn);
        Self::new(db, parser, metadata, downloader, notify)
    }

    /// 创建并启动 worker
    pub(crate) async fn spawn_worker(&self, sub: subscriptions::Model) -> Result<()> {
        let bangumi_id = sub.bangumi_id;
        let mut workers = self.workers.lock().await;
        if let std::collections::hash_map::Entry::Vacant(e) = workers.entry(bangumi_id) {
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
            );
            let worker_clone = worker.clone();
            worker.spawn();
            e.insert(worker_clone);
            info!("已为番剧 {} 创建下载任务处理器", bangumi_id);
        }
        Ok(())
    }

    pub async fn spawn(&mut self) -> Result<()> {
        self.task_manager.spawn().await?;

        // 获取所有已订阅的番剧
        let subscriptions = self.db.get_active_subscriptions().await?;

        // 为新订阅的番剧创建并启动 worker
        for subscription in subscriptions {
            self.spawn_worker(subscription).await?;
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
        let torrent = self.db.get_torrent_by_info_hash(info_hash).await?;
        if torrent.is_none() {
            return Err(anyhow::anyhow!("未找到种子信息"));
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

    pub async fn trigger_collection(&self, bangumi_id: i32) -> Result<()> {
        let mut workers = self.workers.lock().await;
        if let Some(worker) = workers.get_mut(&bangumi_id) {
            worker.trigger_collection();
        }
        Ok(())
    }
}
