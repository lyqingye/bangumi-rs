use crate::{
    ThirdPartyDownloader, Tid,
    worker::{Tx, Worker},
};
use anyhow::Result;
use chrono::Local;
use model::sea_orm_active_enums::DownloadStatus;
use tracing::{debug, error, info, warn};

impl Worker {
    pub(crate) fn spawn_syncer(&self) -> Result<()> {
        info!("启动远程任务同步器");
        let worker = self.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(worker.config.sync_interval);
            loop {
                ticker.tick().await;
                worker.sync_remote_task_status().await;
            }
        });
        Ok(())
    }

    pub async fn sync_remote_task_status(&self) {
        for downloader in &self.downloaders {
            if let Err(e) = self
                .sync_remote_task_status_for_downloader(&***downloader)
                .await
            {
                error!("同步远程任务状态失败: {}", e);
            }
        }
    }

    pub async fn sync_remote_task_status_for_downloader(
        &self,
        downloader: &dyn ThirdPartyDownloader,
    ) -> Result<()> {
        let local_tasks = self
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

        if local_tasks.is_empty() {
            return Ok(());
        }

        info!("需要同步的下载任务数量: {}", local_tasks.len());

        let tids: Vec<Tid> = local_tasks
            .iter()
            .map(|task| Tid::from(task.tid()))
            .collect();

        let remote_tasks = downloader.list_tasks(&tids).await?;

        for local_task in local_tasks {
            let info_hash = local_task.info_hash.clone();
            let tid = Tid::from(local_task.tid());

            let (status, err_msg, result) = if let Some(remote_task) = remote_tasks.get(&tid) {
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
                        self.send_event(Tx::TaskCompleted(info_hash, result))?;
                    }

                    DownloadStatus::Cancelled => {
                        self.send_event(Tx::CancelTask(info_hash))?;
                    }

                    DownloadStatus::Failed => {
                        self.send_event(Tx::TaskFailed(info_hash, err_msg.unwrap_or_default()))?;
                    }

                    // 本地状态和远程任务状态不一致，例如本地状态是Downloading, 然后远程任务被用户手动暂停了
                    // 例如本地状态是Paused, 然后远程任务被用户手动恢复了
                    // 所以此时需要更新本地任务状态
                    DownloadStatus::Paused | DownloadStatus::Downloading => {
                        info!("远程任务被手动暂停或恢复: info_hash={}", info_hash);
                        self.send_event(Tx::TaskStatusUpdated((info_hash, status)))?;
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
                    self.send_event(Tx::TaskFailed(info_hash, "下载超时".to_string()))?;
                }
            }
        }
        info!("同步远程任务状态完成");

        Ok(())
    }
}
