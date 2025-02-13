use sea_orm::DatabaseConnection;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{Local, NaiveDateTime};
use model::{sea_orm_active_enums::DownloadStatus, torrent_download_tasks::Model};
use pan_115::{
    client::Client as Pan115Client,
    errors::Pan115Error,
    model::{OfflineTask, OfflineTaskStatus},
};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error, info, warn};

use crate::{db::Db, tasks::TaskManager, Downloader};

type RetryQueue = Arc<Mutex<HashMap<String, (Model, u32)>>>;
type TaskCache = Arc<Mutex<HashMap<String, chrono::NaiveDateTime>>>;

/// 下载器配置
#[derive(Debug, Clone)]
pub struct Config {
    /// 状态同步间隔
    pub sync_interval: Duration,
    /// 请求队列大小
    pub request_queue_size: usize,
    /// 最大重试次数
    pub max_retry_count: i32,
    /// 重试任务间隔
    pub retry_processor_interval: Duration,
    /// 下载目录
    pub download_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sync_interval: Duration::from_secs(60),
            request_queue_size: 100,
            max_retry_count: 5,
            retry_processor_interval: Duration::from_secs(5),
            download_dir: PathBuf::from("/"),
        }
    }
}

impl Config {
    pub fn calculate_next_retry(&self, retry_count: i32) -> NaiveDateTime {
        const BASE_SECONDS: u64 = 30; // 基础时间：30秒
        const MAX_MINUTES: u64 = 60; // 最大间隔：60分钟
        const MAX_SECONDS: u64 = MAX_MINUTES * 60; // 最大间隔（秒）：3600秒

        // 计算当前重试间隔（秒）
        let delay_seconds = BASE_SECONDS * 2u64.pow((retry_count as u32).min(7));

        // 确保不超过最大间隔
        let final_delay = delay_seconds.min(MAX_SECONDS);

        Local::now().naive_utc() + Duration::from_secs(final_delay)
    }
}

/// 115网盘下载器
#[derive(Clone)]
pub struct Pan115Downloader {
    tasks: TaskManager,
    pan115: Pan115Client,
    download_sender: Option<mpsc::Sender<String>>,
    is_spawned: Arc<std::sync::atomic::AtomicBool>,
    config: Config,
    path_cache: Arc<Mutex<HashMap<PathBuf, String>>>,
}

// 公共接口实现
#[async_trait]
impl Downloader for Pan115Downloader {
    /// 获取下载器名称
    fn name(&self) -> &'static str {
        "pan_115"
    }

    /// 添加下载任务
    ///
    /// # Arguments
    /// * `info_hash` - 种子的 info hash
    /// * `dir` - 下载目录路径
    async fn add_task(&self, info_hash: &str, dir: PathBuf) -> Result<()> {
        info!(
            "添加下载任务: info_hash={}, dir={}",
            info_hash,
            dir.display()
        );
        self.create_task(info_hash, &dir).await?;
        self.send_to_queue(info_hash.to_string()).await
    }

    /// 获取指定任务列表
    ///
    /// # Arguments
    /// * `info_hashes` - 要查询的任务的 info hash 列表
    async fn list_tasks(&self, info_hashes: &[String]) -> Result<Vec<Model>> {
        self.tasks
            .list_by_info_hashes_without_cache(info_hashes)
            .await
    }
}

// 初始化相关方法
impl Pan115Downloader {
    /// 使用数据库连接创建下载器实例
    pub async fn new_with_conn(
        conn: Arc<DatabaseConnection>,
        pan115: Pan115Client,
        config: Config,
    ) -> Result<Self> {
        info!("创建下载器实例: config={:?}", config);
        let db = Db::new(conn);
        Self::new(db, pan115, config).await
    }

    /// 创建下载器实例
    pub async fn new(db: Db, pan115: Pan115Client, config: Config) -> Result<Self> {
        info!("创建下载器实例: config={:?}", config);
        let tasks = TaskManager::new(db).await?;
        Ok(Self {
            tasks,
            pan115,
            download_sender: None,
            is_spawned: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            config,
            path_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// 从环境变量创建下载器实例
    pub async fn new_from_env() -> Result<Self> {
        info!("从环境变量创建下载器实例");
        let db = Db::new_from_env().await?;
        let mut pan115 = Pan115Client::new_from_env()?;
        pan115.login_check().await?;
        Self::new(db, pan115, Config::default()).await
    }

    /// 启动下载服务
    pub async fn spawn(&mut self) -> Result<()> {
        info!("启动下载服务");
        if !self.try_set_spawned() {
            return Err(anyhow!("下载服务已经启动"));
        }

        let (sender, receiver) = mpsc::channel(self.config.request_queue_size);
        self.download_sender = Some(sender.clone());

        // 恢复未处理完的任务
        self.recover_pending_tasks(sender).await?;

        // 启动后台任务
        self.spawn_background_tasks(receiver);
        info!("下载服务启动完成");
        Ok(())
    }

    /// 尝试设置服务启动标志
    fn try_set_spawned(&self) -> bool {
        self.is_spawned
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
            )
            .is_ok()
    }
}

// 辅助方法
impl Pan115Downloader {
    /// 创建磁力链接
    fn create_magnet_link(info_hash: &str) -> String {
        format!("magnet:?xt=urn:btih:{}", info_hash)
    }
}

// 任务管理相关方法
impl Pan115Downloader {
    /// 创建下载任务
    async fn create_task(&self, info_hash: &str, dir: &PathBuf) -> Result<()> {
        let full_path = self.config.download_dir.join(dir);
        info!(
            "创建下载任务: info_hash={}, dir={}",
            info_hash,
            full_path.display()
        );

        let now = Local::now().naive_utc();
        let task = Model {
            info_hash: info_hash.to_string(),
            download_status: DownloadStatus::Pending,
            downloader: Some(self.name().to_string()),
            context: None,
            err_msg: None,
            created_at: now,
            updated_at: now,
            dir: full_path.to_string_lossy().into_owned(),
            retry_count: 0,
            next_retry_at: now,
        };

        self.tasks.batch_upsert_tasks(vec![task.clone()]).await?;
        Ok(())
    }

    /// 发送任务到处理队列
    async fn send_to_queue(&self, info_hash: String) -> Result<()> {
        info!("发送任务到队列: info_hash={}", info_hash);
        let sender = self
            .download_sender
            .as_ref()
            .ok_or_else(|| anyhow!("下载服务未启动"))?;

        sender
            .send(info_hash)
            .await
            .map_err(|e| anyhow!("发送下载任务到队列失败: {}", e))
    }
}

// 任务处理相关方法
impl Pan115Downloader {
    /// 启动后台任务
    fn spawn_background_tasks(&self, receiver: mpsc::Receiver<String>) {
        info!("启动后台任务处理器");
        self.spawn_task_processor(receiver);
        self.spawn_retry_processor();
        self.spawn_status_syncer();
    }

    /// 启动任务处理器
    fn spawn_task_processor(&self, mut receiver: mpsc::Receiver<String>) {
        info!("启动任务处理器");
        let svc = self.clone();
        tokio::spawn(async move {
            while let Some(info_hash) = receiver.recv().await {
                if let Err(e) = svc.process_task(&info_hash).await {
                    error!("处理下载任务失败: {} - {}", info_hash, e);
                }
            }
            warn!("任务处理器已停止");
        });
    }

    /// 处理单个任务
    async fn process_task(&self, info_hash: &str) -> Result<()> {
        info!("开始处理任务: info_hash={}", info_hash);
        let task = match self.tasks.get_by_info_hash(info_hash).await? {
            Some(task) => {
                if task.download_status != DownloadStatus::Pending {
                    return Ok(());
                }
                task
            }
            None => {
                anyhow::bail!("任务不存在: info_hash={}", info_hash);
            }
        };

        let dir_cid = self
            .get_or_create_dir_cid(&PathBuf::from(&task.dir))
            .await?;
        let magnet = Self::create_magnet_link(&info_hash);

        match self.pan115.add_offline_task(&[&magnet], &dir_cid).await {
            Ok(_) => {
                info!("成功添加下载任务到网盘: {}", info_hash);
                self.tasks
                    .update_task_status(&info_hash, DownloadStatus::Downloading, None)
                    .await?;
            }
            Err(e) => {
                match e {
                    Pan115Error::OfflineTaskExisted => {
                        warn!("任务已在网盘中存在: {}", info_hash);
                        self.tasks
                            .update_task_status(&info_hash, DownloadStatus::Downloading, None)
                            .await?;
                    }
                    Pan115Error::NotLogin
                    | Pan115Error::OfflineInvalidLink
                    | Pan115Error::OfflineNoTimes => {
                        error!("添加离线下载任务失败: {} - {}", info_hash, e);
                        self.tasks
                            .update_task_status(
                                &info_hash,
                                DownloadStatus::Failed,
                                Some(e.to_string()),
                            )
                            .await?;
                    }
                    _ => {
                        warn!("添加离线下载任务失败: {} - {}, 将重试", info_hash, e);
                        let retry_count = task.retry_count + 1;
                        let next_retry_at = self.config.calculate_next_retry(retry_count);
                        self.tasks
                            .update_task_retry_status(
                                &info_hash,
                                retry_count,
                                next_retry_at,
                                Some(e.to_string()),
                            )
                            .await?;
                    }
                };
            }
        }

        Ok(())
    }
}

// 重试机制相关方法
impl Pan115Downloader {
    /// 启动重试处理器
    fn spawn_retry_processor(&self) {
        info!("启动重试处理器");
        let svc = self.clone();
        let interval = self.config.retry_processor_interval;

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                if let Err(e) = svc.process_retry().await {
                    error!("处理重试队列失败: {}", e);
                }
            }
        });
    }

    /// 处理重试队列中的任务
    async fn process_retry(&self) -> Result<()> {
        let now = Local::now().naive_utc();
        let mut tasks = self
            .tasks
            .list_by_statues(&vec![DownloadStatus::Retrying])
            .await?;
        for task in tasks.as_mut_slice() {
            if task.retry_count >= self.config.max_retry_count {
                warn!(
                    "任务重试次数超过上限: info_hash={}, retry_count={}",
                    task.info_hash, task.retry_count
                );
                self.tasks
                    .update_task_status(
                        &task.info_hash,
                        DownloadStatus::Failed,
                        Some("重试次数超过上限".to_string()),
                    )
                    .await?;
                continue;
            }

            if now < task.next_retry_at {
                continue;
            }

            info!(
                "开始重试任务: info_hash={}, retry_count={}",
                task.info_hash, task.retry_count
            );

            // 重试
            task.download_status = DownloadStatus::Pending;
            self.send_to_queue(task.info_hash.clone()).await?;
        }

        Ok(())
    }
}

// 路径缓存相关方法
impl Pan115Downloader {
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

// 任务恢复相关方法
impl Pan115Downloader {
    /// 恢复未处理完的任务
    async fn recover_pending_tasks(&self, sender: mpsc::Sender<String>) -> Result<()> {
        info!("开始恢复未处理的下载任务");

        let pending_tasks = self
            .tasks
            .list_by_statues(&vec![DownloadStatus::Pending])
            .await?;

        info!("找到 {} 个未处理的任务", pending_tasks.len());
        for task in pending_tasks {
            if let Err(e) = sender.send(task.info_hash.clone()).await {
                error!("恢复任务到队列失败: {} - {}", task.info_hash, e);
            } else {
                info!("成功恢复任务: info_hash={}", task.info_hash);
            }
        }

        info!("完成恢复未处理的下载任务");
        Ok(())
    }
}

// 状态同步相关方法
impl Pan115Downloader {
    /// 启动状态同步器
    fn spawn_status_syncer(&self) {
        info!("启动状态同步器");
        let svc = self.clone();
        let interval = self.config.sync_interval;

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                if let Err(e) = svc.sync_remote_task_status().await {
                    error!("同步下载状态失败: {}", e);
                }
            }
        });
    }

    /// 同步远程任务状态
    async fn sync_remote_task_status(&self) -> Result<()> {
        debug!("开始同步下载状态");

        let local_tasks = self
            .tasks
            .list_by_statues(&vec![DownloadStatus::Downloading, DownloadStatus::Pending])
            .await?;

        if local_tasks.is_empty() {
            debug!("没有需要同步的下载任务");
            return Ok(());
        }

        info!("开始同步 {} 个任务的状态", local_tasks.len());
        let target_info_hashes: Vec<String> = local_tasks
            .iter()
            .map(|task| task.info_hash.clone())
            .collect();

        let remote_tasks = self.fetch_remote_tasks(&target_info_hashes).await?;
        let remote_task_map: HashMap<String, &OfflineTask> = remote_tasks
            .iter()
            .map(|task| (task.info_hash.clone(), task))
            .collect();

        for local_task in local_tasks {
            let info_hash = local_task.info_hash.clone();

            let (status, err_msg) = if let Some(remote_task) = remote_task_map.get(&info_hash) {
                debug!("发现远程任务: info_hash={}", info_hash);
                (Self::map_task_status(remote_task.status()), None)
            } else {
                warn!("任务在网盘中不存在: {}", info_hash);
                (
                    DownloadStatus::Failed,
                    Some("任务在网盘中不存在".to_string()),
                )
            };

            if status != local_task.download_status {
                info!(
                    "更新任务状态: info_hash={}, old_status={:?}, new_status={:?}, err_msg={:?}",
                    info_hash, local_task.download_status, status, err_msg
                );
                self.tasks
                    .update_task_status(&info_hash, status, err_msg)
                    .await?;
            }
        }

        debug!("同步下载状态完成");
        Ok(())
    }

    /// 获取远程任务列表
    async fn fetch_remote_tasks(&self, target_info_hashes: &[String]) -> Result<Vec<OfflineTask>> {
        let mut page = 0;
        let mut remote_tasks = Vec::new();
        let target_hashes: HashSet<&String> = target_info_hashes.iter().collect();

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
            remote_tasks.extend(filtered_tasks);

            if remote_tasks.len() >= target_info_hashes.len() || resp.page_count == resp.page {
                break;
            }

            page = resp.page + 1;
        }

        Ok(remote_tasks)
    }

    /// 映射任务状态
    fn map_task_status(status: OfflineTaskStatus) -> DownloadStatus {
        match status {
            OfflineTaskStatus::Pending => DownloadStatus::Pending,
            OfflineTaskStatus::Downloading => DownloadStatus::Downloading,
            OfflineTaskStatus::Completed => DownloadStatus::Completed,
            OfflineTaskStatus::Failed | OfflineTaskStatus::Unknow => DownloadStatus::Failed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_from_env() -> Result<()> {
        dotenv::dotenv().ok();
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_target(true)
            .init();
        let mut downloader = Pan115Downloader::new_from_env().await?;
        assert_eq!(downloader.name(), "pan_115");
        downloader.spawn().await?;
        downloader
            .add_task(
                "cf778b1c9b25ae87b5629e405b290df602aa9036",
                PathBuf::from("/test"),
            )
            .await?;

        tokio::time::sleep(Duration::from_secs(120)).await;
        Ok(())
    }
}
