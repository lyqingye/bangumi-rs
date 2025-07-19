use anyhow::Result;
use anyhow::anyhow;
use async_trait::async_trait;
use reqwest::Response;
use reqwest::StatusCode;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info, warn};

use crate::{PROMPT_TEMPLATE, ParseResult, Parser, fill_file_names, parse_msg};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
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
    pub stop: Option<Vec<String>>,
    pub temperature: f32,
    pub top_p: f32,
    pub frequency_penalty: f32,
    pub presence_penalty: f32,
    pub response_format: ResponseFormat,
    pub stream_options: Option<serde_json::Value>,
    pub tools: Option<serde_json::Value>,
    pub tool_choice: String,
    pub logprobs: bool,
    pub top_logprobs: Option<serde_json::Value>,
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

#[derive(Debug, Clone)]
pub struct Config {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: u32,
    pub frequency_penalty: f32,
    pub presence_penalty: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: String::from("https://api.deepseek.com/v1"),
            model: String::from("deepseek-chat"),
            temperature: 0.1,
            top_p: 0.1,
            max_tokens: 8192,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
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
        let api_key = std::env::var("DEEPSEEK_API_KEY")?;
        let base_url = std::env::var("DEEPSEEK_BASE_URL")?;
        let model = std::env::var("DEEPSEEK_MODEL")?;
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

    async fn handle_error_response(
        resp: Response,
        retry_count: u32,
        max_retries: u32,
        base_delay: Duration,
    ) -> Result<Option<Duration>> {
        match resp.status() {
            StatusCode::BAD_REQUEST => {
                let err_msg = resp.text().await.unwrap_or_default();
                error!("请求格式错误: {}", err_msg);
                Err(anyhow!("请求格式错误: {}", err_msg))
            }
            StatusCode::UNAUTHORIZED => {
                error!("API 认证失败，请检查 API Key 是否正确");
                Err(anyhow!("API 认证失败，请检查 API Key 是否正确"))
            }
            StatusCode::PAYMENT_REQUIRED => {
                error!("账户余额不足，请充值后重试");
                Err(anyhow!("账户余额不足，请充值后重试"))
            }
            StatusCode::UNPROCESSABLE_ENTITY => {
                let err_msg = resp.text().await.unwrap_or_default();
                error!("参数错误: {}", err_msg);
                Err(anyhow!("参数错误: {}", err_msg))
            }
            status @ (StatusCode::TOO_MANY_REQUESTS | StatusCode::SERVICE_UNAVAILABLE) => {
                if retry_count >= max_retries {
                    let msg = if status == StatusCode::TOO_MANY_REQUESTS {
                        "请求速率超限"
                    } else {
                        "服务器繁忙"
                    };
                    error!("{}，重试次数已达上限", msg);
                    return Err(anyhow!("{}，重试次数已达上限", msg));
                }
                let delay = base_delay * 2u32.pow(retry_count);
                warn!(
                    "{}，{} 秒后重试",
                    if status == StatusCode::TOO_MANY_REQUESTS {
                        "请求速率超限"
                    } else {
                        "服务器繁忙"
                    },
                    delay.as_secs()
                );
                Ok(Some(delay))
            }
            StatusCode::INTERNAL_SERVER_ERROR => {
                if retry_count >= max_retries {
                    error!("服务器内部错误，重试次数已达上限");
                    return Err(anyhow!("服务器内部错误，重试次数已达上限"));
                }
                warn!("服务器内部错误，{} 秒后重试", base_delay.as_secs());
                Ok(Some(base_delay))
            }
            status => {
                error!("未知错误，状态码: {}", status);
                Err(anyhow!("未知错误，状态码: {}", status))
            }
        }
    }

    pub async fn chat_completion(&self, messages: Vec<Message>) -> Result<ChatCompletionResponse> {
        let max_retries = 3;
        let mut retry_count = 0;
        let base_delay = Duration::from_secs(1);

        loop {
            let request = ChatCompletionRequest {
                model: self.config.model.clone(),
                messages: messages.clone(),
                stream: false,
                max_tokens: self.config.max_tokens,
                stop: None,
                temperature: self.config.temperature,
                top_p: self.config.top_p,
                frequency_penalty: self.config.frequency_penalty,
                presence_penalty: self.config.presence_penalty,
                response_format: ResponseFormat {
                    format_type: String::from("json_object"),
                },
                stream_options: None,
                tools: None,
                tool_choice: String::from("none"),
                logprobs: false,
                top_logprobs: None,
            };

            match self
                .client
                .post(format!("{}/chat/completions", self.config.base_url))
                .headers(self.build_headers())
                .json(&request)
                .send()
                .await
            {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let body = resp.text().await?;
                        debug!("{}", body);
                        let response: ChatCompletionResponse = serde_json::from_str(&body)
                            .map_err(|_| anyhow::anyhow!("反序列化失败: {}", body))?;
                        return Ok(response);
                    }

                    if let Some(delay) =
                        Self::handle_error_response(resp, retry_count, max_retries, base_delay)
                            .await?
                    {
                        tokio::time::sleep(delay).await;
                        retry_count += 1;
                        continue;
                    }
                }
                Err(e) => {
                    if retry_count >= max_retries {
                        error!("网络请求失败，重试次数已达上限: {}", e);
                        return Err(anyhow!("网络请求失败: {}", e));
                    }
                    let delay = base_delay * 2u32.pow(retry_count);
                    warn!("网络请求失败，{} 秒后重试: {}", delay.as_secs(), e);
                    tokio::time::sleep(delay).await;
                    retry_count += 1;
                    continue;
                }
            }
        }
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
        format!("deepseek-{}", self.config.model)
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
        dotenv::dotenv()?;
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_target(true)
            .init();
        let client = Client::from_env()?;

        let now = time::Instant::now();
        let resp = client
            .parse_file_names(vec![
                "[ANBU]_Princess_Lover!_-_01_[2048A39A].mkv".to_string(),
            ])
            .await?;
        println!("{resp:?}");
        println!("{}", now.elapsed().as_millis());
        Ok(())
    }
}
