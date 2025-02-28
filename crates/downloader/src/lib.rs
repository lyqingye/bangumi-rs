#![allow(unused)]
pub mod context;
pub mod db;
pub mod metrics;
pub mod pan_115_dl;
mod tasks;

use anyhow::Result;
use async_trait::async_trait;
use pan_115::model::DownloadInfo;
use std::path::PathBuf;
use tokio::sync::broadcast;

use model::{sea_orm_active_enums::DownloadStatus, torrent_download_tasks::Model};

#[async_trait]
pub trait Downloader: Send + Sync {
    fn name(&self) -> &'static str;
    async fn add_task(&self, info_hash: &str, dir: PathBuf) -> Result<()>;
    async fn list_tasks(&self, info_hashes: &[String]) -> Result<Vec<Model>>;
    async fn download_file(&self, info_hash: &str, ua: &str) -> Result<DownloadInfo>;
    async fn cancel_task(&self, info_hash: &str) -> Result<()>;
    async fn metrics(&self) -> metrics::Metrics;
    async fn subscribe(&self) -> broadcast::Receiver<Event>;
    async fn retry(&self, info_hash: &str) -> Result<()>;
}

#[derive(Debug, Clone)]
pub enum Event {
    /// 任务更新
    TaskUpdated((String, DownloadStatus, Option<String>)),
}
