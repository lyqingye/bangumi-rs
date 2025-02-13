use anyhow::Result;
use async_trait::async_trait;
pub mod telegram;
pub mod worker;

/// 通知系统核心 trait
#[async_trait]
pub trait Notifier: Send + Sync {
    /// 发送文本消息
    async fn send_message(&self, text: &str) -> Result<()>;

    /// 发送带格式的消息（Markdown/HTML等）
    async fn send_formatted_message(&self, text: &str, parse_mode: &str) -> Result<()>;

    /// 发送带附件的消息
    async fn send_message_with_attachment(
        &self,
        text: &str,
        attachment: &[u8],
        file_name: &str,
    ) -> Result<()>;
}
