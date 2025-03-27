use crate::{Store, ThirdPartyDownloader, Tid, resource::Resource};
use anyhow::Result;
use bytes::Bytes;
use model::{
    sea_orm_active_enums::{DownloadStatus, ResourceType},
    torrent_download_tasks::Model,
};
use statig::awaitable::InitializedStateMachine;
use statig::prelude::*;
use std::sync::Arc;
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

#[derive(Clone)]
pub struct TaskStm {
    pub store: Arc<Box<dyn Store>>,
    pub downloaders: Vec<Arc<Box<dyn ThirdPartyDownloader>>>,
}

#[derive(Clone)]
pub struct Context<'a> {
    pub tid: Tid,
    pub info_hash: &'a str,
    pub task: &'a Model,
    pub tdl: &'a dyn ThirdPartyDownloader,
    pub next_event: Option<Event>,
}

impl Context<'_> {
    #[allow(invalid_value)]
    pub fn uninitialized() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[derive(Clone)]
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
    Remove,
    // 任务完成
    Completed,
    // 任务同步
    Synced(DownloadStatus),
}

#[state_machine(initial = "State::pending()", context_identifier = "ctx")]
impl TaskStm {
    #[state]
    async fn pending(&self, ctx: &mut Context<'_>, event: &Event) -> Response<State> {
        match event {
            Event::Start(resource) => run_action!(self.act_start(ctx, resource)),
            Event::Failed(_, err_msg) => run_action!(self.act_fail(ctx, err_msg)),
            Event::Remove => run_action!(self.act_remove(ctx)),
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
            Event::Completed => run_action!(self.act_complete(ctx)),
            Event::Remove => run_action!(self.act_remove(ctx)),
            Event::Synced(status) => run_action!(self.act_sync(ctx, status)),
            _ => Handled,
        }
    }

    #[state]
    async fn paused(&self, ctx: &mut Context<'_>, event: &Event) -> Response<State> {
        match event {
            Event::Resume => run_action!(self.act_resume(ctx)),
            Event::Remove => run_action!(self.act_remove(ctx)),
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
            Event::Remove => run_action!(self.act_remove(ctx)),
            Event::Synced(status) => run_action!(self.act_sync(ctx, status)),
            _ => Handled,
        }
    }

    #[state]
    async fn failed(&self, ctx: &mut Context<'_>, event: &Event) -> Response<State> {
        match event {
            Event::Retry => run_action!(self.act_retry(ctx)),
            Event::Fallback => run_action!(self.act_fallback(ctx)),
            Event::Remove => run_action!(self.act_remove(ctx)),
            Event::Synced(status) => run_action!(self.act_sync(ctx, status)),
            _ => Handled,
        }
    }

    #[state]
    async fn completed(&self, ctx: &mut Context<'_>, event: &Event) -> Response<State> {
        match event {
            Event::Remove => run_action!(self.act_remove(ctx)),
            Event::Synced(status) => run_action!(self.act_sync(ctx, status)),
            _ => Handled,
        }
    }

    #[state]
    async fn cancelled(&self, ctx: &mut Context<'_>, event: &Event) -> Response<State> {
        match event {
            Event::Remove => run_action!(self.act_remove(ctx)),
            Event::Synced(status) => run_action!(self.act_sync(ctx, status)),
            _ => Handled,
        }
    }
}

impl TaskStm {
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
                    Some(err_msg.to_string()),
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

    async fn act_complete(&self, ctx: &mut Context<'_>) -> Result<Response<State>> {
        if ctx.tdl.config().delete_task_on_completion {
            if let Err(e) = ctx.tdl.remove_task(&ctx.tid, false).await {
                warn!("移除任务出错: tid={}, 错误: {}", ctx.tid, e);
            }
        }

        self.store
            .update_status(ctx.info_hash, DownloadStatus::Completed, None, None)
            .await?;

        Ok(Transition(State::completed()))
    }

    async fn act_retry(&self, ctx: &mut Context<'_>) -> Result<Response<State>> {
        let resource = self.get_task_resource_by_info_hash(ctx.info_hash).await?;
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
        let fallback_downloader = self
            .downloaders
            .iter()
            .filter(|d| !ctx.task.downloader.contains(d.name()))
            .max_by_key(|d| d.config().priority)
            .map(|d| d.name());

        match fallback_downloader {
            Some(downloader) => {
                let mut new_downloader = ctx.task.downloader.to_string();
                new_downloader.push(',');
                new_downloader.push_str(downloader);

                self.store
                    .assign_downloader(ctx.info_hash, new_downloader)
                    .await?;

                let resource = self.get_task_resource_by_info_hash(ctx.info_hash).await?;
                ctx.next_event = Some(Event::Start(resource));
                Ok(Transition(State::pending()))
            }
            None => Ok(Transition(State::failed())),
        }
    }

    async fn act_remove(&self, ctx: &mut Context<'_>) -> Result<Response<State>> {
        if let Err(e) = ctx.tdl.remove_task(&ctx.tid, true).await {
            warn!("移除任务出错: tid={}, 错误: {}", ctx.tid, e);
        }

        self.store
            .update_status(ctx.info_hash, DownloadStatus::Cancelled, None, None)
            .await?;

        Ok(Transition(State::cancelled()))
    }
}

impl TaskStm {
    async fn get_task_resource(&self, task: &Model) -> Result<Resource> {
        Ok(match task.resource_type {
            ResourceType::Torrent => {
                let torrent = self
                    .store
                    .get_torrent_by_info_hash(&task.info_hash)
                    .await?
                    .ok_or_else(|| {
                        anyhow::anyhow!("种子文件不存在: info_hash={}", task.info_hash)
                    })?;
                let data = torrent
                    .data
                    .ok_or_else(|| anyhow::anyhow!("种子内容为空: info_hash={}", task.info_hash))?;
                Resource::TorrentFileBytes(Bytes::from(data), task.info_hash.clone())
            }
            ResourceType::Magnet => {
                let magnet = task
                    .magnet
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("磁力链接为空: info_hash={}", task.info_hash))?;
                Resource::from_magnet_link(magnet)?
            }
            ResourceType::InfoHash => Resource::from_info_hash(task.info_hash.clone())?,
            ResourceType::TorrentURL => {
                let torrent_url = task.torrent_url.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("种子下载URL为空: info_hash={}", task.info_hash)
                })?;
                Resource::from_torrent_url(torrent_url, &task.info_hash)?
            }
        })
    }

    async fn get_task_resource_by_info_hash(&self, info_hash: &str) -> Result<Resource> {
        let tasks = self.store.list_by_hashes(&[info_hash.to_string()]).await?;
        let task = tasks
            .first()
            .ok_or_else(|| anyhow::anyhow!("任务不存在: info_hash={}", info_hash))?;
        self.get_task_resource(task).await
    }
}

impl TaskStm {
    pub async fn new(
        store: Arc<Box<dyn Store>>,
        downloaders: Vec<Arc<Box<dyn ThirdPartyDownloader>>>,
        ctx: &mut Context<'_>,
    ) -> InitializedStateMachine<Self> {
        Self { store, downloaders }
            .uninitialized_state_machine()
            .init_with_context(ctx)
            .await
    }
}
