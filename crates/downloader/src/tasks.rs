use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use chrono::NaiveDateTime;
use model::{sea_orm_active_enums::DownloadStatus, torrent_download_tasks};
use tokio::sync::{broadcast, RwLock};

use crate::{context::Pan115Context, db::Db, Event};

/// 注意，这里只会存放待处理的任务，已失败或已结束的任务不会存放在这里
#[derive(Clone)]
pub struct TaskManager {
    db: Db,
    tasks: Arc<RwLock<HashMap<String, torrent_download_tasks::Model>>>,
    event_sender: broadcast::Sender<Event>,
}

impl TaskManager {
    pub async fn new(db: Db, event_sender: broadcast::Sender<Event>) -> Result<Self> {
        let tm = Self {
            db,
            tasks: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
        };
        tm.init_tasks().await?;
        Ok(tm)
    }

    pub async fn init_tasks(&self) -> Result<()> {
        let tasks = self
            .db
            .list_download_tasks_by_status(vec![
                DownloadStatus::Pending,
                DownloadStatus::Downloading,
                DownloadStatus::Retrying,
            ])
            .await?;
        let mut cache = self.tasks.write().await;
        for task in tasks {
            cache.insert(task.info_hash.clone(), task);
        }
        Ok(())
    }

    pub async fn get_by_info_hash(
        &self,
        info_hash: &str,
    ) -> Result<Option<torrent_download_tasks::Model>> {
        let cache = self.tasks.read().await;
        let task = cache.get(info_hash);
        Ok(task.cloned())
    }

    pub async fn get_by_info_hash_without_cache(
        &self,
        info_hash: &str,
    ) -> Result<Option<torrent_download_tasks::Model>> {
        let task = self.db.get_by_info_hash(info_hash).await?;
        Ok(task)
    }
    pub async fn list_by_info_hashes(
        &self,
        info_hashes: &[String],
    ) -> Result<Vec<torrent_download_tasks::Model>> {
        let mut tasks = Vec::new();
        let cache = self.tasks.read().await;
        for info_hash in info_hashes {
            if let Some(task) = cache.get(info_hash) {
                tasks.push(task.clone());
            }
        }
        Ok(tasks)
    }

    pub async fn list_by_info_hashes_without_cache(
        &self,
        info_hashes: &[String],
    ) -> Result<Vec<torrent_download_tasks::Model>> {
        self.db.list_download_tasks(info_hashes.to_vec()).await
    }

    pub async fn list_by_statues(
        &self,
        status: &[DownloadStatus],
    ) -> Result<Vec<torrent_download_tasks::Model>> {
        let mut tasks = Vec::new();

        let caches = self.tasks.read().await;
        for task in caches.values() {
            if status.contains(&task.download_status) {
                tasks.push(task.clone());
            }
        }

        Ok(tasks)
    }

    pub async fn update_task_status(
        &self,
        info_hash: &str,
        status: DownloadStatus,
        err_msg: Option<String>,
        context: Option<Pan115Context>,
    ) -> Result<()> {
        let mut cache = self.tasks.write().await;
        let task = cache
            .get_mut(info_hash)
            .ok_or_else(|| anyhow::anyhow!("任务不存在于缓存中"))?;

        self.db
            .update_task_status(
                info_hash,
                status.clone(),
                err_msg.clone(),
                context.clone().map(|c| c.try_into().unwrap_or_default()),
            )
            .await?;

        // 推送事件
        let _ = self.event_sender.send(Event::TaskUpdated((
            task.info_hash.clone(),
            status.clone(),
            err_msg.clone(),
        )));

        // 如果任务已经结束，则从缓存中移除
        match status {
            DownloadStatus::Failed | DownloadStatus::Completed | DownloadStatus::Cancelled => {
                cache.remove(info_hash);
            }
            _ => {
                task.download_status = status;
                task.err_msg = err_msg;
                task.context = context.map(|c| c.try_into().unwrap_or_default());
            }
        }

        Ok(())
    }

    pub async fn update_task_retry_status(
        &self,
        info_hash: &str,
        retry_count: i32,
        next_retry_at: NaiveDateTime,
        err_msg: Option<String>,
    ) -> Result<()> {
        let mut cache = self.tasks.write().await;
        let task = cache
            .get_mut(info_hash)
            .ok_or_else(|| anyhow::anyhow!("任务不存在于缓存中"))?;
        self.db
            .update_task_retry_status(info_hash, retry_count, next_retry_at, err_msg.clone())
            .await?;
        task.retry_count = retry_count;
        task.next_retry_at = next_retry_at;
        task.download_status = DownloadStatus::Retrying;
        task.err_msg = err_msg;
        Ok(())
    }

    pub async fn batch_upsert_tasks(
        &self,
        tasks: Vec<torrent_download_tasks::Model>,
    ) -> Result<()> {
        self.db.batch_upsert_download_tasks(tasks.clone()).await?;
        let mut cache = self.tasks.write().await;
        for task in tasks {
            match cache.get_mut(&task.info_hash) {
                Some(t) => {
                    t.updated_at = task.updated_at;
                }
                None => {
                    cache.insert(task.info_hash.clone(), task);
                }
            }
        }
        Ok(())
    }

    pub async fn tasks_count(&self) -> usize {
        let cache = self.tasks.read().await;
        cache.len()
    }
}
