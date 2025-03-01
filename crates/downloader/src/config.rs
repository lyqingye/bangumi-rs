use std::time::Duration;

use chrono::{Local, NaiveDateTime};

/// 下载器配置
#[derive(Debug, Clone)]
pub struct Config {
    /// 状态同步间隔
    pub sync_interval: Duration,
    /// 请求队列大小
    pub request_queue_size: usize,
    /// 事件队列大小
    pub event_queue_size: usize,
    /// 最大重试次数
    pub max_retry_count: i32,
    /// 重试任务间隔
    pub retry_processor_interval: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sync_interval: Duration::from_secs(60),
            request_queue_size: 100,
            event_queue_size: 100,
            max_retry_count: 5,
            retry_processor_interval: Duration::from_secs(5),
        }
    }
}

impl Config {
    pub fn calculate_next_retry(&self, retry_count: i32) -> NaiveDateTime {
        const BASE_SECONDS: u64 = 30; // 基础时间：30秒
        const MAX_MINUTES: u64 = 60; // 最大间隔：60分钟
        const MAX_SECONDS: u64 = MAX_MINUTES * 60; // 最大间隔（秒）：3600秒

        // 计算当前重试间隔（秒）
        let delay_seconds = BASE_SECONDS * 2u64.pow((retry_count as u32).min(7));

        // 确保不超过最大间隔
        let final_delay = delay_seconds.min(MAX_SECONDS);

        Local::now().naive_utc() + Duration::from_secs(final_delay)
    }
}
