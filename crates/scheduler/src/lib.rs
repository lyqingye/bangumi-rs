#![deny(clippy::unused_async)]
mod db;
pub mod metrics;
mod scheduler;
mod selector;
mod subscribe;
mod tasks;
mod worker;

pub use db::Db;
pub use scheduler::Scheduler;
pub use selector::TorrentSelector;
pub use tasks::TaskManager;
use torrent::Torrent;
pub use worker::BangumiWorker;

use anyhow::Result;

pub async fn download_torrent(client: &reqwest::Client, download_url: &str) -> Result<Vec<u8>> {
    let response = client.get(download_url).send().await?;
    let data = response.bytes().await?;
    let bz = data.to_vec();

    // 尝试解析为 Torrent
    let _ = Torrent::from_bytes(&bz)?;
    Ok(bz)
}
