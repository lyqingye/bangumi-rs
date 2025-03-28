use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use crate::{
    DownloadInfo, Downloader, DownloaderInfo, FileInfo, RemoteTaskStatus, Store,
    ThirdPartyDownloader, Tid,
    config::Config,
    dlrs::{Dlrs, assigned_dlr},
    errors::{Error, Result},
    metrics,
    resource::Resource,
    stm::{Context, Event, TaskDL},
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
                    actor.sync_all().await;
                }
            });
        }

        Ok(())
    }

    pub async fn run_loop(&self, mut tx_rx: mpsc::UnboundedReceiver<Tx>) {
        let mut ctx = Context::uninit(self.dlrs().best());
        let mut stm = TaskDL::new(&**self.store, &self.dlrs, &mut ctx, &self.notify_tx).await;
        let dlrs = Dlrs::from(&self.dlrs);

        while let Some(tx) = tx_rx.recv().await {
            if let Event::Shutdown(tx) = tx.1 {
                let _ = tx.send(());
                break;
            }

            self.execute(&mut stm, tx, &dlrs)
                .await
                .inspect_err(|e| {
                    error!("处理事件失败: {}", e);
                })
                .ok();
        }
    }

    async fn execute(
        &self,
        stm: &mut InitializedStateMachine<TaskDL<'_>>,
        tx: Tx,
        dlrs: &Dlrs<'_>,
    ) -> Result<()> {
        let (info_hash, mut event) = tx;

        let mut init = true;
        loop {
            let task = self
                .store
                .list_by_hashes(&[info_hash.clone()])
                .await?
                .first()
                .cloned()
                .ok_or(Error::TaskNotFound(info_hash.clone()))?;

            let tdl = dlrs.must_take(&task.downloader)?;

            let mut ctx = Context {
                tid: Tid::from(task.tid()),
                info_hash: &task.info_hash,
                task: &task,
                tdl,
                next_event: None,
            };

            // 这里是为了复用状态机，避免状态机在处理事件时，状态发生变化
            // 所以需要先初始化状态机
            if init {
                stm.handle_with_context(&Event::Init(task.download_status.clone()), &mut ctx)
                    .await;
                init = false;
            }

            stm.handle_with_context(&event, &mut ctx).await;

            if let Some(next) = ctx.next_event {
                event = next;
            } else {
                break;
            }
        }

        Ok(())
    }

    async fn recover(&self) -> Result<()> {
        let pending = self
            .store
            .list_by_status(&[DownloadStatus::Pending])
            .await?;

        for task in pending {
            let resource = self
                .store
                .load_resource(&task.info_hash)
                .await?
                .ok_or(Error::ResourceNotFound(task.info_hash.to_string()))?;

            self.tx
                .send((task.info_hash.clone(), Event::Start(resource)))
                .inspect_err(|e| {
                    error!("恢复任务到队列失败: {} - {}", task.info_hash, e);
                })
                .ok();
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
            self.retry(&task.info_hash).await?;
        }
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx.send((String::new(), Event::Shutdown(tx)))?;
        let _ = tokio::time::timeout(Duration::from_secs(60), rx)
            .await
            .map_err(|_| Error::ShutdownTimeout)?;
        Ok(())
    }

    pub fn dlrs(&self) -> Dlrs<'_> {
        Dlrs::from(&self.dlrs)
    }
}

/// 同步所有下载器任务状态
impl Actor {
    pub async fn sync_all(&self) {
        let tstats = [
            DownloadStatus::Downloading,
            DownloadStatus::Pending,
            DownloadStatus::Paused,
        ];

        let tasks = match self.store.list_by_status(&tstats).await {
            Ok(tasks) => tasks,
            Err(e) => {
                error!("无法获取需要同步的任务状态: {}", e);
                return;
            }
        };

        if tasks.is_empty() {
            return;
        }

        info!("需要同步状态的任务数量: {}", tasks.len());

        // 根据下载器分组
        let mut dlr_with_tasks = HashMap::new();
        for task in tasks.iter() {
            let dlr = assigned_dlr(&task.downloader);
            dlr_with_tasks
                .entry(dlr)
                .or_insert_with(Vec::new)
                .push(task);
        }

        // 同步每个下载器
        for (dlr, tasks) in dlr_with_tasks.iter() {
            self.sync_single(dlr, tasks)
                .await
                .inspect_err(|e| {
                    error!("同步下载器:({}) 任务状态失败: {}", dlr, e);
                })
                .ok();
        }
    }

    async fn sync_single(&self, dlr_name: &str, ltasks: &[&Model]) -> Result<()> {
        let dlr = self.dlrs().must_take(dlr_name)?;
        if ltasks.is_empty() {
            return Ok(());
        }

        let tids: Vec<Tid> = ltasks.iter().map(|t| Tid::from(t.tid())).collect();
        let rtasks = dlr.list_tasks(&tids).await?;

        for ltask in ltasks {
            let ih = ltask.info_hash.clone();
            let tid = Tid::from(ltask.tid());

            self.sync_task(dlr, &ltask, &rtasks, &tid, &ih)?;
        }

        debug!("同步远程任务状态完成");
        Ok(())
    }

    fn sync_task(
        &self,
        dlr: &dyn ThirdPartyDownloader,
        ltask: &Model,
        rtasks: &HashMap<Tid, RemoteTaskStatus>,
        tid: &Tid,
        ih: &str,
    ) -> Result<()> {
        let (st, err_msg, res) = self.detect_task_status(ltask, rtasks, tid, ih);

        if st != ltask.download_status {
            self.handle_status_change(ih, &ltask.download_status, &st, err_msg, res)?;
        }

        self.chk_task_timeout(dlr, ltask, ih)
    }

    fn detect_task_status(
        &self,
        ltask: &Model,
        rtasks: &HashMap<Tid, RemoteTaskStatus>,
        tid: &Tid,
        ih: &str,
    ) -> (DownloadStatus, Option<String>, Option<String>) {
        if let Some(rtask) = rtasks.get(tid) {
            debug!("发现远程任务: info_hash={}", ih);
            (
                rtask.status.clone(),
                rtask.err_msg.clone(),
                rtask.result.clone(),
            )
        } else if ltask.download_status == DownloadStatus::Pending {
            // 本地任务还没被处理，可能在队列中排队
            (DownloadStatus::Pending, None, None)
        } else {
            // 这里的场景是，本地任务已经被处理，但远程任务不存在

            warn!("任务在下载器中不存在: {}", ih);
            (
                DownloadStatus::Pending,
                Some("任务在下载器中不存在".to_string()),
                None,
            )
        }
    }

    fn handle_status_change(
        &self,
        ih: &str,
        old_st: &DownloadStatus,
        new_st: &DownloadStatus,
        err_msg: Option<String>,
        res: Option<String>,
    ) -> Result<()> {
        match new_st {
            DownloadStatus::Completed => {
                self.tx.send((ih.to_string(), Event::Completed(res)))?;
            }
            DownloadStatus::Cancelled => {
                self.tx.send((ih.to_string(), Event::Cancel))?;
            }
            DownloadStatus::Failed => {
                let err = err_msg.unwrap_or_default();
                self.tx
                    .send((ih.to_string(), Event::Failed(ih.to_string(), err)))?;
            }
            DownloadStatus::Paused | DownloadStatus::Downloading => {
                info!("远程任务被手动暂停或恢复: info_hash={}", ih);
                self.tx
                    .send((ih.to_string(), Event::Synced(new_st.clone())))?;
            }
            _ => {
                warn!(
                    "未处理的任务状态: info_hash={}, local_status={:?}, remote_status={:?}, err_msg={:?}",
                    ih, old_st, new_st, err_msg
                );
            }
        }

        Ok(())
    }

    fn chk_task_timeout(
        &self,
        dlr: &dyn ThirdPartyDownloader,
        task: &Model,
        ih: &str,
    ) -> Result<()> {
        if !matches!(
            task.download_status,
            DownloadStatus::Pending | DownloadStatus::Downloading | DownloadStatus::Paused
        ) {
            return Ok(());
        }

        let now = Local::now().naive_utc();
        let elapse = now - task.updated_at;

        if elapse > dlr.config().download_timeout {
            self.tx.send((
                ih.to_string(),
                Event::Failed(ih.to_string(), "下载超时".to_string()),
            ))?;
        }

        Ok(())
    }
}

#[async_trait]
impl Downloader for Actor {
    async fn add_task(
        &self,
        resource: Resource,
        dir: PathBuf,
        dlr_name: Option<String>,
        allow_fallback: bool,
    ) -> Result<()> {
        let info_hash = resource.info_hash();

        let dlr = if let Some(name) = dlr_name {
            self.dlrs().must_take(&name)?
        } else {
            self.dlrs().best()
        };

        self.store
            .create(&resource, dir, dlr.name().to_string(), allow_fallback)
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
        let tasks = self
            .store
            .list_by_status(&[
                DownloadStatus::Downloading,
                DownloadStatus::Pending,
                DownloadStatus::Retrying,
            ])
            .await;
        if let Ok(tasks) = tasks {
            metrics::Metrics {
                num_of_tasks: tasks.len(),
            }
        } else {
            metrics::Metrics { num_of_tasks: 0 }
        }
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
        debug!("列出文件: info_hash={}", info_hash);

        let task = self
            .store
            .list_by_hashes(&[info_hash.to_string()])
            .await?
            .first()
            .cloned()
            .ok_or_else(|| Error::TaskNotFound(info_hash.to_string()))?;

        let tid = Tid::from(task.tid());
        let dlr = self.dlrs().must_take(&task.downloader)?;
        let mut result = dlr.list_files(&tid, task.context.clone()).await?;

        for file in result.iter_mut() {
            file.file_id = format!("{}-{}", dlr.name(), file.file_id);
        }

        Ok(result)
    }

    async fn download_file(&self, file_id: &str, ua: &str) -> Result<DownloadInfo> {
        debug!("下载文件: file_id={}, ua={}", file_id, ua);

        let (dlr_name, file_id) = file_id
            .split_once('-')
            .ok_or_else(|| Error::InvalidFileId(file_id.to_string()))?;

        let dlr = self.dlrs().must_take(dlr_name)?;
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
        self.dlrs().best().recommended_resource_type()
    }

    fn dlrs(&self) -> Vec<DownloaderInfo> {
        self.dlrs().info()
    }

    fn take_dlr(&self, dlr: &str) -> Option<&dyn ThirdPartyDownloader> {
        self.dlrs().take(dlr)
    }
}
