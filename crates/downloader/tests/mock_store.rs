use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use downloader::Store;
use model::sea_orm_active_enums::DownloadStatus;
use model::torrent_download_tasks::Model;
use model::torrents::Model as TorrentModel;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Default)]
pub struct MockStore {
    tasks: Arc<RwLock<HashMap<String, Model>>>,
    torrents: Arc<RwLock<HashMap<String, TorrentModel>>>,
}

#[allow(unused)]
impl MockStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn insert_task(&self, task: Model) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        tasks.insert(task.info_hash.clone(), task);
        Ok(())
    }

    pub async fn get_tasks(&self) -> Vec<Model> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    pub async fn insert_torrent(&self, torrent: TorrentModel) -> Result<()> {
        let mut torrents = self.torrents.write().await;
        torrents.insert(torrent.info_hash.clone(), torrent);
        Ok(())
    }
}

#[async_trait]
impl Store for MockStore {
    async fn list_by_hashes(&self, info_hashes: &[String]) -> Result<Vec<Model>> {
        let tasks = self.tasks.read().await;
        let result = info_hashes
            .iter()
            .filter_map(|hash| tasks.get(hash).cloned())
            .collect();
        Ok(result)
    }

    async fn list_by_status(&self, status: &[DownloadStatus]) -> Result<Vec<Model>> {
        let tasks = self.tasks.read().await;
        let result = tasks
            .values()
            .filter(|task| status.contains(&task.download_status))
            .cloned()
            .collect();
        Ok(result)
    }

    async fn list_by_downloader_and_status(
        &self,
        downloader: &str,
        status: &[DownloadStatus],
    ) -> Result<Vec<Model>> {
        let tasks = self.tasks.read().await;
        let result = tasks
            .values()
            .filter(|task| task.downloader == downloader && status.contains(&task.download_status))
            .cloned()
            .collect();
        Ok(result)
    }

    async fn update_status(
        &self,
        info_hash: &str,
        status: DownloadStatus,
        err_msg: Option<String>,
        result: Option<String>,
    ) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(info_hash) {
            task.download_status = status;
            task.err_msg = err_msg;
            task.context = result;
        }
        Ok(())
    }

    async fn upsert(&self, task: Model) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        tasks.insert(task.info_hash.clone(), task);
        Ok(())
    }

    async fn update_retry_status(
        &self,
        info_hash: &str,
        next_retry_at: NaiveDateTime,
        err_msg: Option<String>,
    ) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(info_hash) {
            task.retry_count += 1;
            task.err_msg = err_msg;
            task.next_retry_at = next_retry_at;
        }
        Ok(())
    }

    async fn get_torrent_by_info_hash(&self, info_hash: &str) -> Result<Option<TorrentModel>> {
        let torrents = self.torrents.read().await;
        Ok(torrents.get(info_hash).cloned())
    }

    async fn assign_downloader(&self, info_hash: &str, downloader: String) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(info_hash) {
            task.downloader = downloader;
            task.retry_count = 0;
        }
        Ok(())
    }
}
