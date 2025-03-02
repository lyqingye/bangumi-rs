use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use lru::LruCache;
use model::sea_orm_active_enums::DownloadStatus;
use pan_115::{
    errors::Pan115Error,
    model::{DownloadInfo, OfflineTaskStatus},
};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::{context::Pan115Context, RemoteTaskStatus, ThirdPartyDownloader};
use anyhow::Result;
use anyhow::{anyhow, Context};
use async_trait::async_trait;

#[derive(Clone)]
pub struct Pan115DownloaderImpl {
    pan115: pan_115::client::Client,
    path_cache: Arc<Mutex<HashMap<PathBuf, String>>>,
    file_name_cache: Arc<Mutex<LruCache<String, (DownloadInfo, std::time::Instant)>>>,
    download_cache_ttl: Duration,
}

fn create_magnet_link(info_hash: &str) -> String {
    format!("magnet:?xt=urn:btih:{}", info_hash)
}

#[async_trait]
impl ThirdPartyDownloader for Pan115DownloaderImpl {
    fn name(&self) -> &'static str {
        "pan_115"
    }

    async fn add_task(&self, info_hash: &str, dir: PathBuf) -> Result<Option<String>> {
        let dir_cid = self.get_or_create_dir_cid(&dir).await?;
        let magnet = create_magnet_link(info_hash);

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
                (
                    task.info_hash.clone(),
                    RemoteTaskStatus {
                        status: map_task_status(task.status()),
                        err_msg,
                        result: None,
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

    async fn download_file(
        &self,
        info_hash: &str,
        ua: &str,
        result: Option<String>,
    ) -> Result<DownloadInfo> {
        let mut cache = self.file_name_cache.lock().await;
        let cache_key = format!("{}-{}", info_hash, ua);

        let now = std::time::Instant::now();
        if let Some((download_info, last_update)) = cache.get(&cache_key) {
            let ttl = now.duration_since(*last_update);
            if ttl < self.download_cache_ttl {
                info!("命中缓存: info_hash={}", info_hash);
                return Ok(download_info.clone());
            }
        }
        match result {
            Some(result) => {
                let context: Pan115Context = serde_json::from_str(&result)?;
                let mut client = self.pan115.clone();
                let expect_file_name = context.file_name.clone();
                let files = client
                    .list_files_with_fn(&context.file_id, move |file| {
                        !file.is_dir() && file.name == expect_file_name
                    })
                    .await?;
                if files.is_empty() {
                    return Err(anyhow::anyhow!("文件不存在"));
                }
                let file = files.first().unwrap();
                let download_info = client
                    .download_file(&file.file_id, Some(ua))
                    .await?
                    .context("下载文件失败")?;
                cache.put(cache_key, (download_info.clone(), now));
                Ok(download_info)
            }
            None => Err(anyhow::anyhow!("该下载器不支持下载文件")),
        }
    }

    async fn cancel_task(&self, info_hash: &str) -> Result<()> {
        self.pan115.delete_offline_task(&[info_hash], true).await?;
        Ok(())
    }

    async fn remove_task(&self, info_hash: &str) -> Result<()> {
        self.pan115.delete_offline_task(&[info_hash], true).await?;
        Ok(())
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
