use async_trait::async_trait;
use chrono::{Local, NaiveDateTime};
use downloader::errors::{Error, Result};
use downloader::resource::Resource;
use downloader::{Store, Tid};
use model::sea_orm_active_enums::{DownloadStatus, ResourceType};
use model::torrent_download_tasks::Model;
use model::torrents::Model as TorrentModel;
use std::collections::HashMap;
use std::path::PathBuf;
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

    async fn list_by_dlr_and_status(
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

    async fn get_torrent(&self, info_hash: &str) -> Result<Option<TorrentModel>> {
        let torrents = self.torrents.read().await;
        Ok(torrents.get(info_hash).cloned())
    }

    async fn assign_dlr(&self, info_hash: &str, downloader: String) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(info_hash) {
            task.downloader = downloader;
            task.retry_count = 0;
        }
        Ok(())
    }

    async fn update_tid(&self, info_hash: &str, tid: &Tid) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(info_hash) {
            task.tid = Some(tid.to_string());
        }
        Ok(())
    }

    async fn get_by_hash(&self, info_hash: &str) -> Result<Option<Model>> {
        let tasks = self.tasks.read().await;
        Ok(tasks.get(info_hash).cloned())
    }

    async fn load_resource(&self, info_hash: &str) -> Result<Option<Resource>> {
        let task = self.get_by_hash(info_hash).await?;
        if let Some(task) = task {
            match task.resource_type {
                ResourceType::InfoHash => {
                    Ok(Some(Resource::from_info_hash(task.info_hash.clone())?))
                }
                _ => Err(Error::UnsupportedResourceType(task.resource_type)),
            }
        } else {
            Ok(None)
        }
    }

    async fn create(
        &self,
        resource: &Resource,
        dir: PathBuf,
        downloader: String,
        allow_fallback: bool,
    ) -> Result<()> {
        let now = Local::now().naive_utc();
        let mut tasks = self.tasks.write().await;
        tasks.insert(
            resource.info_hash().to_string(),
            Model {
                info_hash: resource.info_hash().to_string(),
                download_status: DownloadStatus::Pending,
                downloader: downloader.to_string(),
                allow_fallback,
                context: None,
                err_msg: None,
                created_at: now,
                updated_at: now,
                dir: dir.to_string_lossy().into_owned(),
                retry_count: 0,
                next_retry_at: now,
                resource_type: resource.get_type(),
                magnet: resource.magnet(),
                torrent_url: resource.torrent_url(),
                tid: None,
            },
        );
        Ok(())
    }
}
