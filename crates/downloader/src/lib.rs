#![allow(unused)]
pub mod context;
pub mod db;
pub mod pan_115_dl;
mod tasks;

use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

use model::torrent_download_tasks::Model;

#[async_trait]
pub trait Downloader: Send + Sync {
    fn name(&self) -> &'static str;
    async fn add_task(&self, info_hash: &str, dir: PathBuf) -> Result<()>;
    async fn list_tasks(&self, info_hashes: &[String]) -> Result<Vec<Model>>;
    async fn download_file_as_response(&self, info_hash: &str)
        -> Result<Option<reqwest::Response>>;
}
