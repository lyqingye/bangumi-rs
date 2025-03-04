use crate::worker::{Tx, Worker};
use anyhow::Result;
use chrono::Local;
use model::sea_orm_active_enums::DownloadStatus;
use tracing::{error, info, warn};

impl Worker {
    pub(crate) fn spawn_retry_processor(&self) {
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
        info!("开始处理重试队列");
        let now = Local::now().naive_utc();
        let mut tasks = self
            .store
            .list_by_status(&[DownloadStatus::Retrying])
            .await?;
        info!("重试队列中的任务数量: {}", tasks.len());
        for task in tasks.as_mut_slice() {
            if now < task.next_retry_at {
                continue;
            }

            info!(
                "开始重试任务: info_hash={}, retry_count={}",
                task.info_hash, task.retry_count
            );

            // 重试
            self.send_event(Tx::AutoRetry(task.info_hash.clone()))
                .await?;
        }

        info!("重试队列处理完成");

        Ok(())
    }
}
