use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::{resource::Resource, RemoteTaskStatus, ThirdPartyDownloader};
use anyhow::Result;
use async_trait::async_trait;
use model::sea_orm_active_enums::{DownloadStatus, ResourceType};
use pan_115::model::{DownloadInfo, FileInfo};
use qbittorrent::model::{
    torrent::{
        AddTorrentArg, GetTorrentListArg, Hashes, State, Torrent, TorrentFile, TorrentSource,
    },
    Sep,
};
use reqwest::Url;

#[derive(Debug, Clone)]
pub struct Config {
    pub save_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            save_path: PathBuf::from("/downloads"),
        }
    }
}

#[derive(Clone)]
pub struct QbittorrentDownloaderImpl {
    cli: Arc<qbittorrent::client::Client>,
    config: Config,
}

impl QbittorrentDownloaderImpl {
    pub fn new(cli: qbittorrent::client::Client, config: Config) -> Self {
        Self {
            cli: Arc::new(cli),
            config,
        }
    }
}

#[async_trait]
impl ThirdPartyDownloader for QbittorrentDownloaderImpl {
    fn name(&self) -> &'static str {
        "qbittorrent"
    }

    async fn add_task(&self, resource: Resource, dir: PathBuf) -> Result<Option<String>> {
        if dir.is_absolute() {
            return Err(anyhow::anyhow!("保存路径必须为相对路径"));
        }
        let info_hash = resource.info_hash();
        let dir = self.config.save_path.join(dir);
        let source = match resource {
            Resource::MagnetInfoHash(_) | Resource::MagnetLink(_, _) => {
                let magnet = resource.magnet().unwrap_or_default();
                TorrentSource::Urls {
                    urls: Sep::from(vec![magnet.parse::<Url>()?]),
                }
            }
            Resource::TorrentFileBytes(data, info_hash) => {
                let torrent = TorrentFile {
                    filename: format!("{}.torrent", info_hash),
                    data,
                };
                TorrentSource::TorrentFiles {
                    torrents: vec![torrent],
                }
            }
        };
        let arg = AddTorrentArg {
            source,
            savepath: Some(dir.to_string_lossy().to_string()),
            ..Default::default()
        };

        self.cli.add_torrent(arg).await?;
        Ok(Some(info_hash.to_owned()))
    }

    async fn list_tasks(
        &self,
        info_hashes: &[String],
    ) -> Result<HashMap<String, RemoteTaskStatus>> {
        let arg = GetTorrentListArg {
            hashes: Some(Sep::<String, '|'>::from(info_hashes).to_string()),
            ..Default::default()
        };
        let torrents = self.cli.get_torrent_list(arg).await?;
        let mut result = HashMap::new();
        for torrent in torrents {
            let (status, err_msg) = map_task_status(&torrent);
            let remote_task_status = RemoteTaskStatus {
                status,
                err_msg,
                result: None,
            };
            result.insert(torrent.hash.unwrap(), remote_task_status);
        }
        Ok(result)
    }

    async fn cancel_task(&self, info_hash: &str) -> Result<()> {
        self.cli
            .stop_torrents(Hashes::Hashes(Sep::from(vec![info_hash.to_string()])))
            .await?;
        Ok(())
    }

    async fn remove_task(&self, info_hash: &str, remove_files: bool) -> Result<()> {
        self.cli
            .delete_torrents(
                Hashes::Hashes(Sep::from(vec![info_hash.to_string()])),
                remove_files,
            )
            .await?;
        Ok(())
    }

    async fn pause_task(&self, info_hash: &str) -> Result<()> {
        self.cli
            .stop_torrents(Hashes::Hashes(Sep::from(vec![info_hash.to_string()])))
            .await?;
        Ok(())
    }

    async fn resume_task(&self, info_hash: &str) -> Result<()> {
        self.cli
            .start_torrents(Hashes::Hashes(Sep::from(vec![info_hash.to_string()])))
            .await?;
        Ok(())
    }

    async fn list_files(&self, _info_hash: &str, _result: Option<String>) -> Result<Vec<FileInfo>> {
        return Err(anyhow::anyhow!("不支持获取下载文件列表"));
    }

    async fn download_file(&self, _file_id: &str, _ua: &str) -> Result<DownloadInfo> {
        return Err(anyhow::anyhow!("不支持下载文件"));
    }

    fn supports_resource_type(&self, resource_type: ResourceType) -> bool {
        matches!(
            resource_type,
            ResourceType::Magnet | ResourceType::InfoHash | ResourceType::Torrent
        )
    }

    fn recommended_resource_type(&self) -> ResourceType {
        ResourceType::Torrent
    }
}

fn map_task_status(torrent: &Torrent) -> (DownloadStatus, Option<String>) {
    if torrent.state.is_none() {
        return (DownloadStatus::Pending, None);
    }
    let state = torrent.state.as_ref().unwrap();

    // 如果进度为100%，则认为是已完成
    if let Some(progress) = torrent.progress {
        if progress == 1.0 {
            return (DownloadStatus::Completed, None);
        }
    }
    match state {
        State::Error => (DownloadStatus::Failed, Some("下载失败".to_string())),
        State::MissingFiles => (DownloadStatus::Failed, Some("文件缺失".to_string())),
        State::Unknown => (
            DownloadStatus::Failed,
            Some("下载失败，未知错误".to_string()),
        ),
        State::PausedDL => (DownloadStatus::Paused, None),

        _ => (DownloadStatus::Downloading, None),
    }
}
