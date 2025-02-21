use crate::logger::LogMessage;
use std::fmt::Write;
use tokio::sync::broadcast;
use tracing_subscriber::Layer;

pub struct BroadcastLayer {
    log_tx: broadcast::Sender<LogMessage>,
    filter: tracing::level_filters::LevelFilter,
}

impl BroadcastLayer {
    pub fn new(
        log_tx: broadcast::Sender<LogMessage>,
        filter: tracing::level_filters::LevelFilter,
    ) -> Self {
        Self { log_tx, filter }
    }
}

impl<S> Layer<S> for BroadcastLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let level = event.metadata().level();
        // 检查日志级别是否满足过滤条件
        if !self.filter.enabled(event.metadata(), ctx) {
            return;
        }

        // 创建一个字符串缓冲区来存储格式化的日志
        let mut buffer = String::new();

        // 写入时间戳
        let now = chrono::Local::now();
        write!(&mut buffer, "[{}] ", now.format("%Y-%m-%d %H:%M:%S")).ok();

        write!(&mut buffer, "[{}] ", level).ok();

        // 写入实际的日志消息
        let mut visitor = StringVisitor(String::new());
        event.record(&mut visitor);
        buffer.push_str(&visitor.0);

        // 发送日志消息到broadcast channel
        let _ = self.log_tx.send(LogMessage { content: buffer });
    }
}

// 用于访问日志字段的辅助结构
struct StringVisitor(String);

impl tracing::field::Visit for StringVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            write!(&mut self.0, "{:?}", value).ok();
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.0.push_str(value);
        }
    }
}
