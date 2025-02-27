use std::path::Path;

use anyhow::{Context, Result};
use reqwest::{header::USER_AGENT, Client as ReqwestClient, Url};
use tracing::instrument;

use super::model::{CalendarResponse, EpisodeList, EpisodeType, Subject};

#[derive(Debug, Clone)]
pub struct Client {
    base_url: String,
    image_base_url: String,
    cli: ReqwestClient,
}

/// ref: https://github.com/bangumi/api/blob/master/docs-raw/user%20agent.md
const UA: &str = "lyqingye/bangumi-rs";

impl Client {
    pub fn new_with_client(
        cli: ReqwestClient,
        base_url: &str,
        image_base_url: &str,
    ) -> Result<Self> {
        Ok(Self {
            base_url: base_url.to_string(),
            image_base_url: image_base_url.to_string(),
            cli,
        })
    }

    pub fn new_from_env() -> Result<Self> {
        let base_url = std::env::var("BANGUMI_TV_BASE_URL")?;
        let image_base_url = std::env::var("BANGUMI_TV_IMAGE_BASE_URL")?;
        Self::new_with_client(
            ReqwestClient::builder().user_agent(UA).build()?,
            &base_url,
            &image_base_url,
        )
    }

    #[instrument(name = "获取放送列表")]
    pub async fn get_calendar(&self) -> Result<Vec<CalendarResponse>> {
        let url = format!("{}/calendar", self.base_url);
        let response = self
            .cli
            .get(&url)
            .header(USER_AGENT, UA)
            .send()
            .await?
            .text()
            .await?;
        let resp: Vec<CalendarResponse> = serde_json::from_str(&response)
            .with_context(|| format!("解析放送列表失败: {}", response))?;
        Ok(resp)
    }

    #[instrument(name = "获取剧集信息", skip(self), fields(subject_id = %subject_id))]
    pub async fn episodes(
        &self,
        subject_id: i32,
        ep_type: EpisodeType,
        limit: i32,
        offset: i32,
    ) -> Result<EpisodeList> {
        let response = self
            .cli
            .get(format!("{}/v0/episodes", self.base_url))
            .header(USER_AGENT, UA)
            .query(&[
                ("subject_id", subject_id),
                ("type", ep_type as i32),
                ("limit", limit),
                ("offset", offset),
            ])
            .send()
            .await?
            .text()
            .await?;
        let resp: EpisodeList = serde_json::from_str(&response)
            .with_context(|| format!("解析剧集信息失败: {}", response))?;
        Ok(resp)
    }

    #[instrument(name = "获取番剧信息", skip(self), fields(subject_id = %subject_id))]
    pub async fn get_subject(&self, subject_id: i32) -> Result<Option<Subject>> {
        let response = self
            .cli
            .get(format!("{}/v0/subjects/{}", self.base_url, subject_id))
            .header(USER_AGENT, UA)
            .send()
            .await?
            .text()
            .await?;
        let resp: Subject = serde_json::from_str(&response)
            .with_context(|| format!("解析番剧信息失败: {}", response))?;
        Ok(Some(resp))
    }

    pub async fn download_image(&self, file_path: &str, path: impl AsRef<Path>) -> Result<()> {
        let base = self.image_base_url.as_str().trim_end_matches('/');
        let url = Url::parse(file_path)?;
        let proxyed_url = format!("{}{}", base, url.path());
        let response = self.cli.get(proxyed_url).send().await?;
        let bytes = response.bytes().await?;
        tokio::fs::write(path, bytes).await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;

    async fn create_client() -> Result<Client> {
        dotenv::dotenv()?;
        let cli = Client::new_from_env()?;
        Ok(cli)
    }

    #[tokio::test]
    async fn test_calendar() -> Result<()> {
        let cli = create_client().await?;
        let out = cli.get_calendar().await?;
        println!("{:?}", out);
        Ok(())
    }

    #[tokio::test]
    async fn test_episodes() -> Result<()> {
        let cli = create_client().await?;
        let resp = cli.episodes(475354, EpisodeType::Normal, 100, 0).await?;
        println!("{:?}", resp);
        Ok(())
    }

    #[tokio::test]
    async fn test_subject() -> Result<()> {
        let cli = create_client().await?;
        let resp = cli.get_subject(475354).await?;
        println!("{:?}", resp);
        Ok(())
    }

    #[tokio::test]
    async fn test_download_image() -> Result<()> {
        let cli = create_client().await?;
        cli.download_image(
            "http://lain.bgm.tv/pic/cover/l/37/17/455626_6hH1b.jpg",
            "/Users/lyqingye/Desktop/test.jpg",
        )
        .await?;
        Ok(())
    }
}
