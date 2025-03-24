use anyhow::Result;
use async_trait::async_trait;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::{PROMPT_TEMPLATE, ParseResult, Parser, fill_file_names, parse_msg};

#[derive(Debug, Serialize)]
pub struct ImageUrl {
    pub url: String,
    pub detail: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum Content {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    Image { image_url: ImageUrl },
}

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
    pub content: Vec<Content>,
}

#[derive(Debug, Serialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub format_type: String,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
    pub max_tokens: u32,
    pub stop: Vec<String>,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: u32,
    pub frequency_penalty: f32,
    pub n: u32,
    pub response_format: ResponseFormat,
}

#[derive(Debug, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
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
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub usage: Usage,
    pub created: u64,
    pub model: String,
    pub object: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: u32,
    pub max_tokens: u32,
    pub frequency_penalty: f32,
    pub n: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: String::from("https://api.siliconflow.cn/v1"),
            model: String::from("Qwen/Qwen2.5-7B-Instruct"),
            temperature: 0.0,
            top_p: 0.1,
            top_k: 1,
            max_tokens: 4096,
            frequency_penalty: 0.0,
            n: 1,
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
        let api_key = std::env::var("SILICONFLOW_API_KEY")?;
        let base_url = std::env::var("SILICONFLOW_BASE_URL")?;
        let model = std::env::var("SILICONFLOW_MODEL")?;
        Ok(Self::new(
            Config {
                api_key,
                base_url,
                model,
                ..Default::default()
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
            stream: false,
            max_tokens: self.config.max_tokens,
            stop: vec![],
            temperature: self.config.temperature,
            top_p: self.config.top_p,
            top_k: self.config.top_k,
            frequency_penalty: self.config.frequency_penalty,
            n: self.config.n,
            response_format: ResponseFormat {
                format_type: String::from("text"),
            },
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.config.base_url))
            .headers(self.build_headers())
            .json(&request)
            .send()
            .await?;

        let body = response.text().await?;
        debug!("{}", body);
        let response: ChatCompletionResponse =
            serde_json::from_str(&body).map_err(|_| anyhow::anyhow!("反序列化失败: {}", body))?;

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
                content: vec![Content::Text {
                    text: PROMPT_TEMPLATE.to_owned(),
                }],
            },
            Message {
                role: Role::User,
                content: vec![Content::Text { text: params }],
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
        format!("siliconflow-{}", self.config.model)
    }

    fn max_file_name_length(&self) -> usize {
        5
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::time;

    #[tokio::test]
    #[ignore]
    async fn test_client_creation() -> Result<()> {
        dotenv::dotenv().ok();
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_target(true)
            .init();
        let client = Client::from_env()?;

        let now = time::Instant::now();
        let resp = client.parse_file_names(vec![
        "[LoliHouse] 孤nsingle人的异世界攻略 / Hitoribocchi no Isekai Kouryaku - 12 [WebRip 1080p HEVC-10bit AAC][官方简繁内封字幕]".to_string(),
        "[LoliHouse] 孤nsingle人的异世界攻略 / Hitoribocchi no Isekai Kouryaku - 12 [WebRip 1080p HEVC-10bit AAC][简繁双语]".to_string(),
        "[LoliHouse] 孤nsingle人的异世界攻略 / Hitoribocchi no Isekai Kouryaku - 12 [WebRip 1080p HEVC-10bit AAC][简体中文]".to_string(),
        "[LoliHouse] 孤nsingle人的异世界攻略 / Hitoribocchi no Isekai Kouryaku - 12 [WebRip 1080p HEVC-10bit AAC][简日双语]".to_string(),
        "[Skymoon-Raws] 孤nsingle人的异世界攻略 / Hitoribocchi no Isekai Kouryaku - 12 [ViuTV][WEB-DL][1080p][AVC AAC] [BIG5]".to_string(),
        ]).await?;
        println!("{:?}", resp);
        println!("{}", now.elapsed().as_millis());
        Ok(())
    }
}
