#![allow(unused)]
pub mod config;
pub mod context;
pub mod db;
pub mod metrics;
pub mod pan_115_dl;
mod retry;
mod syncer;
mod tasks;
pub mod thirdparty;
pub mod worker;

use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use mockall::automock;
use pan_115::model::DownloadInfo;
use std::{collections::HashMap, path::PathBuf};
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

#[derive(Debug, Clone)]
pub struct RemoteTaskStatus {
    pub status: DownloadStatus,
    pub err_msg: Option<String>,
    pub result: Option<String>,
}

#[mockall::automock]
#[async_trait]
pub trait ThirdPartyDownloader: Send + Sync {
    fn name(&self) -> &'static str;
    async fn add_task(&self, info_hash: &str, dir: PathBuf) -> Result<Option<String>>;
    async fn list_tasks(&self, info_hashes: &[String])
        -> Result<HashMap<String, RemoteTaskStatus>>;
    async fn download_file(
        &self,
        info_hash: &str,
        ua: &str,
        result: Option<String>,
    ) -> Result<DownloadInfo>;
    async fn cancel_task(&self, info_hash: &str) -> Result<()>;
    async fn remove_task(&self, info_hash: &str) -> Result<()>;
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
}
