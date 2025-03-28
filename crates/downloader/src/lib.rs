#![deny(clippy::unused_async)]
pub mod actor;
pub mod config;
pub mod context;
pub mod db;
pub mod dlrs;
pub mod errors;
pub mod metrics;
pub mod resource;
pub mod stm;
pub mod thirdparty;

use crate::errors::Result;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use resource::Resource;
use std::{
    collections::HashMap,
    fmt::{self, Display},
    path::PathBuf,
};
use tokio::sync::broadcast;

use model::{
    sea_orm_active_enums::{DownloadStatus, ResourceType},
    torrent_download_tasks::Model,
    torrents::Model as TorrentModel,
};

#[derive(Debug, Clone)]
pub struct DownloaderInfo {
    pub name: String,
    pub priority: u8,
}

#[derive(Debug, Clone)]
pub enum AccessType {
    Redirect,
    Forward,
}

#[derive(Debug, Clone)]
pub struct DownloadInfo {
    pub url: String,
    pub access_type: AccessType,
}

#[async_trait]
pub trait Downloader: Send + Sync {
    async fn add_task(
        &self,
        resource: Resource,
        dir: PathBuf,
        downloader: Option<String>,
        allow_fallback: bool,
    ) -> Result<()>;
    async fn list_tasks(&self, info_hashes: &[String]) -> Result<Vec<Model>>;
    async fn list_files(&self, info_hash: &str) -> Result<Vec<FileInfo>>;
    async fn download_file(&self, file_id: &str, ua: &str) -> Result<DownloadInfo>;
    async fn cancel_task(&self, info_hash: &str) -> Result<()>;
    async fn remove_task(&self, info_hash: &str, remove_files: bool) -> Result<()>;
    async fn metrics(&self) -> metrics::Metrics;
    async fn subscribe(&self) -> broadcast::Receiver<Event>;
    async fn retry(&self, info_hash: &str) -> Result<()>;
    async fn pause_task(&self, info_hash: &str) -> Result<()>;
    async fn resume_task(&self, info_hash: &str) -> Result<()>;
    fn supports_resource_type(&self, resource_type: ResourceType) -> bool;
    fn recommended_resource_type(&self) -> ResourceType;
    fn take_dlr(&self, downloader: &str) -> Option<&dyn ThirdPartyDownloader>;
    fn dlrs(&self) -> Vec<DownloaderInfo>;
}

#[derive(Debug, Clone)]
pub enum Event {
    /// 任务更新
    TaskUpdated((String, DownloadStatus, Option<String>)),
}

#[derive(Debug, Clone)]
pub struct RemoteTaskStatus {
    pub status: DownloadStatus,
    pub err_msg: Option<String>,
    pub result: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub file_id: String,
    pub file_name: String,
    pub file_size: usize,
    pub is_dir: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tid(String);

impl From<String> for Tid {
    fn from(tid: String) -> Self {
        Self(tid)
    }
}

impl From<Tid> for String {
    fn from(tid: Tid) -> Self {
        tid.0
    }
}

impl From<&str> for Tid {
    fn from(tid: &str) -> Self {
        Self(tid.to_string())
    }
}

impl Tid {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Tid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[mockall::automock]
#[async_trait]
pub trait ThirdPartyDownloader: Send + Sync {
    fn name(&self) -> &'static str;
    async fn add_task(
        &self,
        resource: Resource,
        dir: PathBuf,
    ) -> Result<(Option<Tid>, Option<String>)>;
    async fn list_tasks(&self, tid: &[Tid]) -> Result<HashMap<Tid, RemoteTaskStatus>>;

    async fn list_files(&self, tid: &Tid, result: Option<String>) -> Result<Vec<FileInfo>>;
    async fn dl_file(&self, file_id: &str, ua: &str) -> Result<DownloadInfo>;
    async fn cancel_task(&self, tid: &Tid) -> Result<()>;
    async fn remove_task(&self, tid: &Tid, remove_files: bool) -> Result<()>;
    async fn pause_task(&self, tid: &Tid) -> Result<()>;
    async fn resume_task(&self, tid: &Tid) -> Result<()>;
    // 支持的资源类型
    fn supports_resource_type(&self, resource_type: ResourceType) -> bool;
    // 推荐的资源类型
    fn recommended_resource_type(&self) -> ResourceType;
    fn config(&self) -> &config::GenericConfig;
}

#[async_trait]
pub trait Store: Send + Sync {
    async fn create(
        &self,
        resource: &Resource,
        dir: PathBuf,
        downloader: String,
        allow_fallback: bool,
    ) -> Result<()>;
    async fn get_by_hash(&self, info_hash: &str) -> Result<Option<Model>>;
    async fn load_resource(&self, info_hash: &str) -> Result<Option<Resource>>;
    async fn list_by_hashes(&self, info_hashes: &[String]) -> Result<Vec<Model>>;
    async fn list_by_status(&self, status: &[DownloadStatus]) -> Result<Vec<Model>>;
    async fn list_by_dlr_and_status(
        &self,
        downloader: &str,
        status: &[DownloadStatus],
    ) -> Result<Vec<Model>>;
    async fn update_status(
        &self,
        info_hash: &str,
        status: DownloadStatus,
        err_msg: Option<String>,
        result: Option<String>,
    ) -> Result<()>;
    async fn update_tid(&self, info_hash: &str, tid: &Tid) -> Result<()>;
    async fn update_retry_status(
        &self,
        info_hash: &str,
        next_retry_at: NaiveDateTime,
        err_msg: Option<String>,
    ) -> Result<()>;
    async fn get_torrent(&self, info_hash: &str) -> Result<Option<TorrentModel>>;
    async fn assign_dlr(&self, info_hash: &str, downloader: String) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_info_hash() {
        let resource = Resource::from_magnet_link("magnet:?xt=urn:btih:e93a1a84df5f95b0a350ef4c25b91c2c88adce4b&dn=filename&tr=tracker_url".to_string()).unwrap();
        assert_eq!(
            resource.info_hash(),
            "e93a1a84df5f95b0a350ef4c25b91c2c88adce4b"
        );
    }
}
