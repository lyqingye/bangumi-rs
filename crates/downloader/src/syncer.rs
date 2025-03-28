use std::collections::HashMap;

use crate::{
    Tid,
    worker::{Tx, Worker},
};
use anyhow::{Context as AnyhowContext, Result};
use chrono::Local;
use model::{sea_orm_active_enums::DownloadStatus, torrent_download_tasks::Model};
use tracing::{debug, error, info, warn};

impl Worker {
    pub(crate) fn spawn_syncer(&self) -> Result<()> {
        info!("启动远程任务同步器");
        let worker = self.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(worker.config.sync_interval);
            loop {
                ticker.tick().await;
                if let Err(e) = worker.sync_remote_task_status().await {
                    error!("同步远程任务状态失败: {}", e);
                }
            }
        });
        Ok(())
    }

    pub async fn sync_remote_task_status(&self) -> Result<()> {
        let active_tasks = self
            .store
            .list_by_status(&[
                DownloadStatus::Downloading,
                DownloadStatus::Pending,
                DownloadStatus::Paused,
            ])
            .await
            .context("从数据库获取活动任务失败")?;

        if active_tasks.is_empty() {
            return Ok(());
        }

        // 按最后使用的下载器分组
        let mut tasks_by_downloader: HashMap<String, Vec<Model>> = HashMap::new();
        for task in active_tasks {
            if let Some(last_downloader_name) = task.downloader.split(',').last() {
                tasks_by_downloader
                    .entry(last_downloader_name.to_string())
                    .or_default()
                    .push(task);
            } else {
                warn!(
                    "任务 {} 的下载器字段为空或格式错误: '{}'",
                    task.info_hash, task.downloader
                );
            }
        }

        for (downloader_name, tasks) in tasks_by_downloader {
            if tasks.is_empty() {
                continue;
            }

            let downloader = match self.take_downloader_by_name(&downloader_name) {
                Ok(d) => d,
                Err(e) => {
                    error!(
                        "找不到下载器 '{}' 来同步 {} 个任务: {}",
                        downloader_name,
                        tasks.len(),
                        e
                    );
                    continue; // 跳过这个下载器的同步
                }
            };

            info!(
                "为下载器 '{}' 同步 {} 个任务的状态",
                downloader_name,
                tasks.len()
            );

            let tids: Vec<Tid> = tasks.iter().map(|task| Tid::from(task.tid())).collect();

            let remote_tasks = match downloader.list_tasks(&tids).await {
                Ok(rt) => rt,
                Err(e) => {
                    error!("调用下载器 '{}' 的 list_tasks 失败: {}", downloader_name, e);
                    continue; // 跳过这个下载器的同步
                }
            };

            for local_task in tasks {
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
                    warn!("任务在下载器 '{}' 中不存在: {}", downloader_name, info_hash);
                    (
                        DownloadStatus::Pending,
                        Some(format!("任务在下载器 '{}' 中不存在", downloader_name)),
                        None,
                    )
                };

                // 检查状态是否变化
                if status != local_task.download_status {
                    info!(
                        "远程任务状态更新: info_hash={}, old_status={:?}, new_status={:?}, err_msg={:?}",
                        info_hash, local_task.download_status, status, err_msg
                    );

                    match status {
                        DownloadStatus::Completed => {
                            self.send_event(Tx::TaskCompleted(info_hash, result))?;
                        }
                        DownloadStatus::Cancelled => {
                            // 通常由用户操作触发，同步时发现Cancelled可能意味着远程被取消
                            self.send_event(Tx::CancelTask(info_hash))?;
                        }
                        DownloadStatus::Failed => {
                            self.send_event(Tx::TaskFailed(
                                info_hash,
                                err_msg.unwrap_or_else(|| "未知错误".to_string()),
                            ))?;
                        }
                        DownloadStatus::Paused | DownloadStatus::Downloading => {
                            // 远程状态与本地不一致，发送事件以更新本地状态
                            info!(
                                "远程任务状态与本地不符: info_hash={}, local={:?}, remote={:?}",
                                info_hash, local_task.download_status, status
                            );
                            self.send_event(Tx::TaskStatusUpdated((info_hash, status)))?;
                        }
                        _ => {
                            warn!(
                                "同步时遇到未处理的任务状态: info_hash={}, local_status={:?}, remote_status={:?}, err_msg={:?}",
                                info_hash, local_task.download_status, status, err_msg
                            );
                        }
                    }
                }
                // 状态未变，检查超时 (仅对 Pending, Downloading, Paused 状态)
                else if matches!(
                    local_task.download_status,
                    DownloadStatus::Pending | DownloadStatus::Downloading | DownloadStatus::Paused
                ) {
                    let now = Local::now().naive_utc();
                    let elapsed = now - local_task.updated_at;
                    // 使用当前下载器的配置来检查超时
                    if elapsed > downloader.config().download_timeout {
                        warn!(
                            "下载超时 ({} > {:?}): info_hash={}",
                            elapsed,
                            downloader.config().download_timeout,
                            info_hash
                        );
                        self.send_event(Tx::TaskFailed(info_hash, "下载超时".to_string()))?;
                    }
                }
            }
        }
        info!("同步远程任务状态完成");
        Ok(())
    }
}
