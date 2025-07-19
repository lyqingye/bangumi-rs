use anyhow::Result;
use async_trait::async_trait;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::{PROMPT_TEMPLATE, ParseResult, Parser, fill_file_names, parse_msg};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct FormatProperty {
    pub r#type: String,
}

#[derive(Debug, Serialize)]
pub struct Options {
    pub temperature: f32,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
    pub format: Option<FormatProperty>,
    pub options: Option<Options>,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub message: Message,
    pub model: String,
    pub created_at: String,
    pub done: bool,
    pub done_reason: String,
    pub total_duration: u64,
    pub load_duration: u64,
    pub prompt_eval_count: u32,
    pub prompt_eval_duration: u64,
    pub eval_count: u32,
    pub eval_duration: u64,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: String,
    pub model: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_url: String::from("http://localhost:11434"),
            model: String::from("qwen2.5-coder:7b"),
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
        let base_url = std::env::var("OLLAMA_BASE_URL")?;
        let model = std::env::var("OLLAMA_MODEL")?;
        Ok(Self::new(
            Config { base_url, model },
            reqwest::Client::new(),
        ))
    }

    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers
    }

    pub async fn chat_completion(&self, messages: Vec<Message>) -> Result<ChatCompletionResponse> {
        let request = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages,
            stream: false,
            format: None,
            options: Some(Options { temperature: 0.0 }),
        };

        let response: ChatCompletionResponse = self
            .client
            .post(format!("{}/api/chat", self.config.base_url))
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
                role: "system".to_string(),
                content: PROMPT_TEMPLATE.to_owned(),
            },
            Message {
                role: "user".to_string(),
                content: params,
            },
        ];
        let response = self.chat_completion(messages).await?;
        let mut output: Vec<ParseResult> = parse_msg(&response.message.content)?;
        fill_file_names(file_names, &mut output)?;
        Ok(output)
    }

    fn name(&self) -> String {
        format!("ollama-{}", self.config.model)
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
        let client = Client::from_env()?;
        let now = time::Instant::now();

        let resp = client.parse_file_names(vec![
        "[ANBU]_Princess_Lover!_-_01_[2048A39A].mkv".to_string(),
        "[ANBU-Menclave]_Canaan_-_01_[1024x576_H.264_AAC][12F00E89].mkv".to_string(),
        "[52wy][SlamDunk][001][Jpn_Chs_Cht][x264_aac][DVDRip][7FE2C873].mkv".to_string(),
        "[Yameii] NieR Automata Ver1.1a - S01E10 [English Dub] [CR WEB-DL 1080p] [3703AD3A]".to_string(),
        "[LoliHouse] 孤nsingle人的异世界攻略 / Hitoribocchi no Isekai Kouryaku - 12 [WebRip 1080p HEVC-10bit AAC][简繁内封字幕]".to_string(),
        "[Skymoon-Raws] 孤nsingle人的异世界攻略 / Hitoribocchi no Isekai Kouryaku - 12 [ViuTV][WEB-DL][1080p][AVC AAC]".to_string(),
        "[ANi] Loner Life in Another World / 孤nsingle人的异世界攻略 - 12 [1080P][Baha][WEB-DL][AAC AVC][简日内嵌][MP4]".to_string(),
        "【幻樱字幕组】【1月新番】【魔法制造者 ~异世界魔法的制作方法~ Magic Maker ~Isekai Mahou no Tsukurikata~】【04】【BIG5】【1920X1080】".to_string()
        ]).await?;

        println!("{resp:?}");
        println!("{}", now.elapsed().as_millis());
        Ok(())
    }
}
