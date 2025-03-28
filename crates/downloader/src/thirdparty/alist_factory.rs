use crate::errors::{Error, Result};
use crate::{
    AccessType, DownloadInfo, FileInfo, RemoteTaskStatus, ThirdPartyDownloader, Tid, config,
    context::TorrentContext, resource::Resource,
};
use anyhow::Context;
use async_trait::async_trait;
use lru::LruCache;
use model::sea_orm_active_enums::{DownloadStatus, ResourceType};
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    num::NonZero,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tracing::info;

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
                download_dir: PathBuf::from("/downloads"),
            },
            file_list_cache_size: 16,
            url: "http://127.0.0.1:5244".to_string(),
            username: "admin".to_string(),
            password: "123456".to_string(),
        }
    }
}

pub async fn create_downloader(
    config: config::GenericConfig,
    url: String,
    username: String,
    password: String,
    tool: alist::Tools,
) -> Result<impl ThirdPartyDownloader> {
    let config = Config {
        generic: config,
        file_list_cache_size: 16,
        url: url.clone(),
        username: username.clone(),
        password: password.clone(),
    };
    let mut client = alist::AListClient::new(url, username, password);
    client.login().await?;

    Ok(match tool {
        alist::Tools::Qbittorrent => AlistDownloaderImpl::new(
            config,
            client,
            tool,
            ResourceType::TorrentURL,
            vec![ResourceType::TorrentURL, ResourceType::Magnet],
        ),
        alist::Tools::Transmission => AlistDownloaderImpl::new(
            config,
            client.clone(),
            tool,
            ResourceType::TorrentURL,
            vec![ResourceType::TorrentURL, ResourceType::Magnet],
        ),
        alist::Tools::Pan115 | alist::Tools::PikPak => AlistDownloaderImpl::new(
            config,
            client.clone(),
            tool,
            ResourceType::Magnet,
            vec![ResourceType::Magnet],
        ),
    })
}

#[derive(Clone)]
pub struct AlistDownloaderImpl {
    config: Config,
    client: alist::AListClient,
    file_cache: Arc<Mutex<LruCache<String, String>>>,
    tool: alist::Tools,
    recommended_resource_type: ResourceType,
    supports_resource_type: Vec<ResourceType>,
}

impl AlistDownloaderImpl {
    pub fn new(
        config: Config,
        client: alist::AListClient,
        tool: alist::Tools,
        recommended_resource_type: ResourceType,
        supports_resource_type: Vec<ResourceType>,
    ) -> Self {
        let cache = LruCache::new(NonZero::new(config.file_list_cache_size).unwrap());
        Self {
            config,
            client,
            file_cache: Arc::new(Mutex::new(cache)),
            tool,
            recommended_resource_type,
            supports_resource_type,
        }
    }
}

#[async_trait]
impl ThirdPartyDownloader for AlistDownloaderImpl {
    fn name(&self) -> &'static str {
        match self.tool {
            alist::Tools::Qbittorrent => "alist:qbittorrent",
            alist::Tools::Transmission => "alist:transmission",
            alist::Tools::Pan115 => "alist:115",
            alist::Tools::PikPak => "alist:pikpak",
        }
    }

    async fn add_task(
        &self,
        resource: Resource,
        dir: PathBuf,
    ) -> Result<(Option<Tid>, Option<String>)> {
        let url = match resource {
            Resource::TorrentURL(url, _) => url,
            Resource::MagnetInfoHash(_) | Resource::MagnetLink(_, _) => {
                resource.magnet().unwrap_or_default()
            }
            _ => {
                return Err(Error::UnsupportedResourceType(resource.get_type()));
            }
        };

        let path = self.config.generic.download_dir.join(dir);
        let request = alist::AddOfflineDownloadTaskRequest {
            urls: vec![url.to_string()],
            path: path.to_string_lossy().to_string(),
            tool: self.tool,
            delete_policy: alist::DeletePolicy::DeleteNever,
        };
        let result = self.client.add_offline_download_task(request).await?;
        if result.tasks.is_empty() {
            return Err(anyhow::anyhow!("添加任务失败, 没有返回任何结果").into());
        }

        let ctx = TorrentContext {
            dir: path.to_string_lossy().to_string(),
            files: vec![],
        };
        Ok((
            Some(Tid::from(result.tasks.first().unwrap().id.clone())),
            Some(ctx.try_into()?),
        ))
    }

    async fn list_tasks(&self, tids: &[Tid]) -> Result<HashMap<Tid, RemoteTaskStatus>> {
        let mut tasks = HashMap::new();
        for tid in tids {
            let task = self
                .client
                .get_task_info(alist::TaskType::OfflineDownload, tid.as_str())
                .await?
                .with_context(|| "获取任务信息失败")?;
            let (status, err_msg) = map_task_status(task);
            tasks.insert(
                tid.clone(),
                RemoteTaskStatus {
                    status,
                    err_msg,
                    result: None,
                },
            );
        }
        Ok(tasks)
    }

    async fn cancel_task(&self, tid: &Tid) -> Result<()> {
        self.client
            .cancel_task(alist::TaskType::OfflineDownload, tid.as_str())
            .await?;
        self.client
            .delete_task(alist::TaskType::OfflineDownload, tid.as_str())
            .await?;
        Ok(())
    }

    async fn remove_task(&self, tid: &Tid, _remove_files: bool) -> Result<()> {
        self.client
            .delete_task(alist::TaskType::OfflineDownload, tid.as_str())
            .await?;
        Ok(())
    }

    async fn pause_task(&self, _tid: &Tid) -> Result<()> {
        info!("alist 不支持暂停任务");
        Ok(())
    }

    async fn resume_task(&self, _tid: &Tid) -> Result<()> {
        info!("alist 不支持恢复任务");
        Ok(())
    }

    async fn list_files(&self, tid: &Tid, result: Option<String>) -> Result<Vec<FileInfo>> {
        let ctx = result.context(Error::NoDownloadResult(tid.to_string()))?;
        let ctx = TorrentContext::try_from(ctx)?;
        let files = self
            .client
            .list_recursive_files(ctx.dir.as_str(), None::<String>, false, Some(10))
            .await?;
        let files = files
            .files
            .iter()
            .filter(|file| !file.file.is_dir)
            .map(|file| {
                let mut hasher = DefaultHasher::new();
                file.full_path.hash(&mut hasher);
                let file_id = hasher.finish().to_string();
                let fi = FileInfo {
                    file_id: file_id.clone(),
                    file_name: file.file.name.clone(),
                    file_size: file.file.size as usize,
                    is_dir: false,
                };

                {
                    let mut file_cache = self.file_cache.lock().unwrap();
                    file_cache.put(file_id.clone(), file.full_path.clone());
                }
                fi
            })
            .collect();
        Ok(files)
    }

    async fn dl_file(&self, file_id: &str, _ua: &str) -> Result<DownloadInfo> {
        let file_path = {
            let mut file_cache = self.file_cache.lock().unwrap();
            file_cache.get(file_id).cloned()
        };

        if let Some(file_path) = file_path {
            let file = self
                .client
                .get_file(&file_path, None::<String>)
                .await?
                .ok_or(Error::FileNotFound(file_path))?;
            let download_info = DownloadInfo {
                url: file.raw_url.clone(),
                access_type: AccessType::Redirect,
            };

            Ok(download_info)
        } else {
            return Err(Error::FileNotFound(file_path.unwrap_or_default()));
        }
    }

    fn supports_resource_type(&self, resource_type: ResourceType) -> bool {
        self.supports_resource_type.contains(&resource_type)
    }

    fn recommended_resource_type(&self) -> ResourceType {
        self.recommended_resource_type.clone()
    }

    fn config(&self) -> &config::GenericConfig {
        &self.config.generic
    }
}

fn map_task_status(task: alist::TaskInfo) -> (DownloadStatus, Option<String>) {
    match task.state {
        alist::TaskState::Errored | alist::TaskState::Failed | alist::TaskState::Failing => (
            DownloadStatus::Failed,
            Some(format!("下载失败: {}", task.error.unwrap_or_default())),
        ),
        alist::TaskState::Succeeded => (DownloadStatus::Completed, None),
        alist::TaskState::Canceling | alist::TaskState::Canceled => {
            (DownloadStatus::Cancelled, None)
        }
        _ => (DownloadStatus::Downloading, None),
    }
}
