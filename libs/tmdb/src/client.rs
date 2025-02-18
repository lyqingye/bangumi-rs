use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use lazy_static::lazy_static;
use regex;
use reqwest::Url;
use std::{path::Path, sync::Arc};
use tmdb_api::{
    client::reqwest::ReqwestExecutor,
    prelude::Command,
    tvshow::{
        details::TVShowDetails, episode::details::TVShowEpisodeDetails, search::TVShowSearch,
        SeasonShort, TVShow, TVShowShort,
    },
};
use tracing::{debug, instrument};

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

    let index = result.find(" ");
    if let Some(index) = index {
        result = result[..index].to_string();
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

    async fn try_match_tv_show(
        &self,
        air_date: NaiveDate,
        tv_shows: &Vec<TVShowShort>,
    ) -> Result<Vec<TVShowShort>> {
        let mut tv_candidates = Vec::new();
        for result in tv_shows {
            debug!(tv_name = %result.inner.name, air_date = %air_date);
            if let Some(first_air_date) = result.inner.first_air_date {
                // 检查年月是否完全匹配
                if first_air_date.year() == air_date.year()
                    && first_air_date.month() == air_date.month()
                {
                    tv_candidates.push(result.clone());
                    debug!(
                        tv_name = %result.inner.name,
                        tv_date = %first_air_date,
                        "找到年月完全匹配的TV"
                    );
                }
            }
        }
        Ok(tv_candidates)
    }

    async fn try_match_seasons(
        &self,
        seasons: &Vec<SeasonShort>,
        air_date: NaiveDate,
    ) -> Result<Option<SeasonShort>> {
        for season in seasons {
            debug!(
                season_name = %season.inner.name,
                season_air_date = %season.inner.air_date.unwrap_or_default()
            );
            if let Some(season_air_date) = season.inner.air_date {
                if season_air_date.year() == air_date.year()
                    && season_air_date.month() == air_date.month()
                {
                    return Ok(Some(season.clone()));
                }
            }
        }
        Ok(None)
    }

    #[instrument(name = "TMDB 匹配番剧", skip(self), fields(name = %name))]
    pub async fn match_bangumi(
        &self,
        name: &str,
        air_date: Option<NaiveDate>,
    ) -> Result<Option<(TVShow, SeasonShort)>> {
        let clean_name = extract_anime_name(name);
        debug!(name = %clean_name, "清理后的番剧名称");

        // 1. 执行搜索
        let search_results = TVShowSearch::new(clean_name)
            .with_language(Some(self.language.clone()))
            .execute(&self.client)
            .await
            .map_err(|e| anyhow::anyhow!("TMDB搜索失败: {}", e))?;

        debug!(results_count = %search_results.results.len(), "搜索结果数量");

        if search_results.results.is_empty() {
            debug!("未找到匹配的结果");
            return Ok(None);
        }

        // 如果没有放送时间，直接返回第一个结果的第一季
        if air_date.is_none() {
            debug!("未提供放送时间，使用第一个搜索结果");

            let tv = &search_results.results[0].inner;
            let details = TVShowDetails::new(tv.id)
                .with_language(Some(self.language.clone()))
                .execute(&self.client)
                .await
                .map_err(|e| anyhow::anyhow!("获取详情失败: {}", e))?;

            return Ok(details
                .seasons
                .first()
                .map(|season| (details.clone(), season.clone())));
        }

        let air_date = air_date.unwrap();

        // 1. 匹配TV
        debug!("尝试匹配TV: {}", air_date);
        let mut tv_shows = self
            .try_match_tv_show(air_date, &search_results.results)
            .await?;
        if tv_shows.is_empty() {
            debug!("未找到匹配的TV，使用所有搜索结果");
            tv_shows = search_results.results;
        }

        // 2. 匹配Season
        debug!("尝试匹配Season: {}", air_date);
        let mut tv_details_cache = Vec::new();
        for tv in tv_shows {
            let details = TVShowDetails::new(tv.inner.id)
                .with_language(Some(self.language.clone()))
                .execute(&self.client)
                .await
                .map_err(|e| anyhow::anyhow!("获取详情失败: {}", e))?;

            tv_details_cache.push(details.clone());

            let season = self.try_match_seasons(&details.seasons, air_date).await?;
            if let Some(season) = season {
                return Ok(Some((details.clone(), season.clone())));
            }
        }

        // 3. 匹配Episode
        debug!("尝试匹配Episode: {}", air_date);
        for tv in tv_details_cache {
            for season in tv.seasons.iter() {
                for i in 1..=season.episode_count {
                    let episode =
                        TVShowEpisodeDetails::new(tv.inner.id, season.inner.season_number, i)
                            .with_language(Some(self.language.clone()))
                            .execute(&self.client)
                            .await
                            .map_err(|e| anyhow::anyhow!("获取详情失败: {}", e))?;

                    debug!(
                        episode_name = %episode.inner.name,
                        episode_number = %episode.inner.episode_number,
                        episode_air_date = %episode.inner.air_date
                    );
                    if episode.inner.air_date.year() == air_date.year()
                        && episode.inner.air_date.month() == air_date.month()
                    {
                        return Ok(Some((tv.clone(), season.clone())));
                    }
                }
            }
        }

        debug!("未找到匹配的结果");
        Ok(None)
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
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_target(true) // 不显示目标模块
            .init();
        let tmdb = Client::new_from_env()?;
        let test_date = NaiveDate::from_ymd_opt(2025, 1, 4).unwrap(); // 进击的巨人首播日期
        let rs = tmdb
            .match_bangumi("我独自升级 第二季 -起于暗影-", Some(test_date))
            .await?
            .unwrap();
        println!("tv: {}", rs.0.inner.name);
        println!("season: {} {}", rs.1.inner.name, rs.1.inner.season_number);
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
        assert_eq!(
            extract_anime_name("期待在地下城邂逅有错吗 第五季 丰饶的女神篇"),
            "期待在地下城邂逅有错吗"
        );
    }
}
