use std::{path::PathBuf, time::Duration};

use chrono::{Local, NaiveDateTime};

/// 下载器配置
#[derive(Debug, Clone)]
pub struct Config {
    /// 状态同步间隔
    pub sync_interval: Duration,
    /// 事件队列大小
    pub event_queue_size: usize,
    /// 最大重试次数
    pub max_retry_count: i32,
    /// 重试任务间隔
    pub retry_processor_interval: Duration,
    /// 重试最小间隔
    pub retry_min_interval: chrono::Duration,
    /// 重试最大间隔
    pub retry_max_interval: chrono::Duration,
    /// 下载目录
    pub download_dir: PathBuf,
    /// 下载超时
    pub download_timeout: chrono::Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sync_interval: Duration::from_secs(10),
            event_queue_size: 128,
            max_retry_count: 5,
            retry_processor_interval: Duration::from_secs(5),
            retry_min_interval: chrono::Duration::seconds(30),
            retry_max_interval: chrono::Duration::minutes(10),
            download_dir: PathBuf::from("/"),
            download_timeout: chrono::Duration::minutes(30),
        }
    }
}

impl Config {
    pub fn calculate_next_retry(&self, retry_count: i32) -> NaiveDateTime {
        let diff = (self.retry_max_interval - self.retry_min_interval)
            .num_nanoseconds()
            .unwrap_or_default();
        // 计算当前重试间隔（秒）
        let delay = self.retry_min_interval
            + chrono::Duration::nanoseconds(diff * 2i64.pow((retry_count as u32).min(7)));

        // 确保不超过最大间隔
        let final_delay = delay.min(self.retry_max_interval);

        Local::now().naive_utc() + final_delay
    }
}
