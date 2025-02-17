use anyhow::{Context, Result};
use reqwest::{header::USER_AGENT, Client as ReqwestClient};
use tracing::{instrument, trace};

use super::model::{CalendarResponse, EpisodeList, EpisodeType, Subject};

#[derive(Debug, Clone)]
pub struct Client {
    base_url: String,
    cli: ReqwestClient,
}

const UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.0.0 Safari/537.36 Edg/108.0.1462.54";

impl Client {
    pub fn new_with_client(cli: ReqwestClient, base_url: &str) -> Result<Self> {
        Ok(Self {
            base_url: base_url.to_string(),
            cli,
        })
    }

    pub fn new_from_env() -> Result<Self> {
        let base_url = std::env::var("BANGUMI_TV_BASE_URL")?;
        Self::new_with_client(ReqwestClient::builder().user_agent(UA).build()?, &base_url)
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
        let resp: Vec<CalendarResponse> = serde_json::from_str(&response).with_context(|| response)?;
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
        let resp: EpisodeList = serde_json::from_str(&response).with_context(|| response)?;
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
        let resp: Subject = serde_json::from_str(&response).with_context(|| response)?;
        if resp.id != subject_id {
            return Ok(None);
        }
        Ok(Some(resp))
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
        let resp = cli.episodes(459435, EpisodeType::Normal, 100, 0).await?;
        println!("{:?}", resp);
        Ok(())
    }

    #[tokio::test]
    async fn test_subject() -> Result<()> {
        let cli = create_client().await?;
        let resp = cli.get_subject(525733).await?;
        println!("{:?}", resp);
        Ok(())
    }
}
