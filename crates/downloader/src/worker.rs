use std::sync::Arc;

use anyhow::Result;
use model::{sea_orm_active_enums::DownloadStatus, torrent_download_tasks::Model};
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, error, info};

use crate::{config::Config, tasks::TaskManager, RemoteTaskStatus, ThirdPartyDownloader};

type State = DownloadStatus;

#[derive(Clone)]
pub enum Event {
    // 外部事件
    StartTask(String),
    CancelTask(String),
    RetryTask(String),

    // 内部事件
    RemoteTaskUpdated(String, RemoteTaskStatus),
    AutoRetry(String),
    TaskFailed(String, String),
}

impl Event {
    async fn get_ref_task(&self, tasks: &TaskManager) -> Result<Model> {
        let info_hash = match self {
            Self::StartTask(info_hash) => info_hash,
            Self::CancelTask(info_hash) => info_hash,
            Self::RetryTask(info_hash) => info_hash,
            Self::RemoteTaskUpdated(info_hash, _) => info_hash,
            Self::AutoRetry(info_hash) => info_hash,
            Self::TaskFailed(info_hash, _) => info_hash,
        };
        tasks
            .get_by_info_hash(&info_hash)
            .await?
            .ok_or_else(|| anyhow::anyhow!("任务不存在: info_hash={}", info_hash))
    }
}

pub struct Context {
    ref_task: Model,
}

#[derive(Clone)]
pub struct Worker {
    event_queue: mpsc::Sender<Event>,
    pub(crate) tasks: TaskManager,
    pub(crate) downloader: Arc<dyn ThirdPartyDownloader>,
    pub(crate) config: Config,
}

impl Worker {
    pub async fn spawn(self) -> Result<()> {
        let (event_queue, event_receiver) = mpsc::channel(100);
        // 启动事件循环
        self.spawn_event_loop(event_receiver).await?;
        // 启动同步器
        self.spawn_syncer().await?;
        // 恢复未处理的下载任务
        self.recover_pending_tasks().await?;
        Ok(())
    }

    pub(crate) async fn send_event(&self, event: Event) -> Result<()> {
        self.event_queue.send(event).await?;
        Ok(())
    }

    async fn recover_pending_tasks(&self) -> Result<()> {
        info!("开始恢复未处理的下载任务");

        let pending_tasks = self
            .tasks
            .list_by_statues(&[DownloadStatus::Pending])
            .await?;

        info!("找到 {} 个未处理的任务", pending_tasks.len());
        for task in pending_tasks {
            if let Err(e) = self
                .send_event(Event::StartTask(task.info_hash.clone()))
                .await
            {
                error!("恢复任务到队列失败: {} - {}", task.info_hash, e);
            } else {
                info!("成功恢复任务: info_hash={}", task.info_hash);
            }
        }

        info!("完成恢复未处理的下载任务");
        Ok(())
    }

    async fn spawn_event_loop(&self, mut receiver: mpsc::Receiver<Event>) -> Result<()> {
        let worker = self.clone();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match worker.handle_event(event).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("处理事件失败: {}", e);
                    }
                }
            }
        });
        Ok(())
    }

    async fn handle_event(&self, event: Event) -> Result<()> {
        let task = event.get_ref_task(&self.tasks).await?;
        let mut ctx = Context {
            ref_task: task.clone(),
        };
        let mut event = Some(event);
        let mut state = Some(task.download_status);
        loop {
            match (event, state) {
                (Some(cur_event), Some(cur_state)) => {
                    let (next_event, next_state) =
                        self.transition(cur_event, cur_state, &mut ctx).await?;
                    event = next_event;
                    state = next_state;
                }
                _ => break,
            }
        }
        Ok(())
    }

    async fn transition(
        &self,
        event: Event,
        state: State,
        ctx: &mut Context,
    ) -> Result<(Option<Event>, Option<State>)> {
        match (event, state.clone()) {
            // 开始任务
            (Event::StartTask(info_hash), State::Pending) => {
                self.on_start_task(info_hash, ctx).await
            }

            // 取消任务
            (
                Event::CancelTask(info_hash),
                State::Downloading | State::Pending | State::Retrying,
            ) => self.on_task_cancelled(info_hash, ctx).await,

            // 重试任务
            (Event::RetryTask(info_hash), State::Failed | State::Retrying | State::Cancelled) => {
                self.on_task_retry(info_hash, ctx).await
            }

            // 自动重试
            (Event::AutoRetry(info_hash), State::Retrying) => {
                self.on_task_retry(info_hash, ctx).await
            }

            // 任务失败
            (Event::TaskFailed(info_hash, err_msg), State::Downloading | State::Pending) => {
                self.on_task_failed(info_hash, err_msg, ctx).await
            }

            // 远程任务状态更新
            (
                Event::RemoteTaskUpdated(info_hash, remote_task_status),
                State::Pending | State::Retrying | State::Downloading,
            ) => {
                self.on_remote_task_updated(info_hash, remote_task_status, ctx)
                    .await
            }
            _ => Ok((None, None)),
        }
    }
}

impl Worker {
    async fn on_start_task(
        &self,
        info_hash: String,
        ctx: &mut Context,
    ) -> Result<(Option<Event>, Option<State>)> {
        info!("开始处理任务: info_hash={}", info_hash);

        debug_assert_eq!(ctx.ref_task.info_hash, info_hash);
        debug_assert_eq!(ctx.ref_task.download_status, DownloadStatus::Pending);

        match self
            .downloader
            .add_task(&info_hash, ctx.ref_task.dir.clone().into())
            .await
        {
            Ok(_) => {
                self.tasks
                    .update_task_status(&info_hash, DownloadStatus::Downloading, None, None)
                    .await?;
                Ok((None, Some(State::Downloading)))
            }

            Err(e) => Ok((
                Some(Event::TaskFailed(info_hash, e.to_string())),
                Some(State::Pending),
            )),
        }
    }

    async fn on_remote_task_updated(
        &self,
        info_hash: String,
        remote_task_status: RemoteTaskStatus,
        ctx: &mut Context,
    ) -> Result<(Option<Event>, Option<State>)> {
        match remote_task_status.status {
            DownloadStatus::Failed => {
                self.tasks
                    .update_task_status(
                        &info_hash,
                        DownloadStatus::Failed,
                        remote_task_status.err_msg.clone(),
                        None,
                    )
                    .await?;
                Ok((
                    Some(Event::TaskFailed(
                        info_hash,
                        remote_task_status.err_msg.unwrap_or_default(),
                    )),
                    Some(State::Failed),
                ))
            }
            _ => {
                self.tasks
                    .update_task_status(&info_hash, remote_task_status.status.clone(), None, None)
                    .await?;
                Ok((None, Some(remote_task_status.status)))
            }
        }
    }

    async fn on_task_failed(
        &self,
        info_hash: String,
        err_msg: String,
        ctx: &mut Context,
    ) -> Result<(Option<Event>, Option<State>)> {
        self.downloader.remove_task(&info_hash).await?;
        self.tasks
            .update_task_status(&info_hash, DownloadStatus::Failed, Some(err_msg), None)
            .await?;
        Ok((None, Some(State::Failed)))
    }

    async fn on_task_retry(
        &self,
        info_hash: String,
        ctx: &mut Context,
    ) -> Result<(Option<Event>, Option<State>)> {
        // 删除原有任务，然后重新下载
        self.downloader.remove_task(&info_hash).await?;
        self.tasks
            .update_task_status(&info_hash, DownloadStatus::Pending, None, None)
            .await?;
        Ok((Some(Event::StartTask(info_hash)), Some(State::Pending)))
    }

    async fn on_task_cancelled(
        &self,
        info_hash: String,
        ctx: &mut Context,
    ) -> Result<(Option<Event>, Option<State>)> {
        self.downloader.cancel_task(&info_hash).await?;
        self.tasks
            .update_task_status(&info_hash, DownloadStatus::Cancelled, None, None)
            .await?;
        Ok((None, Some(State::Cancelled)))
    }
}
