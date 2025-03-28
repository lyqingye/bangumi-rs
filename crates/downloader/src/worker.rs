use std::{path::PathBuf, sync::Arc};

use crate::errors::{Error, Result};
use async_trait::async_trait;
use chrono::NaiveDateTime;
use model::{sea_orm_active_enums::DownloadStatus, torrent_download_tasks::Model};
use tokio::sync::{broadcast, mpsc, oneshot};
use tracing::{debug, error, info, warn};

use crate::{
    DownloadInfo, Downloader, DownloaderInfo, Event, FileInfo, Resource, ResourceType, Store,
    ThirdPartyDownloader, Tid, config::Config, db::Db, metrics,
    thirdparty::pan_115_impl::Pan115DownloaderImpl,
};

type State = DownloadStatus;

pub enum Tx {
    // 外部事件
    StartTask(Resource),
    CancelTask(String),
    RemoveTask((String, bool)),
    RetryTask(String),
    PauseTask(String),
    ResumeTask(String),

    // 内部事件
    AutoFallback(String),
    AutoRetry(String),
    TaskFailed(String, String),
    TaskCompleted(String, Option<String>),
    TaskStatusUpdated((String, DownloadStatus)),

    Shutdown(oneshot::Sender<()>),
}

impl Tx {
    fn debug_name(&self) -> &'static str {
        match self {
            Self::StartTask(_) => "StartTask",
            Self::CancelTask(_) => "CancelTask",
            Self::RetryTask(_) => "RetryTask",
            Self::AutoRetry(_) => "AutoRetry",
            Self::AutoFallback(_) => "AutoFallback",
            Self::TaskFailed(_, _) => "TaskFailed",
            Self::TaskCompleted(_, _) => "TaskCompleted",
            Self::RemoveTask((_, _)) => "RemoveTask",
            Self::PauseTask(_) => "PauseTask",
            Self::ResumeTask(_) => "ResumeTask",
            Self::Shutdown(_) => "Shutdown",
            Self::TaskStatusUpdated(_) => "TaskStatusUpdated",
        }
    }

    async fn get_ref_task(&self, tasks: &dyn Store) -> Result<Model> {
        let info_hash = match self {
            Self::StartTask(resource) => resource.info_hash(),
            Self::CancelTask(info_hash) => info_hash,
            Self::RetryTask(info_hash) => info_hash,
            Self::AutoRetry(info_hash) => info_hash,
            Self::AutoFallback(info_hash) => info_hash,
            Self::TaskFailed(info_hash, _) => info_hash,
            Self::TaskCompleted(info_hash, _) => info_hash,
            Self::RemoveTask((info_hash, _)) => info_hash,
            Self::PauseTask(info_hash) => info_hash,
            Self::ResumeTask(info_hash) => info_hash,
            Self::Shutdown(_) => unreachable!(),
            Self::TaskStatusUpdated((info_hash, _)) => info_hash,
        };
        tasks
            .list_by_hashes(&[info_hash.to_string()])
            .await?
            .first()
            .cloned()
            .ok_or_else(|| Error::TaskNotFound(info_hash.to_string()))
    }
}

pub struct Context<'a> {
    tid: Tid,
    ref_task: Model,
    downloader: &'a dyn ThirdPartyDownloader,
}

impl Context<'_> {
    pub fn tid(&self) -> &Tid {
        &self.tid
    }

    pub fn info_hash(&self) -> &str {
        &self.ref_task.info_hash
    }
}

#[derive(Clone)]
pub struct Worker {
    event_queue: mpsc::UnboundedSender<Tx>,
    pub(crate) store: Arc<Box<dyn Store>>,
    pub(crate) config: Config,
    notify_tx: broadcast::Sender<Event>,
    pub(crate) downloaders: Vec<Arc<Box<dyn ThirdPartyDownloader>>>,
}

impl Worker {
    pub fn new_with_conn(
        store: Box<dyn Store>,
        config: Config,
        downloaders: Vec<Arc<Box<dyn ThirdPartyDownloader>>>,
    ) -> Result<Self> {
        let (notify_tx, _) = broadcast::channel(config.event_queue_size);
        Ok(Self {
            event_queue: mpsc::unbounded_channel().0,
            store: Arc::new(store),
            config,
            notify_tx,
            downloaders,
        })
    }
}

impl Worker {
    pub async fn spawn(&mut self) -> Result<()> {
        let (event_queue, event_receiver) = mpsc::unbounded_channel();
        self.event_queue = event_queue;
        // 启动事件循环
        self.spawn_event_loop(event_receiver);
        // 启动同步器
        self.spawn_syncer()?;
        // 恢复未处理的下载任务
        self.recover_pending_tasks().await?;
        // 启动重试处理器
        self.spawn_retry_processor();

        info!("Downloader 已启动，配置: {:?}", self.config);
        Ok(())
    }

    pub(crate) fn send_event(&self, event: Tx) -> Result<()> {
        self.event_queue
            .send(event)
            .map_err(|_| Error::ChannelClosed)?;
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("正在关闭 Downloader...");
        let (tx, rx) = oneshot::channel();
        self.send_event(Tx::Shutdown(tx))?;
        rx.await.map_err(|_| Error::ChannelClosed)?;
        info!("Downloader 已完全关闭");
        Ok(())
    }

    async fn recover_pending_tasks(&self) -> Result<()> {
        info!("开始恢复未处理的下载任务");

        let pending_tasks = self
            .store
            .list_by_status(&[DownloadStatus::Pending])
            .await?;

        info!("找到 {} 个未处理的任务", pending_tasks.len());
        for task in pending_tasks {
            let resource = self
                .store
                .load_resource(&task.info_hash)
                .await?
                .ok_or_else(|| Error::ResourceNotFound(task.info_hash.to_string()))?;
            if let Err(e) = self.send_event(Tx::StartTask(resource)) {
                error!("恢复任务到队列失败: {} - {}", task.info_hash, e);
            } else {
                info!("成功恢复任务: info_hash={}", task.info_hash);
            }
        }

        info!("完成恢复未处理的下载任务");
        Ok(())
    }

    fn spawn_event_loop(&self, mut receiver: mpsc::UnboundedReceiver<Tx>) {
        let worker = self.clone();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                if let Tx::Shutdown(done) = event {
                    let _ = done.send(());
                    info!("Downloader 事件循环已停止");
                    break;
                }
                match worker.handle_event(event).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("处理事件失败: {}", e);
                    }
                }
            }
        });
    }

    async fn handle_event(&self, event: Tx) -> Result<()> {
        let task = event.get_ref_task(&**self.store).await?;
        debug!(
            "处理事件: {}, 任务状态: {:?}",
            event.debug_name(),
            task.download_status
        );

        let mut ctx = Context {
            tid: Tid::from(task.tid()),
            ref_task: task.clone(),
            downloader: self.take_downloader(&task.downloader)?,
        };
        let mut event = Some(event);
        let mut state = Some(task.download_status);
        while let (Some(cur_event), Some(cur_state)) = (event.take(), state.take()) {
            let (next_event, next_state) = self.transition(cur_event, cur_state, &mut ctx).await?;
            // 更新上下文中任务的状态
            if let Some(state) = next_state.as_ref() {
                ctx.ref_task.download_status = state.clone();
            }

            // 更新上下文
            if let Some(event) = next_event.as_ref() {
                ctx.ref_task = event.get_ref_task(&**self.store).await?;
                ctx.downloader = self.take_downloader(&ctx.ref_task.downloader)?;
                ctx.tid = Tid::from(ctx.ref_task.tid().to_string());
            }
            event = next_event;
            state = next_state;
        }
        Ok(())
    }

    // NOTE: 这个方法需要是 pub(crate) 才能在 syncer.rs 中调用
    pub(crate) fn take_downloader_by_name(&self, name: &str) -> Result<&dyn ThirdPartyDownloader> {
        self.downloaders
            .iter()
            .find(|d| d.name() == name)
            .map(|d| &***d)
            .ok_or_else(|| Error::DownloaderNotFound(name.to_string()))
    }
}

/// External Function
impl Worker {
    pub async fn add_task(
        &self,
        resource: Resource,
        dir: PathBuf,
        downloader: Option<String>,
        allow_fallback: bool,
    ) -> Result<()> {
        let info_hash = resource.info_hash();
        info!(
            "添加下载任务: info_hash={:?}, dir={}, downloader={:?}, allow_fallback={}",
            info_hash,
            dir.display(),
            downloader,
            allow_fallback
        );
        let downloader = if let Some(downloader_name) = downloader {
            self.take_downloader_by_name(&downloader_name)?
        } else {
            self.best_downloader()
        };
        self.store
            .create(
                &resource,
                dir,
                downloader.name().to_string(),
                allow_fallback,
            )
            .await?;
        self.send_event(Tx::StartTask(resource))?;
        Ok(())
    }

    pub fn cancel_task(&self, info_hash: &str) -> Result<()> {
        info!("取消下载任务: info_hash={}", info_hash);
        self.send_event(Tx::CancelTask(info_hash.to_string()))?;
        Ok(())
    }

    pub fn remove_task(&self, info_hash: &str, remove_files: bool) -> Result<()> {
        info!(
            "移除下载任务: info_hash={}, remove_files={}",
            info_hash, remove_files
        );
        self.send_event(Tx::RemoveTask((info_hash.to_string(), remove_files)))?;
        Ok(())
    }

    pub fn retry_task(&self, info_hash: &str) -> Result<()> {
        info!("重试下载任务: info_hash={}", info_hash);
        self.send_event(Tx::RetryTask(info_hash.to_string()))?;
        Ok(())
    }

    pub async fn list_files(&self, info_hash: &str) -> Result<Vec<FileInfo>> {
        info!("列出文件: info_hash={}", info_hash);
        let task = self
            .store
            .list_by_hashes(&[info_hash.to_string()])
            .await?
            .first()
            .cloned()
            .ok_or_else(|| Error::TaskNotFound(info_hash.to_string()))?;
        let tid = Tid::from(task.tid());
        let downloader = self.take_downloader(&task.downloader)?;
        let mut result = downloader.list_files(&tid, task.context.clone()).await?;
        for file in result.iter_mut() {
            file.file_id = format!("{}-{}", downloader.name(), file.file_id);
        }
        Ok(result)
    }

    pub async fn download_file(&self, file_id: &str, ua: &str) -> Result<DownloadInfo> {
        info!("下载文件: file_id={}, ua={}", file_id, ua);
        let (downloader_name, file_id) = file_id
            .split_once('-')
            .ok_or_else(|| Error::InvalidFileId(file_id.to_string()))?;
        let downloader = self.take_downloader_by_name(downloader_name)?;
        let result = downloader.dl_file(file_id, ua).await;

        if let Err(ref e) = result {
            warn!("下载文件失败: file_id={}, 错误: {}", file_id, e);
        } else {
            debug!("下载文件成功: file_id={}", file_id);
        }

        result
    }

    pub fn pause_task(&self, info_hash: &str) -> Result<()> {
        info!("暂停下载任务: info_hash={}", info_hash);
        self.send_event(Tx::PauseTask(info_hash.to_string()))?;
        Ok(())
    }

    pub fn resume_task(&self, info_hash: &str) -> Result<()> {
        info!("恢复下载任务: info_hash={}", info_hash);
        self.send_event(Tx::ResumeTask(info_hash.to_string()))?;
        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.notify_tx.subscribe()
    }

    pub async fn metrics(&self) -> metrics::Metrics {
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
}

/// State Transition
impl Worker {
    async fn transition(
        &self,
        event: Tx,
        state: State,
        ctx: &mut Context<'_>,
    ) -> Result<(Option<Tx>, Option<State>)> {
        debug!(
            "状态转换开始: 事件={}, 状态={:?}",
            event.debug_name(),
            state
        );
        match (event, state.clone()) {
            // 开始任务
            (Tx::StartTask(resource), State::Pending) => self.on_start_task(resource, ctx).await,

            // 取消任务
            (Tx::CancelTask(_), State::Downloading | State::Retrying) => {
                self.on_task_cancelled(ctx).await
            }

            // 暂停任务
            (Tx::PauseTask(_), State::Downloading | State::Retrying) => {
                self.on_task_paused(ctx).await
            }

            // 恢复任务
            (Tx::ResumeTask(_), State::Paused) => self.on_task_resumed(ctx).await,

            // 移除任务
            (Tx::RemoveTask((_, _)), _) => self.on_task_removed(ctx).await,

            // 重试任务
            (Tx::RetryTask(_), State::Failed | State::Cancelled) => self.on_task_retry(ctx).await,

            // 自动重试
            (Tx::AutoRetry(_), State::Retrying) => self.on_task_retry(ctx).await,

            // 任务失败
            (Tx::TaskFailed(_, err_msg), State::Downloading | State::Pending) => {
                self.on_task_failed(err_msg, ctx).await
            }

            // 任务完成
            (Tx::TaskCompleted(_, result), State::Downloading) => {
                self.on_task_completed(result, ctx).await
            }

            // 任务状态更新
            (Tx::TaskStatusUpdated((_, status)), _) => {
                self.on_task_status_updated(status, ctx).await
            }

            // 自动回退
            (Tx::AutoFallback(_), State::Failed) => self.on_task_fallback(ctx).await,

            (event, state) => {
                warn!(
                    "无效的状态转换: 事件={}, 状态={:?}",
                    event.debug_name(),
                    state
                );
                Ok((None, None))
            }
        }
    }

    async fn on_start_task(
        &self,
        resource: Resource,
        ctx: &mut Context<'_>,
    ) -> Result<(Option<Tx>, Option<State>)> {
        let info_hash = resource.info_hash().to_string();
        info!(
            "开始处理任务(StartTask): info_hash={} state={:?} dir={}",
            info_hash, ctx.ref_task.download_status, ctx.ref_task.dir
        );

        match ctx
            .downloader
            .add_task(resource, ctx.ref_task.dir.clone().into())
            .await
        {
            Ok((tid, result)) => {
                info!(
                    "处理任务成功(StartTask): info_hash={} state={:?}, 结果: {:?}",
                    info_hash,
                    DownloadStatus::Downloading,
                    result
                );
                self.update_task_status(&info_hash, DownloadStatus::Downloading, None, result)
                    .await?;

                if let Some(tid) = tid {
                    self.store.update_tid(&info_hash, &tid).await?;
                }

                Ok((None, Some(State::Downloading)))
            }

            Err(e) => {
                error!(
                    "处理任务失败(StartTask): info_hash={} state={:?} -> TaskFailed, err_msg={}",
                    info_hash,
                    DownloadStatus::Pending,
                    e
                );
                Ok((
                    Some(Tx::TaskFailed(info_hash, e.to_string())),
                    Some(State::Pending),
                ))
            }
        }
    }

    async fn on_task_failed(
        &self,
        err_msg: String,
        ctx: &mut Context<'_>,
    ) -> Result<(Option<Tx>, Option<State>)> {
        let tid = ctx.tid();
        warn!(
            "处理任务失败(TaskFailed): tid={} state={:?} -> TaskFailed, err_msg={}",
            tid, ctx.ref_task.download_status, err_msg
        );

        debug!("从第三方下载器移除失败任务: tid={}", tid);
        if let Err(e) = ctx.downloader.remove_task(tid, true).await {
            warn!("移除失败任务出错: tid={}, 错误: {}", tid, e);
        }

        if ctx.ref_task.retry_count >= ctx.downloader.config().max_retry_count {
            warn!(
                "任务重试次数已达上限: tid={}, 重试次数={}/{}",
                tid,
                ctx.ref_task.retry_count,
                ctx.downloader.config().max_retry_count
            );
            self.update_task_status(
                ctx.info_hash(),
                DownloadStatus::Failed,
                Some(format!(
                    "重试次数超过上限({}): {}",
                    ctx.downloader.config().max_retry_count,
                    err_msg
                )),
                None,
            )
            .await?;

            if ctx.ref_task.allow_fallback {
                info!("任务允许自动回退，尝试其它下载器: tid={}", tid);
                Ok((
                    Some(Tx::AutoFallback(ctx.info_hash().to_string())),
                    Some(State::Failed),
                ))
            } else {
                info!("任务不允许自动回退，直接失败: tid={}", tid);
                Ok((None, Some(State::Failed)))
            }
        } else {
            let next_retry_at = ctx
                .downloader
                .config()
                .calculate_next_retry(ctx.ref_task.retry_count + 1);

            info!(
                "更新任务重试状态(TaskFailed): tid={}, 重试次数={}/{}, next_retry_at={}",
                tid,
                ctx.ref_task.retry_count + 1,
                ctx.downloader.config().max_retry_count,
                next_retry_at
            );
            self.update_task_retry_status(ctx.info_hash(), next_retry_at, Some(err_msg))
                .await?;
            Ok((
                Some(Tx::AutoRetry(ctx.info_hash().to_string())),
                Some(State::Retrying),
            ))
        }
    }

    async fn on_task_retry(&self, ctx: &mut Context<'_>) -> Result<(Option<Tx>, Option<State>)> {
        let tid = ctx.tid();
        info!(
            "开始重试任务(TaskRetry): tid={} state={:?}, 重试次数={}/{}",
            tid,
            ctx.ref_task.download_status,
            ctx.ref_task.retry_count,
            ctx.downloader.config().max_retry_count
        );
        let resource = self
            .store
            .load_resource(ctx.info_hash())
            .await?
            .ok_or_else(|| Error::ResourceNotFound(ctx.info_hash().to_string()))?;
        // 删除原有任务，然后重新下载
        if let Err(e) = ctx.downloader.remove_task(tid, true).await {
            warn!("移除任务准备重试出错: tid={}, 错误: {}", tid, e);
        }

        self.update_task_status(ctx.info_hash(), DownloadStatus::Pending, None, None)
            .await?;
        Ok((Some(Tx::StartTask(resource)), Some(State::Pending)))
    }

    async fn on_task_cancelled(
        &self,
        ctx: &mut Context<'_>,
    ) -> Result<(Option<Tx>, Option<State>)> {
        let tid = ctx.tid();
        info!(
            "取消任务(TaskCancelled): tid={} state={:?}",
            tid, ctx.ref_task.download_status
        );

        if let Err(e) = ctx.downloader.cancel_task(tid).await {
            warn!("取消任务出错: tid={}, 错误: {}", tid, e);
        }

        self.update_task_status(ctx.info_hash(), DownloadStatus::Cancelled, None, None)
            .await?;
        Ok((None, Some(State::Cancelled)))
    }

    async fn on_task_completed(
        &self,
        result: Option<String>,
        ctx: &mut Context<'_>,
    ) -> Result<(Option<Tx>, Option<State>)> {
        let tid = ctx.tid();
        info!(
            "任务完成(TaskCompleted): tid={} state={:?}, 结果: {:?}",
            tid, ctx.ref_task.download_status, result
        );
        self.update_task_status(ctx.info_hash(), DownloadStatus::Completed, None, result)
            .await?;

        if ctx.downloader.config().delete_task_on_completion {
            if let Err(e) = ctx.downloader.remove_task(tid, false).await {
                warn!("清理下载记录出错: tid={}, 错误: {}", tid, e);
            }
        }

        Ok((None, Some(State::Completed)))
    }

    async fn on_task_removed(&self, ctx: &mut Context<'_>) -> Result<(Option<Tx>, Option<State>)> {
        let tid = ctx.tid();
        info!(
            "移除任务(TaskRemoved): tid={} state={:?}",
            tid, ctx.ref_task.download_status
        );
        if let Err(e) = ctx.downloader.remove_task(tid, true).await {
            warn!("移除任务出错: tid={}, 错误: {}", tid, e);
        }
        self.update_task_status(ctx.info_hash(), DownloadStatus::Cancelled, None, None)
            .await?;
        Ok((None, Some(State::Cancelled)))
    }

    async fn on_task_paused(&self, ctx: &mut Context<'_>) -> Result<(Option<Tx>, Option<State>)> {
        let tid = ctx.tid();
        info!(
            "暂停任务(TaskPaused): tid={} state={:?}",
            tid, ctx.ref_task.download_status
        );

        if let Err(e) = ctx.downloader.pause_task(tid).await {
            warn!("暂停任务出错: tid={}, 错误: {}", tid, e);
            Ok((None, Some(State::Downloading)))
        } else {
            self.update_task_status(ctx.info_hash(), DownloadStatus::Paused, None, None)
                .await?;
            Ok((None, Some(State::Paused)))
        }
    }

    async fn on_task_resumed(&self, ctx: &mut Context<'_>) -> Result<(Option<Tx>, Option<State>)> {
        let tid = ctx.tid();
        info!(
            "恢复任务(TaskResumed): tid={} state={:?}",
            tid, ctx.ref_task.download_status
        );

        if let Err(e) = ctx.downloader.resume_task(tid).await {
            warn!("恢复任务出错: tid={}, 错误: {}", tid, e);
            Ok((None, Some(State::Paused)))
        } else {
            self.update_task_status(ctx.info_hash(), DownloadStatus::Downloading, None, None)
                .await?;
            Ok((None, Some(State::Downloading)))
        }
    }

    async fn on_task_status_updated(
        &self,
        status: DownloadStatus,
        ctx: &mut Context<'_>,
    ) -> Result<(Option<Tx>, Option<State>)> {
        let tid = ctx.tid();
        info!(
            "远程任务状态被手动更新(TaskStatusUpdated): tid={} state={:?}",
            tid, status
        );
        self.update_task_status(ctx.info_hash(), status.clone(), None, None)
            .await?;
        Ok((None, Some(status)))
    }

    async fn on_task_fallback(&self, ctx: &mut Context<'_>) -> Result<(Option<Tx>, Option<State>)> {
        let tid = ctx.tid();
        info!(
            "自动回退任务(AutoFallback): tid={} state={:?}",
            tid, ctx.ref_task.download_status
        );

        // 找到优先级最高的未使用下载器
        let fallback_downloader = self
            .downloaders
            .iter()
            .filter(|d| !ctx.ref_task.downloader.contains(d.name()))
            .max_by_key(|d| d.config().priority)
            .map(|d| d.name());

        match fallback_downloader {
            Some(downloader) => {
                info!(
                    "自动回退任务(AutoFallback): tid={} state={:?}, 使用备用下载器: {}",
                    tid, ctx.ref_task.download_status, downloader
                );
                // 更新任务的下载器
                let mut new_downloader = ctx.ref_task.downloader.to_string();
                new_downloader.push(',');
                new_downloader.push_str(downloader);

                // 更新任务状态
                self.store
                    .assign_dlr(ctx.info_hash(), new_downloader)
                    .await?;

                // 重新启动任务
                let resource = self
                    .store
                    .load_resource(ctx.info_hash())
                    .await?
                    .ok_or_else(|| Error::ResourceNotFound(ctx.info_hash().to_string()))?;

                // 更新任务状态为 Pending
                self.update_task_status(ctx.info_hash(), DownloadStatus::Pending, None, None)
                    .await?;

                Ok((Some(Tx::StartTask(resource)), Some(State::Pending)))
            }
            None => {
                info!("自动回退失败: tid={}, 没有可用的备选下载器", tid);
                // 没有可用的备选下载器，标记为失败
                self.update_task_status(
                    ctx.info_hash(),
                    DownloadStatus::Failed,
                    Some(format!(
                        "没有可用的备选下载器: {}",
                        &ctx.ref_task.err_msg.as_ref().unwrap_or(&"".to_string())
                    )),
                    None,
                )
                .await?;
                Ok((None, Some(State::Failed)))
            }
        }
    }

    fn take_downloader(&self, assigned_downloader: &str) -> Result<&dyn ThirdPartyDownloader> {
        let latest = assigned_downloader
            .split(',')
            .last()
            .unwrap_or(assigned_downloader);
        let downloader = &***self
            .downloaders
            .iter()
            .find(|d| d.name() == latest)
            .ok_or_else(|| Error::DownloaderNotFound(latest.to_string()))?;
        Ok(downloader)
    }

    fn best_downloader(&self) -> &dyn ThirdPartyDownloader {
        &***self
            .downloaders
            .iter()
            .max_by_key(|d| d.config().priority)
            .unwrap()
    }

    async fn update_task_retry_status(
        &self,
        info_hash: &str,
        next_retry_at: NaiveDateTime,
        err_msg: Option<String>,
    ) -> Result<()> {
        self.store
            .update_retry_status(info_hash, next_retry_at, err_msg.clone())
            .await?;

        // 推送事件
        let _ = self.notify_tx.send(Event::TaskUpdated((
            info_hash.to_string(),
            DownloadStatus::Retrying,
            err_msg,
        )));
        Ok(())
    }

    async fn update_task_status(
        &self,
        info_hash: &str,
        status: DownloadStatus,
        err_msg: Option<String>,
        result: Option<String>,
    ) -> Result<()> {
        self.store
            .update_status(info_hash, status.clone(), err_msg.clone(), result.clone())
            .await?;

        let _ = self
            .notify_tx
            .send(Event::TaskUpdated((info_hash.to_string(), status, err_msg)));
        Ok(())
    }
}

/// Implmentation Downloader
#[async_trait]
impl Downloader for Worker {
    async fn add_task(
        &self,
        resource: Resource,
        dir: PathBuf,
        downloader: Option<String>,
        allow_fallback: bool,
    ) -> Result<()> {
        self.add_task(resource, dir, downloader, allow_fallback)
            .await
    }

    async fn list_tasks(&self, info_hashes: &[String]) -> Result<Vec<Model>> {
        self.store.list_by_hashes(info_hashes).await
    }

    async fn download_file(&self, file_id: &str, ua: &str) -> Result<DownloadInfo> {
        self.download_file(file_id, ua).await
    }

    async fn cancel_task(&self, info_hash: &str) -> Result<()> {
        self.cancel_task(info_hash)
    }

    async fn metrics(&self) -> metrics::Metrics {
        self.metrics().await
    }

    async fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.subscribe()
    }

    async fn retry(&self, info_hash: &str) -> Result<()> {
        self.retry_task(info_hash)
    }

    async fn remove_task(&self, info_hash: &str, remove_files: bool) -> Result<()> {
        self.remove_task(info_hash, remove_files)
    }

    async fn list_files(&self, info_hash: &str) -> Result<Vec<FileInfo>> {
        self.list_files(info_hash).await
    }

    async fn pause_task(&self, info_hash: &str) -> Result<()> {
        self.pause_task(info_hash)
    }

    async fn resume_task(&self, info_hash: &str) -> Result<()> {
        self.resume_task(info_hash)
    }

    fn supports_resource_type(&self, resource_type: ResourceType) -> bool {
        for downloader in self.downloaders.iter() {
            if downloader.supports_resource_type(resource_type.clone()) {
                return true;
            }
        }
        false
    }

    fn recommended_resource_type(&self) -> ResourceType {
        self.best_downloader().recommended_resource_type()
    }

    fn take_dlr(&self, downloader: &str) -> Option<&dyn ThirdPartyDownloader> {
        self.downloaders
            .iter()
            .find(|d| d.name() == downloader)
            .map(|d| &***d)
    }

    fn dlrs(&self) -> Vec<DownloaderInfo> {
        let mut downloaders: Vec<DownloaderInfo> = self
            .downloaders
            .iter()
            .map(|d| DownloaderInfo {
                name: d.name().to_string(),
                priority: d.config().priority,
            })
            .collect();

        downloaders.sort_by(|a, b| b.priority.cmp(&a.priority));

        downloaders
    }
}

impl Worker {
    pub async fn new_from_env() -> Result<Self> {
        let db = Db::new_from_env().await?;
        let downloader = Pan115DownloaderImpl::new_from_env()?;
        let config = Config::default();
        Self::new_with_conn(Box::new(db), config, vec![Arc::new(Box::new(downloader))])
    }
}
