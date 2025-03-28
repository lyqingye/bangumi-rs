use crate::dlrs::{Dlrs, assign_dlr};
use crate::errors::{Error, Result};
use crate::{Store, ThirdPartyDownloader, Tid, resource::Resource};
use chrono::NaiveDateTime;
use model::sea_orm_active_enums::DownloadStatus;
use model::torrent_download_tasks::Model;
use statig::awaitable::InitializedStateMachine;
use statig::prelude::*;
use std::sync::Arc;
use tokio::sync::{broadcast, oneshot};
use tracing::{info, warn};

macro_rules! run_action {
    ($action:expr) => {
        match $action.await {
            Ok(response) => response,
            Err(e) => {
                tracing::error!("状态转换出错: {}", e);
                Handled
            }
        }
    };
}

pub struct TaskDL<'a> {
    pub store: &'a dyn Store,
    pub dlrs: Dlrs<'a>,
    pub notify_tx: &'a broadcast::Sender<crate::Event>,
}

pub struct Context<'a> {
    pub tid: Tid,
    pub info_hash: &'a str,
    pub task: &'a Model,
    pub tdl: &'a dyn ThirdPartyDownloader,
    pub next_event: Option<Event>,
}

impl<'a> Context<'a> {
    #[allow(clippy::transmute_ptr_to_ref)]
    #[allow(clippy::transmuting_null)]
    #[allow(clippy::missing_transmute_annotations)]
    pub fn uninit(tdl: &'a dyn ThirdPartyDownloader) -> Self {
        Self {
            tid: Tid::from(""),
            info_hash: "",
            task: unsafe { std::mem::transmute(std::ptr::null::<Model>()) },
            tdl,
            next_event: None,
        }
    }
}

#[derive(Debug)]
pub enum Event {
    // 初始化
    Init(DownloadStatus),

    // 启动任务
    Start(Resource),
    // 取消任务
    Cancel,
    // 暂停任务
    Pause,
    // 恢复任务
    Resume,
    // 重试任务
    Retry,
    // 自动降级
    Fallback,
    // 任务失败
    Failed(String, String),
    // 移除任务
    Remove(bool),
    // 任务完成
    Completed(Option<String>),
    // 任务同步
    Synced(DownloadStatus),

    // 关闭状态机
    Shutdown(oneshot::Sender<()>),
}

#[state_machine(
    initial = "State::pending()",
    context_identifier = "ctx",
    state(derive(Debug))
)]

/// 状态机 State Transition Table
#[allow(clippy::needless_lifetimes)]
impl<'a> TaskDL<'a> {
    #[state]
    async fn pending(&self, ctx: &mut Context<'_>, event: &Event) -> Response<State> {
        match event {
            Event::Start(resource) => run_action!(self.start(ctx, resource)),
            Event::Failed(_, err_msg) => run_action!(self.fail(ctx, err_msg)),
            Event::Remove(remove_files) => run_action!(self.remove(ctx, *remove_files)),
            Event::Synced(status) => run_action!(self.sync(ctx, status)),
            Event::Init(status) => run_action!(self.init(ctx, status)),
            _ => Handled,
        }
    }

    #[state]
    async fn downloading(&self, ctx: &mut Context<'_>, event: &Event) -> Response<State> {
        match event {
            Event::Cancel => run_action!(self.cancel(ctx)),
            Event::Pause => run_action!(self.pause(ctx)),
            Event::Failed(_, err_msg) => run_action!(self.fail(ctx, err_msg)),
            Event::Completed(result) => run_action!(self.complete(ctx, result.to_owned())),
            Event::Remove(remove_files) => run_action!(self.remove(ctx, *remove_files)),
            Event::Synced(status) => run_action!(self.sync(ctx, status)),
            Event::Init(status) => run_action!(self.init(ctx, status)),
            _ => Handled,
        }
    }

    #[state]
    async fn paused(&self, ctx: &mut Context<'_>, event: &Event) -> Response<State> {
        match event {
            Event::Resume => run_action!(self.resume(ctx)),
            Event::Remove(remove_files) => run_action!(self.remove(ctx, *remove_files)),
            Event::Synced(status) => run_action!(self.sync(ctx, status)),
            Event::Init(status) => run_action!(self.init(ctx, status)),
            _ => Handled,
        }
    }

    #[state]
    async fn retrying(&self, ctx: &mut Context<'_>, event: &Event) -> Response<State> {
        match event {
            Event::Cancel => run_action!(self.cancel(ctx)),
            Event::Pause => run_action!(self.pause(ctx)),
            Event::Retry => run_action!(self.retry(ctx)),
            Event::Remove(remove_files) => run_action!(self.remove(ctx, *remove_files)),
            Event::Synced(status) => run_action!(self.sync(ctx, status)),
            Event::Init(status) => run_action!(self.init(ctx, status)),
            _ => Handled,
        }
    }

    #[state]
    async fn failed(&self, ctx: &mut Context<'_>, event: &Event) -> Response<State> {
        match event {
            Event::Retry => run_action!(self.retry(ctx)),
            Event::Fallback => run_action!(self.fallback(ctx)),
            Event::Remove(remove_files) => run_action!(self.remove(ctx, *remove_files)),
            Event::Synced(status) => run_action!(self.sync(ctx, status)),
            Event::Init(status) => run_action!(self.init(ctx, status)),
            _ => Handled,
        }
    }

    #[state]
    async fn completed(&self, ctx: &mut Context<'_>, event: &Event) -> Response<State> {
        match event {
            Event::Remove(remove_files) => run_action!(self.remove(ctx, *remove_files)),
            Event::Synced(status) => run_action!(self.sync(ctx, status)),
            Event::Init(status) => run_action!(self.init(ctx, status)),
            _ => Handled,
        }
    }

    #[state]
    async fn cancelled(&self, ctx: &mut Context<'_>, event: &Event) -> Response<State> {
        match event {
            Event::Remove(remove_files) => run_action!(self.remove(ctx, *remove_files)),
            Event::Synced(status) => run_action!(self.sync(ctx, status)),
            Event::Init(status) => run_action!(self.init(ctx, status)),
            _ => Handled,
        }
    }
}

/// 状态机Actions
#[allow(clippy::needless_lifetimes)]
impl<'a> TaskDL<'a> {
    async fn start(&self, ctx: &mut Context<'_>, resource: &Resource) -> Result<Response<State>> {
        let info_hash = resource.info_hash();
        let dir = ctx.task.dir.clone();

        info!("开始任务: info_hash={} dir={}", info_hash, dir);

        match ctx.tdl.add_task(resource.clone(), dir.into()).await {
            Ok((tid, result)) => {
                self.update_status(info_hash, DownloadStatus::Downloading, None, result)
                    .await?;

                if let Some(tid) = tid {
                    self.store.update_tid(info_hash, &tid).await?;
                }

                Ok(Transition(State::downloading()))
            }
            Err(e) => {
                ctx.next_event = Some(Event::Failed(info_hash.to_owned(), e.to_string()));
                Ok(Handled)
            }
        }
    }

    async fn fail(&self, ctx: &mut Context<'_>, err_msg: &str) -> Result<Response<State>> {
        warn!("任务失败: info_hash={} err_msg={}", ctx.info_hash, err_msg);

        ctx.tdl
            .remove_task(&ctx.tid, true)
            .await
            .inspect_err(|e| {
                warn!("移除失败任务出错: tid={}, 错误: {}", ctx.tid, e);
            })
            .ok();

        if ctx.task.retry_count >= ctx.tdl.config().max_retry_count {
            self.update_status(
                ctx.info_hash,
                DownloadStatus::Failed,
                Some(format!(
                    "重试次数超过上限({}): {}",
                    ctx.tdl.config().max_retry_count,
                    err_msg
                )),
                None,
            )
            .await?;

            if ctx.task.allow_fallback {
                ctx.next_event = Some(Event::Fallback);
            }
            Ok(Transition(State::failed()))
        } else {
            let next_retry_at = ctx
                .tdl
                .config()
                .calculate_next_retry(ctx.task.retry_count + 1);

            self.update_retry_status(ctx.info_hash, next_retry_at, Some(err_msg.to_string()))
                .await?;

            ctx.next_event = Some(Event::Retry);
            Ok(Transition(State::retrying()))
        }
    }

    async fn cancel(&self, ctx: &mut Context<'_>) -> Result<Response<State>> {
        info!("取消任务: info_hash={}", ctx.info_hash);

        ctx.tdl
            .cancel_task(&ctx.tid)
            .await
            .inspect_err(|e| {
                warn!("取消任务出错: tid={} 错误: {}", ctx.tid, e);
            })
            .ok();

        self.update_status(ctx.info_hash, DownloadStatus::Cancelled, None, None)
            .await?;

        Ok(Transition(State::cancelled()))
    }

    async fn pause(&self, ctx: &mut Context<'_>) -> Result<Response<State>> {
        info!("暂停任务: info_hash={}", ctx.info_hash);

        ctx.tdl
            .pause_task(&ctx.tid)
            .await
            .inspect_err(|e| {
                warn!("暂停任务出错: tid={} 错误: {}", ctx.tid, e);
            })
            .ok();

        self.update_status(ctx.info_hash, DownloadStatus::Paused, None, None)
            .await?;

        Ok(Transition(State::paused()))
    }

    async fn resume(&self, ctx: &mut Context<'_>) -> Result<Response<State>> {
        info!("恢复任务: info_hash={}", ctx.info_hash);

        ctx.tdl
            .resume_task(&ctx.tid)
            .await
            .inspect_err(|e| {
                warn!("恢复任务出错: tid={}, 错误: {}", ctx.tid, e);
            })
            .ok();

        self.update_status(ctx.info_hash, DownloadStatus::Downloading, None, None)
            .await?;

        Ok(Transition(State::downloading()))
    }

    async fn complete(
        &self,
        ctx: &mut Context<'_>,
        result: Option<String>,
    ) -> Result<Response<State>> {
        info!("任务完成: info_hash={} result={:?}", ctx.info_hash, result);

        if ctx.tdl.config().delete_task_on_completion {
            ctx.tdl
                .remove_task(&ctx.tid, false)
                .await
                .inspect_err(|e| {
                    warn!("移除任务出错: tid={}, 错误: {}", ctx.tid, e);
                })
                .ok();
        }

        self.update_status(ctx.info_hash, DownloadStatus::Completed, None, result)
            .await?;

        Ok(Transition(State::completed()))
    }

    async fn retry(&self, ctx: &mut Context<'_>) -> Result<Response<State>> {
        info!("重试任务: info_hash={}", ctx.info_hash);

        let resource = self
            .store
            .load_resource(ctx.info_hash)
            .await?
            .ok_or(Error::ResourceNotFound(ctx.info_hash.to_string()))?;

        ctx.tdl
            .remove_task(&ctx.tid, true)
            .await
            .inspect_err(|e| {
                warn!("移除任务准备重试出错: tid={}, 错误: {}", ctx.tid, e);
            })
            .ok();

        self.update_status(ctx.info_hash, DownloadStatus::Pending, None, None)
            .await?;

        ctx.next_event = Some(Event::Start(resource));
        Ok(Transition(State::pending()))
    }

    async fn sync(
        &self,
        ctx: &mut Context<'_>,
        status: &DownloadStatus,
    ) -> Result<Response<State>> {
        info!(
            "任务状态同步: info_hash={}, status={:?}",
            ctx.info_hash, status
        );

        self.update_status(ctx.info_hash, status.clone(), None, None)
            .await?;

        self.init(ctx, status).await
    }

    async fn init(
        &self,
        _ctx: &mut Context<'_>,
        status: &DownloadStatus,
    ) -> Result<Response<State>> {
        match status {
            DownloadStatus::Downloading => Ok(Transition(State::downloading())),
            DownloadStatus::Paused => Ok(Transition(State::paused())),
            DownloadStatus::Failed => Ok(Transition(State::failed())),
            DownloadStatus::Completed => Ok(Transition(State::completed())),
            DownloadStatus::Cancelled => Ok(Transition(State::cancelled())),
            DownloadStatus::Pending => Ok(Transition(State::pending())),
            DownloadStatus::Retrying => Ok(Transition(State::retrying())),
        }
    }

    async fn fallback(&self, ctx: &mut Context<'_>) -> Result<Response<State>> {
        // 找到优先级最高的未使用下载器
        let fallback = self.dlrs.best_unused(&ctx.task.downloader);

        match fallback {
            Some(dlr) => {
                info!(
                    "自动降级: info_hash={}, 使用下载器: {}",
                    ctx.info_hash,
                    dlr.name()
                );

                let new_dlr = assign_dlr(&ctx.task.downloader, dlr.name());

                self.store.assign_dlr(ctx.info_hash, new_dlr).await?;

                let resource = self
                    .store
                    .load_resource(ctx.info_hash)
                    .await?
                    .ok_or(Error::ResourceNotFound(ctx.info_hash.to_string()))?;

                ctx.next_event = Some(Event::Start(resource));
                Ok(Transition(State::pending()))
            }
            None => {
                warn!(
                    "自动降级失败,没有可用的备选下载器: info_hash={}",
                    ctx.info_hash
                );

                self.update_status(
                    ctx.info_hash,
                    DownloadStatus::Failed,
                    Some(format!(
                        "没有可用的备选下载器: {}",
                        &ctx.task.err_msg.as_ref().unwrap_or(&"".to_string())
                    )),
                    None,
                )
                .await?;

                Ok(Transition(State::failed()))
            }
        }
    }

    async fn remove(&self, ctx: &mut Context<'_>, remove_files: bool) -> Result<Response<State>> {
        info!(
            "移除任务: info_hash={}, remove_files={}",
            ctx.info_hash, remove_files
        );

        ctx.tdl
            .remove_task(&ctx.tid, remove_files)
            .await
            .inspect_err(|e| {
                warn!("移除任务出错: tid={}, 错误: {}", ctx.tid, e);
            })
            .ok();

        self.update_status(ctx.info_hash, DownloadStatus::Cancelled, None, None)
            .await?;

        Ok(Transition(State::cancelled()))
    }
}

#[allow(clippy::needless_lifetimes)]
impl<'a> TaskDL<'a> {
    pub async fn new(
        store: &'a dyn Store,
        dlrs: &'a [Arc<Box<dyn ThirdPartyDownloader>>],
        ctx: &mut Context<'_>,
        notify_tx: &'a broadcast::Sender<crate::Event>,
    ) -> InitializedStateMachine<Self> {
        Self {
            store,
            dlrs: dlrs.into(),
            notify_tx,
        }
        .uninitialized_state_machine()
        .init_with_context(ctx)
        .await
    }

    async fn update_status(
        &self,
        info_hash: &str,
        status: DownloadStatus,
        err_msg: Option<String>,
        result: Option<String>,
    ) -> Result<()> {
        self.store
            .update_status(info_hash, status.clone(), err_msg.clone(), result)
            .await?;
        let _ = self.notify_tx.send(crate::Event::TaskUpdated((
            info_hash.to_string(),
            status,
            err_msg,
        )));
        Ok(())
    }

    async fn update_retry_status(
        &self,
        info_hash: &str,
        next_retry_at: NaiveDateTime,
        err_msg: Option<String>,
    ) -> Result<()> {
        self.store
            .update_retry_status(info_hash, next_retry_at, err_msg.clone())
            .await?;

        let _ = self.notify_tx.send(crate::Event::TaskUpdated((
            info_hash.to_string(),
            DownloadStatus::Retrying,
            err_msg,
        )));
        Ok(())
    }
}
