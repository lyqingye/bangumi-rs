use anyhow::Result;
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{fill_file_names, parse_msg, ParseResult, Parser, PROMPT_TEMPLATE};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

#[derive(Debug, Serialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct AssistantMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: AssistantMessage,
    pub finish_reason: String,
    pub index: u32,
    pub logprobs: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
    pub created: u64,
    pub model: String,
    pub object: String,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: String::from("https://api.deepbricks.ai/v1"),
            model: String::from("GPT-4o-mini"),
        }
    }
}

#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
    config: Config,
}

impl Client {
    pub fn new(config: Config, client: reqwest::Client) -> Self {
        Self { client, config }
    }

    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("DEEPBRICKS_API_KEY")?;
        let base_url = std::env::var("DEEPBRICKS_BASE_URL")?;
        let model = std::env::var("DEEPBRICKS_MODEL")?;
        Ok(Self::new(
            Config {
                api_key,
                base_url,
                model,
            },
            reqwest::Client::new(),
        ))
    }

    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.config.api_key)).unwrap(),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers
    }

    pub async fn chat_completion(&self, messages: Vec<Message>) -> Result<ChatCompletionResponse> {
        let request = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages,
        };

        let response: ChatCompletionResponse = self
            .client
            .post(format!("{}/chat/completions", self.config.base_url))
            .headers(self.build_headers())
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }
}

#[async_trait]
impl Parser for Client {
    async fn parse_file_names(&self, file_names: Vec<String>) -> Result<Vec<ParseResult>> {
        let params = serde_json::to_string(&file_names)?;
        let messages = vec![
            Message {
                role: Role::System,
                content: PROMPT_TEMPLATE.to_owned(),
            },
            Message {
                role: Role::User,
                content: params,
            },
        ];
        let response = self.chat_completion(messages).await?;
        for choice in response.choices {
            let msg = choice.message;
            if msg.role == Role::Assistant {
                info!("AI 解析结果: {}", msg.content);
                let mut output: Vec<ParseResult> = parse_msg(&msg.content)?;
                fill_file_names(file_names, &mut output)?;
                return Ok(output);
            }
        }

        Ok(Vec::new())
    }

    fn name(&self) -> String {
        format!("deepbricks-{}", self.config.model)
    }

    fn max_file_name_length(&self) -> usize {
        10
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::time;

    #[tokio::test]
    async fn test_client_creation() -> Result<()> {
        dotenv::dotenv()?;
        let client = Client::from_env()?;

        let now = time::Instant::now();
        let resp = client
            .parse_file_names(vec![
                "[LoliHouse] 孤nsingle人的异世界攻略 / Hitoribocchi no Isekai Kouryaku - 12 [WebRip 1080p HEVC-10bit AAC][官方简繁内封字幕]".to_string(),
                "[LoliHouse] 孤nsingle人的异世界攻略 / Hitoribocchi no Isekai Kouryaku - 12 [WebRip 1080p HEVC-10bit AAC][简繁双语]".to_string(),
                "[LoliHouse] 孤nsingle人的异世界攻略 / Hitoribocchi no Isekai Kouryaku - 12 [WebRip 1080p HEVC-10bit AAC][简体中文]".to_string(),
                "[LoliHouse] 孤nsingle人的异世界攻略 / Hitoribocchi no Isekai Kouryaku - 12 [WebRip 1080p HEVC-10bit AAC][简日双语]".to_string(),
                "[Skymoon-Raws] 孤nsingle人的异世界攻略 / Hitoribocchi no Isekai Kouryaku - 12 [ViuTV][WEB-DL][1080p][AVC AAC] [BIG5]".to_string(),
            ])
            .await?;
        println!("{:?}", resp);
        println!("{}", now.elapsed().as_millis());
        Ok(())
    }
}
