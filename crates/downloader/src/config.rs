use std::time::Duration;

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
