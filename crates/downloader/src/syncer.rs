use crate::{
    worker::{Event, Worker},
    RemoteTaskStatus,
};
use anyhow::Result;
use model::sea_orm_active_enums::DownloadStatus;
use tracing::{debug, info, warn};

impl Worker {
    pub(crate) fn spawn_syncer(&self) -> Result<()> {
        let worker = self.clone();
        tokio::spawn(async move {
            worker.sync_remote_task_status().await;
        });
        Ok(())
    }

    async fn sync_remote_task_status(&self) -> Result<()> {
        let local_tasks = self
            .tasks
            .list_by_statues(&[DownloadStatus::Downloading, DownloadStatus::Pending])
            .await?;

        if local_tasks.is_empty() {
            debug!("没有需要同步的下载任务");
            return Ok(());
        }

        let target_info_hashes: Vec<String> = local_tasks
            .iter()
            .map(|task| task.info_hash.clone())
            .collect();

        let remote_tasks = self.downloader.list_tasks(&target_info_hashes).await?;

        for local_task in local_tasks {
            let info_hash = local_task.info_hash.clone();

            let (status, err_msg) = if let Some(remote_task) = remote_tasks.get(&info_hash) {
                debug!("发现远程任务: info_hash={}", info_hash);
                (remote_task.status.clone(), remote_task.err_msg.clone())
            } else {
                warn!("任务在下载器中不存在: {}", info_hash);
                (
                    DownloadStatus::Pending,
                    Some("任务在下载器中不存在".to_string()),
                )
            };

            if status.clone() != local_task.download_status {
                info!(
                    "更新任务状态: info_hash={}, old_status={:?}, new_status={:?}, err_msg={:?}",
                    info_hash, local_task.download_status, status, err_msg
                );

                match status {
                    DownloadStatus::Completed => {
                        self.send_event(Event::TaskCompleted(info_hash)).await?;
                    }

                    DownloadStatus::Cancelled => {
                        self.send_event(Event::CancelTask(info_hash)).await?;
                    }

                    DownloadStatus::Failed => {
                        self.send_event(Event::TaskFailed(info_hash, err_msg.unwrap_or_default()))
                            .await?;
                    }

                    _ => {
                        warn!(
                            "[Syncer] 未处理的任务状态: info_hash={}, status={:?}, err_msg={:?}",
                            info_hash, status, err_msg
                        );
                    }
                }
            }
        }

        Ok(())
    }
}
