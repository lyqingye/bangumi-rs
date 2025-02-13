use super::Notifier;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use teloxide::prelude::*;
use teloxide::types::InputFile;
/// Telegram 通知实现
pub struct TelegramNotifier {
    bot: Bot,
    chat_id: ChatId,
}

impl TelegramNotifier {
    /// 从环境变量创建实例
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            bot: Bot::new(std::env::var("TELEGRAM_BOT_TOKEN")?),
            chat_id: ChatId(std::env::var("TELEGRAM_CHAT_ID")?.parse()?),
        })
    }

    pub fn new_with_client(client: Client, api_key: &str, chat_id: &str) -> Result<Self> {
        Ok(Self {
            bot: Bot::with_client(api_key, client),
            chat_id: ChatId(chat_id.parse()?),
        })
    }
}

#[async_trait]
impl Notifier for TelegramNotifier {
    async fn send_message(&self, text: &str) -> Result<()> {
        self.bot
            .send_message(self.chat_id, text)
            .await
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("Telegram send failed: {}", e))
    }

    async fn send_formatted_message(&self, text: &str, parse_mode: &str) -> Result<()> {
        self.bot
            .send_message(self.chat_id, text)
            .parse_mode(parse_mode.parse().unwrap()) // 支持 Markdown/HTML
            .await
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("Telegram send failed: {}", e))
    }

    async fn send_message_with_attachment(
        &self,
        text: &str,
        attachment: &[u8],
        file_name: &str,
    ) -> Result<()> {
        let document = InputFile::memory(attachment.to_owned()).file_name(file_name.to_owned());

        self.bot
            .send_document(self.chat_id, document)
            .caption(text)
            .await
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("Telegram send failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn test_telegram_notifier() -> Result<()> {
        dotenv::dotenv()?;
        let notifier = TelegramNotifier::from_env()?;
        notifier.send_message("Hello, world!").await?;
        Ok(())
    }
}
