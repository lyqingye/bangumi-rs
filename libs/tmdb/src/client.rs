use anyhow::Result;
use chrono::NaiveDate;
use lazy_static::lazy_static;
use regex;
use reqwest::Url;
use std::{path::Path, sync::Arc};
use tmdb_api::{
    client::reqwest::ReqwestExecutor,
    prelude::Command,
    tvshow::{details::TVShowDetails, search::TVShowSearch, SeasonShort, TVShow},
};
use tracing::instrument;

#[derive(Clone)]
pub struct Client {
    client: Arc<tmdb_api::Client<ReqwestExecutor>>,
    language: String,
    image_base_url: Url,
    http_client: reqwest::Client,
}

lazy_static! {
    static ref PATTERNS: [regex::Regex; 5] = [
        regex::Regex::new(r"\s*第\s*[一二三四五六七八九十\d]+\s*季\s*$").unwrap(),
        regex::Regex::new(r"\s*[Ss]\s*\d+\s*$").unwrap(),
        regex::Regex::new(r"\s*[Ss]eason\s*\d+\s*$").unwrap(),
        regex::Regex::new(r"\s*SEASON\s*\d+\s*$").unwrap(),
        regex::Regex::new(r"\s*[Ss]eason\s*\d+\s*$").unwrap(),
    ];
}

fn extract_anime_name(name: &str) -> String {
    // 移除常见的季数标识
    let name = name.trim();
    let mut result = name.to_string();

    for pattern in PATTERNS.iter() {
        result = pattern.replace(&result, "").to_string();
    }
    result.trim().to_string()
}

impl Client {
    pub fn new(
        client: reqwest::Client,
        api_key: &str,
        base_url: &str,
        image_base_url: &str,
        language: &str,
    ) -> Result<Self> {
        let tmdb = tmdb_api::Client::builder()
            .with_api_key(api_key.to_string())
            .with_executor(client.clone().into())
            .with_base_url(base_url.to_string())
            .build()?;
        Ok(Self {
            client: Arc::new(tmdb),
            language: language.to_string(),
            image_base_url: Url::parse(image_base_url)?,
            http_client: client,
        })
    }

    pub fn new_from_env() -> Result<Self> {
        let api_key = std::env::var("TMDB_API_KEY")?;
        let base_url = std::env::var("TMDB_BASE_URL")?;
        let image_base_url = std::env::var("TMDB_IMAGE_BASE_URL")?;
        let language = std::env::var("TMDB_LANGUAGE")?;
        let client = reqwest::Client::new();
        Self::new(client, &api_key, &base_url, &image_base_url, &language)
    }

    #[instrument(name = "TMDB 匹配番剧", skip(self), fields(name = %name))]
    pub async fn match_bangumi(
        &self,
        name: &str,
        air_date: Option<NaiveDate>,
    ) -> Result<Option<(TVShow, SeasonShort)>> {
        let clean_name = extract_anime_name(name);
        // 1. 执行搜索
        let search_results = TVShowSearch::new(clean_name)
            .with_language(Some(self.language.clone()))
            .execute(&self.client)
            .await
            .map_err(|e| anyhow::anyhow!("TMDB搜索失败: {}", e))?;

        // 2. 如果有air_date就按日期匹配，否则取第一个结果
        let tv_id = if let Some(air_date) = air_date {
            let mut candidates = Vec::new();
            for result in search_results.results {
                if let Some(first_air_date) = result.inner.first_air_date {
                    let days_diff = (first_air_date.signed_duration_since(air_date))
                        .num_days()
                        .abs();
                    candidates.push((result.inner.id, days_diff));
                }
            }
            candidates
                .iter()
                .min_by_key(|(_, diff)| *diff)
                .map(|(id, _)| *id)
        } else {
            search_results.results.first().map(|r| r.inner.id)
        };

        let Some(tv_id) = tv_id else {
            return Ok(None);
        };

        // 4. 获取剧集详情
        let details = TVShowDetails::new(tv_id)
            .with_language(Some(self.language.clone()))
            .execute(&self.client)
            .await
            .map_err(|e| anyhow::anyhow!("获取详情失败: {}", e))?;

        // 5. 如果有air_date就按日期匹配季数，否则取第一季
        let matched_season = if air_date.is_some() {
            details
                .seasons
                .iter()
                .filter_map(|season| {
                    season.inner.air_date.map(|date| {
                        let days_diff = (date.signed_duration_since(air_date.unwrap()))
                            .num_days()
                            .abs();
                        (season, days_diff)
                    })
                })
                .min_by_key(|(_, diff)| *diff)
                .map(|(season, _)| season)
        } else {
            details.seasons.first()
        };

        Ok(matched_season.map(|season| (details.clone(), season.clone())))
    }

    pub async fn download_image(&self, file_path: &str, path: impl AsRef<Path>) -> Result<()> {
        let base = self.image_base_url.as_str().trim_end_matches('/');
        let file_path = file_path.trim_start_matches('/');
        let url = format!("{}/{}", base, file_path);
        let response = self.http_client.get(url).send().await?;
        let bytes = response.bytes().await?;
        tokio::fs::write(path, bytes).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn test_tmdb_search_movie() -> Result<()> {
        dotenv::dotenv()?;
        let tmdb = Client::new_from_env()?;
        let test_date = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap(); // 进击的巨人首播日期
        let rs = tmdb
            .match_bangumi("想变成猫的田万川君", Some(test_date))
            .await?
            .unwrap();
        println!("{:?}", rs);
        Ok(())
    }

    #[test]
    fn test_extract_anime_name() {
        assert_eq!(extract_anime_name("进击的巨人 第四季"), "进击的巨人");
        assert_eq!(extract_anime_name("SPY×FAMILY Season2"), "SPY×FAMILY");
        assert_eq!(extract_anime_name("鬼灭之刃 S2"), "鬼灭之刃");
        assert_eq!(extract_anime_name("咒术回战season 2"), "咒术回战");
        assert_eq!(extract_anime_name("BLEACH S1"), "BLEACH");
        assert_eq!(extract_anime_name("进击的巨人"), "进击的巨人");
    }
}
