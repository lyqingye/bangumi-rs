use std::{
    collections::{HashMap, HashSet},
    num::NonZero,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use lru::LruCache;
use model::sea_orm_active_enums::DownloadStatus;
use pan_115::{
    errors::Pan115Error,
    model::{DownloadInfo, FileInfo, OfflineTaskStatus},
};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::{
    config, context::Pan115Context, RemoteTaskStatus, Resource, ResourceType, ThirdPartyDownloader,
};
use anyhow::Result;
use anyhow::{anyhow, Context};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct Config {
    pub download_cache_ttl: Duration,
    pub download_cache_size: usize,
    pub file_list_cache_size: usize,
    pub generic: config::GenericConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            download_cache_ttl: Duration::from_secs(60 * 60),
            download_cache_size: 16,
            file_list_cache_size: 16,
            generic: config::GenericConfig {
                max_retry_count: 5,
                retry_processor_interval: Duration::from_secs(30),
                retry_min_interval: chrono::Duration::seconds(30),
                retry_max_interval: chrono::Duration::minutes(10),
                download_timeout: chrono::Duration::minutes(30),
                delete_task_on_completion: true,
                priority: 0,
                download_dir: PathBuf::from("/downloads"),
            },
        }
    }
}

#[derive(Clone)]
#[allow(clippy::type_complexity)]
pub struct Pan115DownloaderImpl {
    pan115: pan_115::client::Client,
    path_cache: Arc<Mutex<HashMap<PathBuf, String>>>,
    download_cache: Arc<Mutex<LruCache<String, (DownloadInfo, std::time::Instant)>>>,
    file_list_cache: Arc<Mutex<LruCache<String, (Vec<FileInfo>, std::time::Instant)>>>,
    config: Config,
}

impl Pan115DownloaderImpl {
    pub fn new(pan115: pan_115::client::Client, config: Config) -> Self {
        Self {
            pan115,
            path_cache: Arc::new(Mutex::new(HashMap::new())),
            download_cache: Arc::new(Mutex::new(LruCache::new(
                NonZero::new(config.download_cache_size).unwrap(),
            ))),
            file_list_cache: Arc::new(Mutex::new(LruCache::new(
                NonZero::new(config.file_list_cache_size).unwrap(),
            ))),
            config,
        }
    }

    pub fn new_from_env() -> Result<Self> {
        let pan115 = pan_115::client::Client::new_from_env()?;
        let config = Config::default();
        Ok(Self::new(pan115, config))
    }
}

#[async_trait]
impl ThirdPartyDownloader for Pan115DownloaderImpl {
    fn name(&self) -> &'static str {
        "pan_115"
    }

    async fn add_task(&self, resource: Resource, dir: PathBuf) -> Result<Option<String>> {
        let dir = self.config.generic.download_dir.join(dir);
        let dir_cid = self.get_or_create_dir_cid(&dir).await?;
        let magnet = resource
            .magnet()
            .ok_or_else(|| anyhow::anyhow!("无法从资源中解析出磁力链接"))?;
        let info_hash = resource.info_hash();

        match self.pan115.add_offline_task(&[&magnet], &dir_cid).await {
            Ok(_) => {
                info!("成功添加下载任务到网盘: {}", info_hash);
                Ok(None)
            }
            Err(e) => match e {
                Pan115Error::OfflineTaskExisted => {
                    warn!("任务已在网盘中存在: {}", info_hash);
                    Ok(None)
                }
                _ => Err(anyhow::anyhow!("添加离线下载任务失败: {}", e)),
            },
        }
    }

    async fn list_tasks(
        &self,
        info_hashes: &[String],
    ) -> Result<HashMap<String, RemoteTaskStatus>> {
        let mut page = 0;
        let mut remote_tasks_status = HashMap::new();
        let target_hashes: HashSet<&String> = info_hashes.iter().collect();

        loop {
            debug!("获取离线下载任务列表: page={}", page);
            let resp = self
                .pan115
                .list_offline_tasks_page(page)
                .await
                .map_err(|e| anyhow!("获取网盘下载任务列表失败: {}", e))?;

            if resp.tasks.is_empty() {
                break;
            }

            let filtered_tasks: Vec<_> = resp
                .tasks
                .into_iter()
                .filter(|task| target_hashes.contains(&task.info_hash))
                .collect();

            debug!("获取到 {} 个匹配的任务", filtered_tasks.len());
            remote_tasks_status.extend(filtered_tasks.into_iter().map(|task| {
                let err_msg = if task.is_failed() {
                    Some(format!("离线下载失败: {:?}", task.status()))
                } else {
                    None
                };
                let context: Pan115Context = (&task).into();
                (
                    task.info_hash.clone(),
                    RemoteTaskStatus {
                        status: map_task_status(task.status()),
                        err_msg,
                        result: Some(context.try_into().unwrap_or_default()),
                    },
                )
            }));

            if remote_tasks_status.len() >= target_hashes.len() || resp.page_count == resp.page {
                break;
            }

            page = resp.page + 1;
        }

        Ok(remote_tasks_status)
    }

    async fn list_files(&self, _info_hash: &str, result: Option<String>) -> Result<Vec<FileInfo>> {
        match result {
            Some(result) => {
                let mut cache = self.file_list_cache.lock().await;
                let context: Pan115Context = serde_json::from_str(&result)?;
                let now = std::time::Instant::now();

                if let Some((files, last_update)) = cache.get(&context.file_id) {
                    let ttl = now.duration_since(*last_update);
                    if ttl < self.config.download_cache_ttl {
                        info!("命中缓存: file_id={}", context.file_id);
                        return Ok(files.clone());
                    }
                }

                let client = self.pan115.clone();
                let files = client.list_files_recursive(&context.file_id).await?;
                cache.put(context.file_id, (files.clone(), now));
                Ok(files)
            }
            None => Err(anyhow::anyhow!("该下载器不支持下载文件")),
        }
    }

    async fn download_file(&self, file_id: &str, ua: &str) -> Result<DownloadInfo> {
        let mut cache = self.download_cache.lock().await;
        let cache_key = format!("{}-{}", file_id, ua);

        let now = std::time::Instant::now();
        if let Some((download_info, last_update)) = cache.get(&cache_key) {
            let ttl = now.duration_since(*last_update);
            if ttl < self.config.download_cache_ttl {
                info!("命中缓存: file_id={}", file_id);
                return Ok(download_info.clone());
            }
        }
        let download_info = self
            .pan115
            .download_file(file_id, Some(ua))
            .await?
            .context("下载文件失败")?;
        cache.put(cache_key, (download_info.clone(), now));
        Ok(download_info)
    }

    async fn cancel_task(&self, info_hash: &str) -> Result<()> {
        self.pan115.delete_offline_task(&[info_hash], true).await?;
        Ok(())
    }

    async fn remove_task(&self, info_hash: &str, remove_files: bool) -> Result<()> {
        self.pan115
            .delete_offline_task(&[info_hash], remove_files)
            .await?;
        Ok(())
    }

    async fn pause_task(&self, _info_hash: &str) -> Result<()> {
        info!("115网盘不支持暂停任务");
        Ok(())
    }

    async fn resume_task(&self, _info_hash: &str) -> Result<()> {
        info!("115网盘不支持恢复任务");
        Ok(())
    }

    fn supports_resource_type(&self, resource_type: ResourceType) -> bool {
        matches!(resource_type, ResourceType::Magnet | ResourceType::InfoHash)
    }

    fn recommended_resource_type(&self) -> ResourceType {
        ResourceType::Magnet
    }

    fn config(&self) -> &config::GenericConfig {
        &self.config.generic
    }
}

impl Pan115DownloaderImpl {
    /// 从缓存中获取目录 CID
    async fn get_cached_cid(&self, path: &PathBuf) -> Option<String> {
        let cache = self.path_cache.lock().await;
        let cid = cache.get(path).cloned();
        if let Some(ref cid) = cid {
            debug!("命中目录缓存: path={}, cid={}", path.display(), cid);
        }
        cid
    }

    /// 缓存目录 CID
    async fn cache_path_cid(&self, path: PathBuf, cid: String) {
        debug!("缓存目录 CID: path={}, cid={}", path.display(), cid);
        let mut cache = self.path_cache.lock().await;
        cache.insert(path, cid);
    }

    /// 获取或创建目录 CID
    async fn get_or_create_dir_cid(&self, path: &PathBuf) -> Result<String> {
        if let Some(cid) = self.get_cached_cid(path).await {
            return Ok(cid);
        }

        debug!("创建网盘目录: path={}", path.display());
        let cid = self.pan115.mkdir_by_path(path.clone()).await?;
        self.cache_path_cid(path.clone(), cid.clone()).await;
        Ok(cid)
    }
}

fn map_task_status(status: OfflineTaskStatus) -> DownloadStatus {
    match status {
        OfflineTaskStatus::Pending => DownloadStatus::Pending,
        OfflineTaskStatus::Downloading => DownloadStatus::Downloading,
        OfflineTaskStatus::Completed => DownloadStatus::Completed,
        OfflineTaskStatus::Failed | OfflineTaskStatus::Unknow => DownloadStatus::Failed,
    }
}
