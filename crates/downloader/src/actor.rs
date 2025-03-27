use std::{path::PathBuf, sync::Arc, time::Duration};

use crate::{
    DownloadInfo, Downloader, DownloaderInfo, FileInfo, Store, ThirdPartyDownloader, Tid,
    config::Config,
    errors::{Error, Result},
    metrics,
    resource::Resource,
    stm::{Context, Event, TaskStm},
};
use async_trait::async_trait;
use chrono::Local;
use model::{
    sea_orm_active_enums::{DownloadStatus, ResourceType},
    torrent_download_tasks::Model,
};
use statig::awaitable::InitializedStateMachine;
use tokio::sync::{broadcast, mpsc, oneshot};
use tracing::{debug, error, info, warn};

pub type Tx = (String, Event);

#[derive(Clone)]
pub struct Actor {
    tx: mpsc::UnboundedSender<Tx>,
    store: Arc<Box<dyn Store>>,
    dlrs: Vec<Arc<Box<dyn ThirdPartyDownloader>>>,
    notify_tx: broadcast::Sender<crate::Event>,
    config: Config,
}

impl Actor {
    pub fn new(
        store: Box<dyn Store>,
        config: Config,
        dlrs: Vec<Arc<Box<dyn ThirdPartyDownloader>>>,
    ) -> Result<Self> {
        let (notify_tx, _) = broadcast::channel(config.event_queue_size);
        Ok(Self {
            tx: mpsc::unbounded_channel().0,
            store: Arc::new(store),
            config,
            notify_tx,
            dlrs,
        })
    }

    pub async fn spawn(&mut self) -> Result<()> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.tx = tx;

        {
            let actor = self.clone();
            tokio::spawn(async move {
                actor.run_loop(rx).await;
            });
        }

        self.recover().await?;

        {
            let actor = self.clone();
            tokio::spawn(async move {
                let mut ticker = actor.config.retry_tick();
                loop {
                    ticker.tick().await;

                    actor
                        .auto_retry()
                        .await
                        .inspect_err(|e| {
                            error!("处理重试队列失败: {}", e);
                        })
                        .ok();
                }
            });
        }

        {
            let actor = self.clone();
            tokio::spawn(async move {
                let mut ticker = actor.config.sync_tick();
                loop {
                    ticker.tick().await;
                    actor
                        .sync_all()
                        .await
                        .inspect_err(|e| {
                            error!("同步远程任务状态失败: {}", e);
                        })
                        .ok();
                }
            });
        }

        Ok(())
    }

    pub async fn run_loop(&self, mut tx_rx: mpsc::UnboundedReceiver<Tx>) {
        let mut ctx = Context::uninit(self.best_dlr());
        let mut stm = TaskStm::new(self.store.clone(), self.dlrs.clone(), &mut ctx).await;

        while let Some(tx) = tx_rx.recv().await {
            if let Event::Shutdown(shutdown_tx) = tx.1 {
                let _ = shutdown_tx.send(());
                break;
            }

            match self.execute(&mut stm, tx).await {
                Ok(_) => {}
                Err(e) => {
                    error!("处理事件失败: {}", e);
                }
            }
        }
    }

    async fn execute(&self, stm: &mut InitializedStateMachine<TaskStm>, tx: Tx) -> Result<()> {
        let (info_hash, mut event) = tx;

        loop {
            let task = self
                .store
                .list_by_hashes(&[info_hash.clone()])
                .await?
                .first()
                .cloned()
                .ok_or(Error::TaskNotFound(info_hash.clone()))?;

            let tdl = self.take_dlr(&task.downloader)?;

            let mut ctx = Context {
                tid: Tid::from(task.tid()),
                info_hash: &task.info_hash,
                task: &task,
                tdl,
                next_event: None,
            };
            stm.handle_with_context(&event, &mut ctx).await;
            if ctx.next_event.is_none() {
                break;
            }
            event = ctx.next_event.unwrap();
        }

        Ok(())
    }

    async fn recover(&self) -> Result<()> {
        let pending_tasks = self
            .store
            .list_by_status(&[DownloadStatus::Pending])
            .await?;

        for task in pending_tasks {
            let resource = self
                .store
                .load_resource(&task.info_hash)
                .await?
                .ok_or(Error::ResourceNotFound(task.info_hash.to_string()))?;
            if let Err(e) = self
                .tx
                .send((task.info_hash.clone(), Event::Start(resource)))
            {
                error!("恢复任务到队列失败: {} - {}", task.info_hash, e);
            } else {
                info!("成功恢复任务: info_hash={}", task.info_hash);
            }
        }
        Ok(())
    }

    async fn auto_retry(&self) -> Result<()> {
        let now = Local::now().naive_utc();

        let tasks = self
            .store
            .list_by_status(&[DownloadStatus::Retrying])
            .await?;

        if tasks.is_empty() {
            return Ok(());
        }
        for task in tasks.iter() {
            if now < task.next_retry_at {
                continue;
            }
            // 重试
            self.retry(&task.info_hash).await?;
        }
        Ok(())
    }

    pub async fn sync_all(&self) -> Result<()> {
        for dlr in self.dlrs.iter() {
            self.sync_single(&***dlr).await?;
        }
        Ok(())
    }

    pub async fn sync_single(&self, downloader: &dyn ThirdPartyDownloader) -> Result<()> {
        let local = self
            .store
            .list_by_dlr_and_status(
                downloader.name(),
                &[
                    DownloadStatus::Downloading,
                    DownloadStatus::Pending,
                    DownloadStatus::Paused,
                ],
            )
            .await?;

        if local.is_empty() {
            return Ok(());
        }

        info!("需要同步的下载任务数量: {}", local.len());

        let tids: Vec<Tid> = local.iter().map(|task| Tid::from(task.tid())).collect();

        let remote = downloader.list_tasks(&tids).await?;

        for local_task in local {
            let info_hash = local_task.info_hash.clone();
            let tid = Tid::from(local_task.tid());

            let (status, err_msg, result) = if let Some(remote_task) = remote.get(&tid) {
                debug!("发现远程任务: info_hash={}", info_hash);
                (
                    remote_task.status.clone(),
                    remote_task.err_msg.clone(),
                    remote_task.result.clone(),
                )
            } else if local_task.download_status == DownloadStatus::Pending {
                // NOTE: 说明本地任务还没被处理，可能还在队列中排队，所以在这里忽略
                (DownloadStatus::Pending, None, None)
            } else {
                warn!("任务在下载器中不存在: {}", info_hash);
                (
                    DownloadStatus::Pending,
                    Some("任务在下载器中不存在".to_string()),
                    None,
                )
            };

            if status.clone() != local_task.download_status {
                info!(
                    "远程任务状态更新: info_hash={}, old_status={:?}, new_status={:?}, err_msg={:?}",
                    info_hash, local_task.download_status, status, err_msg
                );

                match status {
                    DownloadStatus::Completed => {
                        self.tx
                            .send((info_hash.clone(), Event::Completed(result)))?;
                    }

                    DownloadStatus::Cancelled => {
                        self.tx.send((info_hash.clone(), Event::Cancel))?;
                    }

                    DownloadStatus::Failed => {
                        self.tx.send((
                            info_hash.clone(),
                            Event::Failed(info_hash.clone(), err_msg.unwrap_or_default()),
                        ))?;
                    }

                    // 本地状态和远程任务状态不一致，例如本地状态是Downloading, 然后远程任务被用户手动暂停了
                    // 例如本地状态是Paused, 然后远程任务被用户手动恢复了
                    // 所以此时需要更新本地任务状态
                    DownloadStatus::Paused | DownloadStatus::Downloading => {
                        info!("远程任务被手动暂停或恢复: info_hash={}", info_hash);
                        self.tx.send((info_hash.clone(), Event::Synced(status)))?;
                    }

                    _ => {
                        warn!(
                            "未处理的任务状态: info_hash={}, local_status={:?}, remote_status={:?}, err_msg={:?}",
                            info_hash, local_task.download_status, status, err_msg
                        );
                    }
                }
            } else if matches!(
                local_task.download_status,
                DownloadStatus::Pending | DownloadStatus::Downloading | DownloadStatus::Paused
            ) {
                let now = Local::now().naive_utc();
                let elapsed = now - local_task.updated_at;
                if elapsed > downloader.config().download_timeout {
                    warn!("下载超时: info_hash={}", info_hash);
                    self.tx.send((
                        info_hash.clone(),
                        Event::Failed(info_hash.clone(), "下载超时".to_string()),
                    ))?;
                }
            }
        }
        info!("同步远程任务状态完成");

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        if let Err(e) = self.tx.send((String::new(), Event::Shutdown(tx))) {
            error!("发送关闭事件失败: {}", e);
        }
        let _ = tokio::time::timeout(Duration::from_secs(60), rx)
            .await
            .map_err(|_| Error::ShutdownTimeout)?;
        Ok(())
    }
}

impl Actor {
    fn take_dlr(&self, assigned_downloader: &str) -> Result<&dyn ThirdPartyDownloader> {
        let latest = assigned_downloader.split(',').last().unwrap();
        let downloader = &***self
            .dlrs
            .iter()
            .find(|d| d.name() == latest)
            .ok_or(Error::DownloaderNotFound(latest.to_string()))?;
        Ok(downloader)
    }
}

#[async_trait]
impl Downloader for Actor {
    async fn add_task(
        &self,
        resource: Resource,
        dir: PathBuf,
        downloader: Option<String>,
        allow_fallback: bool,
    ) -> Result<()> {
        let info_hash = resource.info_hash();
        let downloader = if let Some(downloader_name) = downloader {
            self.take_dlr(&downloader_name)?
        } else {
            self.best_dlr()
        };
        self.store
            .create(
                &resource,
                dir,
                downloader.name().to_string(),
                allow_fallback,
            )
            .await?;
        self.tx
            .send((info_hash.to_owned(), Event::Start(resource)))?;
        Ok(())
    }

    async fn list_tasks(&self, info_hashes: &[String]) -> Result<Vec<Model>> {
        let tasks = self.store.list_by_hashes(info_hashes).await?;
        Ok(tasks)
    }

    async fn cancel_task(&self, info_hash: &str) -> Result<()> {
        self.tx.send((info_hash.to_string(), Event::Cancel))?;
        Ok(())
    }

    async fn metrics(&self) -> metrics::Metrics {
        metrics::Metrics { num_of_tasks: 1 }
    }

    async fn subscribe(&self) -> broadcast::Receiver<crate::Event> {
        self.notify_tx.subscribe()
    }

    async fn retry(&self, info_hash: &str) -> Result<()> {
        self.tx.send((info_hash.to_string(), Event::Retry))?;
        Ok(())
    }

    async fn remove_task(&self, info_hash: &str, remove_files: bool) -> Result<()> {
        self.tx
            .send((info_hash.to_string(), Event::Remove(remove_files)))?;
        Ok(())
    }

    async fn list_files(&self, info_hash: &str) -> Result<Vec<FileInfo>> {
        info!("列出文件: info_hash={}", info_hash);
        let task = self
            .store
            .list_by_hashes(&[info_hash.to_string()])
            .await?
            .first()
            .cloned()
            .ok_or_else(|| Error::TaskNotFound(info_hash.to_string()))?;
        let tid = Tid::from(task.tid());
        let downloader = self.take_dlr(&task.downloader)?;
        let mut result = downloader.list_files(&tid, task.context.clone()).await?;
        for file in result.iter_mut() {
            file.file_id = format!("{}-{}", downloader.name(), file.file_id);
        }
        Ok(result)
    }

    async fn download_file(&self, file_id: &str, ua: &str) -> Result<DownloadInfo> {
        info!("下载文件: file_id={}, ua={}", file_id, ua);
        let (dlr_name, file_id) = file_id
            .split_once('-')
            .ok_or_else(|| Error::InvalidFileId(file_id.to_string()))?;
        let dlr = self.take_dlr(dlr_name)?;
        let result = dlr.dl_file(file_id, ua).await;

        if let Err(ref e) = result {
            warn!("下载文件失败: file_id={}, 错误: {}", file_id, e);
        } else {
            debug!("下载文件成功: file_id={}", file_id);
        }

        result
    }

    async fn pause_task(&self, info_hash: &str) -> Result<()> {
        self.tx.send((info_hash.to_string(), Event::Pause))?;
        Ok(())
    }

    async fn resume_task(&self, info_hash: &str) -> Result<()> {
        self.tx.send((info_hash.to_string(), Event::Resume))?;
        Ok(())
    }

    fn supports_resource_type(&self, resource_type: ResourceType) -> bool {
        for dlr in self.dlrs.iter() {
            if dlr.supports_resource_type(resource_type.clone()) {
                return true;
            }
        }
        false
    }

    fn recommended_resource_type(&self) -> ResourceType {
        self.best_dlr().recommended_resource_type()
    }

    fn get_dlr(&self, dlr: &str) -> Option<&dyn ThirdPartyDownloader> {
        self.dlrs.iter().find(|d| d.name() == dlr).map(|d| &***d)
    }

    fn dlrs(&self) -> Vec<DownloaderInfo> {
        let mut dlrs: Vec<DownloaderInfo> = self
            .dlrs
            .iter()
            .map(|d| DownloaderInfo {
                name: d.name().to_string(),
                priority: d.config().priority,
            })
            .collect();

        dlrs.sort_by(|a, b| b.priority.cmp(&a.priority));

        dlrs
    }
}

impl Actor {
    fn best_dlr(&self) -> &dyn ThirdPartyDownloader {
        &***self
            .dlrs
            .iter()
            .max_by_key(|d| d.config().priority)
            .unwrap()
    }
}
