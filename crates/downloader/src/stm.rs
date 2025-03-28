use crate::dlrs::Dlrs;
use crate::errors::{Error, Result};
use crate::{Store, ThirdPartyDownloader, Tid, resource::Resource};
use model::sea_orm_active_enums::DownloadStatus;
use model::torrent_download_tasks::Model;
use statig::awaitable::InitializedStateMachine;
use statig::prelude::*;
use std::sync::Arc;
use tokio::sync::oneshot;
use tracing::warn;

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

#[state_machine(initial = "State::pending()", context_identifier = "ctx")]
#[allow(clippy::needless_lifetimes)]
impl<'a> TaskDL<'a> {
    #[state]
    async fn pending(&self, ctx: &mut Context<'_>, event: &Event) -> Response<State> {
        match event {
            Event::Start(resource) => run_action!(self.act_start(ctx, resource)),
            Event::Failed(_, err_msg) => run_action!(self.act_fail(ctx, err_msg)),
            Event::Remove(remove_files) => run_action!(self.act_remove(ctx, *remove_files)),
            Event::Synced(status) => run_action!(self.act_sync(ctx, status)),
            _ => Handled,
        }
    }

    #[state]
    async fn downloading(&self, ctx: &mut Context<'_>, event: &Event) -> Response<State> {
        match event {
            Event::Cancel => run_action!(self.act_cancel(ctx)),
            Event::Pause => run_action!(self.act_pause(ctx)),
            Event::Failed(_, err_msg) => run_action!(self.act_fail(ctx, err_msg)),
            Event::Completed(result) => run_action!(self.act_complete(ctx, result.to_owned())),
            Event::Remove(remove_files) => run_action!(self.act_remove(ctx, *remove_files)),
            Event::Synced(status) => run_action!(self.act_sync(ctx, status)),
            _ => Handled,
        }
    }

    #[state]
    async fn paused(&self, ctx: &mut Context<'_>, event: &Event) -> Response<State> {
        match event {
            Event::Resume => run_action!(self.act_resume(ctx)),
            Event::Remove(remove_files) => run_action!(self.act_remove(ctx, *remove_files)),
            Event::Synced(status) => run_action!(self.act_sync(ctx, status)),
            _ => Handled,
        }
    }

    #[state]
    async fn retrying(&self, ctx: &mut Context<'_>, event: &Event) -> Response<State> {
        match event {
            Event::Cancel => run_action!(self.act_cancel(ctx)),
            Event::Pause => run_action!(self.act_pause(ctx)),
            Event::Retry => run_action!(self.act_retry(ctx)),
            Event::Remove(remove_files) => run_action!(self.act_remove(ctx, *remove_files)),
            Event::Synced(status) => run_action!(self.act_sync(ctx, status)),
            _ => Handled,
        }
    }

    #[state]
    async fn failed(&self, ctx: &mut Context<'_>, event: &Event) -> Response<State> {
        match event {
            Event::Retry => run_action!(self.act_retry(ctx)),
            Event::Fallback => run_action!(self.act_fallback(ctx)),
            Event::Remove(remove_files) => run_action!(self.act_remove(ctx, *remove_files)),
            Event::Synced(status) => run_action!(self.act_sync(ctx, status)),
            _ => Handled,
        }
    }

    #[state]
    async fn completed(&self, ctx: &mut Context<'_>, event: &Event) -> Response<State> {
        match event {
            Event::Remove(remove_files) => run_action!(self.act_remove(ctx, *remove_files)),
            Event::Synced(status) => run_action!(self.act_sync(ctx, status)),
            _ => Handled,
        }
    }

    #[state]
    async fn cancelled(&self, ctx: &mut Context<'_>, event: &Event) -> Response<State> {
        match event {
            Event::Remove(remove_files) => run_action!(self.act_remove(ctx, *remove_files)),
            Event::Synced(status) => run_action!(self.act_sync(ctx, status)),
            _ => Handled,
        }
    }
}

#[allow(clippy::needless_lifetimes)]
impl<'a> TaskDL<'a> {
    async fn act_start(
        &self,
        ctx: &mut Context<'_>,
        resource: &Resource,
    ) -> Result<Response<State>> {
        let info_hash = resource.info_hash();
        let dir = ctx.task.dir.clone();

        match ctx.tdl.add_task(resource.clone(), dir.into()).await {
            Ok((tid, result)) => {
                self.store
                    .update_status(info_hash, DownloadStatus::Downloading, None, result)
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

    async fn act_fail(&self, ctx: &mut Context<'_>, err_msg: &str) -> Result<Response<State>> {
        if let Err(e) = ctx.tdl.remove_task(&ctx.tid, true).await {
            warn!("移除失败任务出错: tid={}, 错误: {}", ctx.tid, e);
        }

        if ctx.task.retry_count >= ctx.tdl.config().max_retry_count {
            self.store
                .update_status(
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

            self.store
                .update_retry_status(ctx.info_hash, next_retry_at, Some(err_msg.to_string()))
                .await?;

            ctx.next_event = Some(Event::Retry);
            Ok(Transition(State::retrying()))
        }
    }

    async fn act_cancel(&self, ctx: &mut Context<'_>) -> Result<Response<State>> {
        if let Err(e) = ctx.tdl.cancel_task(&ctx.tid).await {
            warn!("取消任务出错: tid={}, 错误: {}", ctx.tid, e);
        }

        self.store
            .update_status(ctx.info_hash, DownloadStatus::Cancelled, None, None)
            .await?;

        Ok(Transition(State::cancelled()))
    }

    async fn act_pause(&self, ctx: &mut Context<'_>) -> Result<Response<State>> {
        if let Err(e) = ctx.tdl.pause_task(&ctx.tid).await {
            warn!("暂停任务出错: tid={}, 错误: {}", ctx.tid, e);
        }

        self.store
            .update_status(ctx.info_hash, DownloadStatus::Paused, None, None)
            .await?;

        Ok(Transition(State::paused()))
    }

    async fn act_resume(&self, ctx: &mut Context<'_>) -> Result<Response<State>> {
        if let Err(e) = ctx.tdl.resume_task(&ctx.tid).await {
            warn!("恢复任务出错: tid={}, 错误: {}", ctx.tid, e);
        }

        self.store
            .update_status(ctx.info_hash, DownloadStatus::Downloading, None, None)
            .await?;

        Ok(Transition(State::downloading()))
    }

    async fn act_complete(
        &self,
        ctx: &mut Context<'_>,
        result: Option<String>,
    ) -> Result<Response<State>> {
        if ctx.tdl.config().delete_task_on_completion {
            if let Err(e) = ctx.tdl.remove_task(&ctx.tid, false).await {
                warn!("移除任务出错: tid={}, 错误: {}", ctx.tid, e);
            }
        }

        self.store
            .update_status(ctx.info_hash, DownloadStatus::Completed, None, result)
            .await?;

        Ok(Transition(State::completed()))
    }

    async fn act_retry(&self, ctx: &mut Context<'_>) -> Result<Response<State>> {
        let resource = self
            .store
            .load_resource(ctx.info_hash)
            .await?
            .ok_or(Error::ResourceNotFound(ctx.info_hash.to_string()))?;

        if let Err(e) = ctx.tdl.remove_task(&ctx.tid, true).await {
            warn!("移除任务准备重试出错: tid={}, 错误: {}", ctx.tid, e);
        }

        self.store
            .update_status(ctx.info_hash, DownloadStatus::Pending, None, None)
            .await?;

        ctx.next_event = Some(Event::Start(resource));
        Ok(Transition(State::pending()))
    }

    async fn act_sync(
        &self,
        ctx: &mut Context<'_>,
        status: &DownloadStatus,
    ) -> Result<Response<State>> {
        self.store
            .update_status(ctx.info_hash, status.clone(), None, None)
            .await?;

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

    async fn act_fallback(&self, ctx: &mut Context<'_>) -> Result<Response<State>> {
        // 找到优先级最高的未使用下载器
        let fallback = self.dlrs.best_unused(&ctx.task.downloader);

        match fallback {
            Some(dlr) => {
                let mut new_dlr = ctx.task.downloader.to_string();
                new_dlr.push(',');
                new_dlr.push_str(dlr.name());

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
                self.store
                    .update_status(
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

    async fn act_remove(
        &self,
        ctx: &mut Context<'_>,
        remove_files: bool,
    ) -> Result<Response<State>> {
        if let Err(e) = ctx.tdl.remove_task(&ctx.tid, remove_files).await {
            warn!("移除任务出错: tid={}, 错误: {}", ctx.tid, e);
        }

        self.store
            .update_status(ctx.info_hash, DownloadStatus::Cancelled, None, None)
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
    ) -> InitializedStateMachine<Self> {
        Self {
            store,
            dlrs: dlrs.into(),
        }
        .uninitialized_state_machine()
        .init_with_context(ctx)
        .await
    }
}
