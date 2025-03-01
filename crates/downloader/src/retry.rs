use crate::worker::{Event, Worker};
use anyhow::Result;
use chrono::Local;
use model::sea_orm_active_enums::DownloadStatus;
use tracing::{error, info, warn};

impl Worker {
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
            .list_by_statues(&[DownloadStatus::Retrying])
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
                        None,
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
            self.send_event(Event::AutoRetry(task.info_hash.clone()))
                .await?;
        }

        Ok(())
    }
}
