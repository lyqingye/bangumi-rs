#![deny(clippy::unused_async)]
pub mod config;
pub mod context;
pub mod db;
pub mod metrics;
mod retry;
mod syncer;
pub mod thirdparty;
pub mod worker;

use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use pan_115::model::DownloadInfo;
use std::{collections::HashMap, path::PathBuf};
use tokio::sync::broadcast;

use model::{
    sea_orm_active_enums::{DownloadStatus, ResourceType},
    torrent_download_tasks::Model,
    torrents::Model as TorrentModel,
};

#[async_trait]
pub trait Downloader: Send + Sync {
    fn name(&self) -> &'static str;
    async fn add_task(&self, resource: Resource, dir: PathBuf) -> Result<()>;
    async fn list_tasks(&self, info_hashes: &[String]) -> Result<Vec<Model>>;
    async fn list_files(&self, info_hash: &str) -> Result<Vec<pan_115::model::FileInfo>>;
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
pub enum Resource {
    // 磁力链接,InfoHash
    MagnetLink(String, String),
    // InfoHash
    MagnetInfoHash(String),
    // 种子文件字节,InfoHash
    TorrentFileBytes(Vec<u8>, String),
}

impl Resource {
    pub fn from_info_hash<T: Into<String>>(info_hash: T) -> Result<Self> {
        Ok(Resource::MagnetInfoHash(info_hash.into()))
    }

    pub fn from_magnet_link<T: Into<String>>(magnet_link: T) -> Result<Self> {
        let magnet_link = magnet_link.into();
        if let Some(part) = magnet_link.split("btih:").nth(1) {
            // 提取 InfoHash，它可能后跟其他参数（以 & 分隔）
            let info_hash = part.split('&').next().unwrap_or_default();
            if info_hash.len() == 40 {
                return Ok(Resource::MagnetLink(
                    magnet_link.clone(),
                    info_hash.to_string(),
                ));
            }
        }
        Err(anyhow::anyhow!("非法磁力链接，无法获取info_hash"))
    }

    pub fn from_torrent_file_bytes<T: Into<Vec<u8>>>(torrent_file_bytes: T) -> Result<Self> {
        let torrent_file_bytes = torrent_file_bytes.into();
        let torrent = torrent::Torrent::from_bytes(&torrent_file_bytes)?;
        Ok(Resource::TorrentFileBytes(
            torrent_file_bytes,
            torrent.info_hash_str()?,
        ))
    }

    pub fn get_type(&self) -> ResourceType {
        match self {
            Resource::MagnetLink(_, _) => ResourceType::Magnet,
            Resource::MagnetInfoHash(_) => ResourceType::InfoHash,
            Resource::TorrentFileBytes(_, _) => ResourceType::Torrent,
        }
    }

    pub fn magnet(&self) -> Option<String> {
        match self {
            Resource::MagnetLink(magnet, _) => Some(magnet.clone()),
            Resource::MagnetInfoHash(_) => {
                Some(format!("magnet:?xt=urn:btih:{}", self.info_hash()))
            }
            _ => None,
        }
    }

    pub fn info_hash(&self) -> &str {
        match self {
            Resource::MagnetInfoHash(hash) => hash,
            Resource::TorrentFileBytes(_, hash) => hash,
            Resource::MagnetLink(_, hash) => hash,
        }
    }
}

#[mockall::automock]
#[async_trait]
pub trait ThirdPartyDownloader: Send + Sync {
    fn name(&self) -> &'static str;
    async fn add_task(&self, resource: Resource, dir: PathBuf) -> Result<Option<String>>;
    async fn list_tasks(&self, info_hashes: &[String])
        -> Result<HashMap<String, RemoteTaskStatus>>;

    async fn list_files(
        &self,
        info_hash: &str,
        result: Option<String>,
    ) -> Result<Vec<pan_115::model::FileInfo>>;
    async fn download_file(&self, file_id: &str, ua: &str) -> Result<DownloadInfo>;
    async fn cancel_task(&self, info_hash: &str) -> Result<()>;
    async fn remove_task(&self, info_hash: &str, remove_files: bool) -> Result<()>;
    async fn pause_task(&self, info_hash: &str) -> Result<()>;
    async fn resume_task(&self, info_hash: &str) -> Result<()>;
    // 支持的资源类型
    fn supports_resource_type(&self, resource_type: ResourceType) -> bool;
    // 推荐的资源类型
    fn recommended_resource_type(&self) -> ResourceType;
}

#[async_trait]
pub trait Store: Send + Sync {
    async fn list_by_hashes(&self, info_hashes: &[String]) -> Result<Vec<Model>>;
    async fn list_by_status(&self, status: &[DownloadStatus]) -> Result<Vec<Model>>;
    async fn update_status(
        &self,
        info_hash: &str,
        status: DownloadStatus,
        err_msg: Option<String>,
        result: Option<String>,
    ) -> Result<()>;
    async fn update_retry_status(
        &self,
        info_hash: &str,
        next_retry_at: NaiveDateTime,
        err_msg: Option<String>,
    ) -> Result<()>;
    async fn upsert(&self, task: Model) -> Result<()>;
    async fn get_torrent_by_info_hash(&self, info_hash: &str) -> Result<Option<TorrentModel>>;
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
