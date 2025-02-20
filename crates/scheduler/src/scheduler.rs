use anyhow::{Context as _, Result};
use downloader::Downloader;
use metadata;
use model::subscriptions;
use parser;
use sea_orm::DatabaseConnection;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::db::Db;
use crate::tasks::TaskManager;
use crate::worker::BangumiWorker;

/// 调度器，负责管理所有番剧的下载任务
#[derive(Clone)]
pub struct Scheduler {
    db: Db,
    parser: parser::worker::Worker,
    metadata: metadata::worker::Worker,
    downloader: Arc<Box<dyn Downloader>>,
    task_manager: TaskManager,
    workers: Arc<Mutex<HashMap<i32, BangumiWorker>>>, // 存储 worker 实例以便管理生命周期
    notify: notify::worker::Worker,
}

impl Scheduler {
    pub fn new(
        db: Db,
        parser: parser::worker::Worker,
        metadata: metadata::worker::Worker,
        downloader: Arc<Box<dyn Downloader>>,
        notify: notify::worker::Worker,
    ) -> Self {
        let task_manager = TaskManager::new(db.clone());
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
    async fn spawn_worker(&self, sub: subscriptions::Model) -> Result<()> {
        let bangumi_id = sub.bangumi_id;
        let mut workers = self.workers.lock().await;
        if !workers.contains_key(&bangumi_id) {
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
                self.downloader.clone(),
                self.task_manager.clone(),
                self.notify.clone(),
            );
            let worker_clone = worker.clone();
            worker.spawn();
            workers.insert(bangumi_id, worker_clone);
            info!("已为番剧 {} 创建下载任务处理器", bangumi_id);
        }
        Ok(())
    }

    pub async fn spawn(&self) -> Result<()> {
        // 获取所有已订阅的番剧
        let subscriptions = self.db.get_active_subscriptions().await?;

        // 为新订阅的番剧创建并启动 worker
        for subscription in subscriptions {
            self.spawn_worker(subscription).await?;
        }

        info!("启动下载调度器");
        Ok(())
    }

    pub async fn trigger_collection(&self, bangumi_id: i32) -> Result<()> {
        let mut workers = self.workers.lock().await;
        if let Some(worker) = workers.get_mut(&bangumi_id) {
            worker.trigger_collection();
        } else {
            self.metadata
                .request_refresh_metadata(bangumi_id, false)
                .await?;
        }
        Ok(())
    }

    /// 订阅番剧
    pub async fn subscribe(
        &self,
        bangumi_id: i32,
        start_episode_number: Option<i32>,
        resolution_filter: Option<Vec<parser::VideoResolution>>,
        language_filter: Option<Vec<parser::Language>>,
        release_group_filter: Option<String>,
        collector_interval: Option<i32>,
        metadata_interval: Option<i32>,
        task_processor_interval: Option<i32>,
    ) -> Result<()> {
        // 将分辨率列表转换为逗号分隔的字符串
        let resolution_filter_str = resolution_filter.map(|resolutions| {
            resolutions
                .into_iter()
                .filter(|res| *res != parser::VideoResolution::Unknown)
                .map(|res| res.to_string())
                .collect::<Vec<_>>()
                .join(",")
        });

        // 将语言列表转换为逗号分隔的字符串
        let language_filter_str = language_filter.map(|langs| {
            langs
                .into_iter()
                .filter(|lang| *lang != parser::Language::Unknown)
                .map(|lang| lang.to_string())
                .collect::<Vec<_>>()
                .join(",")
        });

        // 刷新元数据
        self.metadata
            .request_refresh_metadata(bangumi_id, false)
            .await?;

        // 4. 获取所有剧集信息
        let episodes = self.db.get_bangumi_episodes(bangumi_id).await?;
        if episodes.is_empty() {
            return Err(anyhow::anyhow!("未找到剧集信息, 无法订阅"));
        }

        let default_start_episode_number =
            episodes.iter().map(|episode| episode.number).min().unwrap();

        let start_episode = start_episode_number.unwrap_or(default_start_episode_number);

        // 5. 收集需要下载的剧集编号
        let episode_numbers: Vec<i32> = episodes
            .iter()
            .filter(|episode| episode.number >= start_episode)
            .map(|episode| episode.number)
            .collect();

        // 6. 批量创建下载任务
        if !episode_numbers.is_empty() {
            info!(
                "为番剧 {} 批量创建第 {} 到 {} 集的下载任务",
                bangumi_id,
                episode_numbers.first().unwrap(),
                episode_numbers.last().unwrap()
            );
            self.task_manager
                .batch_create_tasks(bangumi_id, episode_numbers)
                .await?;
        }

        self.db
            .upsert_subscription(
                bangumi_id,
                Some(start_episode),
                resolution_filter_str,
                language_filter_str,
                release_group_filter,
                collector_interval,
                metadata_interval,
                task_processor_interval,
            )
            .await
            .context("更新订阅状态失败")?;

        let subscription = self
            .db
            .get_subscription(bangumi_id)
            .await?
            .expect("未找到订阅记录");

        // 7. 创建并启动新的 worker
        self.spawn_worker(subscription).await?;

        Ok(())
    }

    /// 取消订阅番剧
    pub async fn unsubscribe(&self, bangumi_id: i32) -> Result<()> {
        // 更新订阅状态为未订阅
        if let Err(e) = self.db.unsubscribe(bangumi_id).await {
            error!("更新订阅状态失败: {}", e);
            return Err(e);
        }

        // 停止并移除对应的 worker
        let mut workers = self.workers.lock().await;
        if let Some(worker) = workers.remove(&bangumi_id) {
            if let Err(e) = worker.shutdown().await {
                error!(
                    "停止番剧 {} 的下载任务处理器失败: {}",
                    worker.bangumi.name, e
                );
            } else {
                info!("已停止番剧 {} 的下载任务处理器", worker.bangumi.name);
            }
        }

        Ok(())
    }

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
        }
        self.task_manager
            .update_task_ready(bangumi_id, episode_number, info_hash)
            .await?;
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

        info!("调度器优雅停机完成");
        Ok(())
    }
}
