use anyhow::{Context, Result};
use async_trait::async_trait;
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    num::NonZero,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use lru::LruCache;
use model::sea_orm_active_enums::{DownloadStatus, ResourceType};
use transmission_rpc::{
    types::{Id, TorrentAction, TorrentAddArgs, TorrentGetField, TorrentStatus},
    SharableTransClient,
};

use crate::{
    config,
    context::{TorrentContext, TorrentFileInfo},
    resource::Resource,
    AccessType, DownloadInfo, FileInfo, RemoteTaskStatus, ThirdPartyDownloader,
};

use base64::{engine::general_purpose::STANDARD, Engine};

#[derive(Clone)]
pub struct Config {
    pub generic: config::GenericConfig,
    pub file_list_cache_size: usize,
    pub url: String,
    pub username: String,
    pub password: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            generic: config::GenericConfig {
                max_retry_count: 1,
                retry_min_interval: chrono::Duration::seconds(30),
                retry_max_interval: chrono::Duration::minutes(60),
                download_timeout: chrono::Duration::hours(1),
                delete_task_on_completion: false,
                priority: 0,
                download_dir: PathBuf::from("/downloads/complete"),
            },
            file_list_cache_size: 16,
            url: "http://127.0.0.1:9091/transmission/rpc".to_string(),
            username: "admin".to_string(),
            password: "123456".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct TransmissionDownloaderImpl {
    cli: Arc<SharableTransClient>,
    config: Config,
    file_cache: Arc<Mutex<LruCache<String, String>>>,
}

impl TransmissionDownloaderImpl {
    pub fn new(config: Config) -> Self {
        let auth = transmission_rpc::types::BasicAuth {
            user: config.username.clone(),
            password: config.password.clone(),
        };
        let sharable_client =
            SharableTransClient::with_auth(reqwest::Url::parse(&config.url).unwrap(), auth);

        Self {
            cli: Arc::new(sharable_client),
            file_cache: Arc::new(Mutex::new(LruCache::new(
                NonZero::new(config.file_list_cache_size).unwrap(),
            ))),
            config,
        }
    }

    #[cfg(test)]
    pub fn new_from_env() -> Result<Self> {
        let config = Config {
            url: std::env::var("TRANSMISSION_URL")?,
            username: std::env::var("TRANSMISSION_USER")?,
            password: std::env::var("TRANSMISSION_PASSWORD")?,
            ..Default::default()
        };
        Ok(Self::new(config))
    }
}

#[async_trait]
impl ThirdPartyDownloader for TransmissionDownloaderImpl {
    fn name(&self) -> &'static str {
        "transmission"
    }

    async fn add_task(&self, resource: Resource, dir: PathBuf) -> Result<Option<String>> {
        if dir.is_absolute() {
            return Err(anyhow::anyhow!("保存路径必须为相对路径"));
        }

        let save_dir = self.config.generic.download_dir.join(dir);
        let info_hash = resource.info_hash().to_owned();

        let (filename, metainfo) = match resource {
            Resource::MagnetInfoHash(_) | Resource::MagnetLink(_, _) => {
                let magnet = resource.magnet().unwrap_or_default();
                (Some(magnet), None)
            }
            Resource::TorrentFileBytes(data, _) => {
                // Transmission 需要 base64 编码的 torrent 文件内容
                let encoded = STANDARD.encode(&data);
                (None, Some(encoded))
            }
            Resource::TorrentURL(url, _) => (Some(url), None),
        };

        let args = TorrentAddArgs {
            download_dir: Some(save_dir.to_string_lossy().to_string()),
            filename,
            metainfo,
            paused: Some(false),
            ..Default::default()
        };

        // 直接使用Arc里的引用
        let resp = self
            .cli
            .torrent_add(args)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        if !resp.is_ok() {
            return Err(anyhow::anyhow!("添加种子任务失败: {}", resp.result));
        }

        Ok(Some(info_hash))
    }

    async fn list_tasks(
        &self,
        info_hashes: &[String],
    ) -> Result<HashMap<String, RemoteTaskStatus>> {
        if info_hashes.is_empty() {
            return Ok(HashMap::new());
        }

        // 使用 hash_string 字段过滤任务
        let ids: Vec<Id> = info_hashes
            .iter()
            .map(|hash| Id::Hash(hash.clone()))
            .collect();
        let fields = vec![
            TorrentGetField::HashString,
            TorrentGetField::Status,
            TorrentGetField::Name,
            TorrentGetField::ErrorString,
            TorrentGetField::Files,
            TorrentGetField::DownloadDir,
            TorrentGetField::PercentDone,
        ];

        // 直接使用Arc里的引用
        let resp = self
            .cli
            .torrent_get(Some(fields), Some(ids))
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        if !resp.is_ok() {
            return Err(anyhow::anyhow!("获取种子任务列表失败: {}", resp.result));
        }

        let mut result = HashMap::new();
        let torrents = resp.arguments.torrents;
        for torrent in torrents {
            let hash = torrent.hash_string.clone().context("任务缺少哈希值")?;
            let (status, err_msg) = map_task_status(&torrent);
            let mut ctx = TorrentContext::default();

            // 对于完成的任务，收集文件信息
            if status == DownloadStatus::Completed {
                if let Some(files) = &torrent.files {
                    if let Some(download_dir) = &torrent.download_dir {
                        let download_dir_str = self
                            .config
                            .generic
                            .download_dir
                            .to_string_lossy()
                            .to_string();
                        ctx.dir = download_dir.replace(&download_dir_str, "");

                        ctx.files = files
                            .iter()
                            .map(|f| TorrentFileInfo {
                                name: f.name.clone(),
                                size: f.length as usize,
                            })
                            .collect();
                    }
                }
            }

            let remote_task_status = RemoteTaskStatus {
                status,
                err_msg,
                result: Some(ctx.try_into()?),
            };

            result.insert(hash, remote_task_status);
        }

        Ok(result)
    }

    async fn cancel_task(&self, info_hash: &str) -> Result<()> {
        let ids = vec![Id::Hash(info_hash.to_string())];
        // 直接使用Arc里的引用
        self.cli
            .torrent_action(TorrentAction::Stop, ids)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(())
    }

    async fn remove_task(&self, info_hash: &str, remove_files: bool) -> Result<()> {
        let ids = vec![Id::Hash(info_hash.to_string())];
        // 直接使用Arc里的引用
        self.cli
            .torrent_remove(ids, remove_files)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(())
    }

    async fn pause_task(&self, info_hash: &str) -> Result<()> {
        let ids = vec![Id::Hash(info_hash.to_string())];
        // 直接使用Arc里的引用
        self.cli
            .torrent_action(TorrentAction::Stop, ids)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(())
    }

    async fn resume_task(&self, info_hash: &str) -> Result<()> {
        let ids = vec![Id::Hash(info_hash.to_string())];
        // 直接使用Arc里的引用
        self.cli
            .torrent_action(TorrentAction::Start, ids)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(())
    }

    async fn list_files(&self, _info_hash: &str, result: Option<String>) -> Result<Vec<FileInfo>> {
        let ctx = result.context("没有下载结果，请确保已经成功下载")?;
        let ctx = TorrentContext::try_from(ctx)?;

        let files = ctx
            .files
            .into_iter()
            .map(|f| {
                let path = Path::new(&ctx.dir).join(&f.name);
                let file_name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or(f.name.clone());

                let mut hasher = DefaultHasher::new();
                path.hash(&mut hasher);
                let file_id = hasher.finish().to_string();

                // 将文件路径缓存起来，以便后续下载使用
                self.file_cache
                    .lock()
                    .unwrap()
                    .put(file_id.clone(), path.to_string_lossy().to_string());

                FileInfo {
                    file_id,
                    file_name,
                    file_size: f.size,
                    is_dir: false,
                }
            })
            .collect();

        Ok(files)
    }

    async fn download_file(&self, file_id: &str, _ua: &str) -> Result<DownloadInfo> {
        let mut file_cache = self.file_cache.lock().unwrap();
        let file_info = file_cache.get(file_id);
        if let Some(file_path) = file_info {
            Ok(DownloadInfo {
                url: format!("{}/{}", self.name(), file_path),
                access_type: AccessType::Forward,
            })
        } else {
            Err(anyhow::anyhow!("文件不存在或缓存已过期"))
        }
    }

    fn supports_resource_type(&self, resource_type: ResourceType) -> bool {
        matches!(
            resource_type,
            ResourceType::Magnet
                | ResourceType::InfoHash
                | ResourceType::Torrent
                | ResourceType::TorrentURL
        )
    }

    fn recommended_resource_type(&self) -> ResourceType {
        ResourceType::TorrentURL
    }

    fn config(&self) -> &config::GenericConfig {
        &self.config.generic
    }
}

// 辅助函数：将 Transmission 的状态映射到我们的状态
fn map_task_status(torrent: &transmission_rpc::types::Torrent) -> (DownloadStatus, Option<String>) {
    if torrent.percent_done.is_none() && torrent.status.is_none() {
        return (DownloadStatus::Pending, None);
    }

    // 检查是否已完成
    if let Some(percent_done) = torrent.percent_done {
        if percent_done >= 1.0 {
            return (DownloadStatus::Completed, None);
        }
    }

    // 检查错误
    if let Some(err_str) = &torrent.error_string {
        if !err_str.is_empty() {
            return (DownloadStatus::Failed, Some(err_str.clone()));
        }
    }

    // 根据状态映射
    match torrent.status {
        Some(TorrentStatus::Stopped) => (DownloadStatus::Paused, None),
        Some(TorrentStatus::QueuedToVerify) | Some(TorrentStatus::Verifying) => {
            (DownloadStatus::Downloading, None)
        }
        Some(TorrentStatus::QueuedToDownload) => (DownloadStatus::Pending, None),
        Some(TorrentStatus::Downloading) => (DownloadStatus::Downloading, None),
        Some(TorrentStatus::QueuedToSeed) | Some(TorrentStatus::Seeding) => {
            (DownloadStatus::Completed, None)
        }
        None => (DownloadStatus::Pending, None),
    }
}
