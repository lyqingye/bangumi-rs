use anyhow::{Context, Result};
use regex::Regex;
use reqwest::{Client as ReqwestClient, Url};
use tracing::instrument;

use crate::model::RssItem;

use super::model::RssResponse;

#[derive(Debug, Clone)]
pub struct Client {
    base_url: String,
    cli: ReqwestClient,
}

impl Client {
    pub fn new_with_client(cli: ReqwestClient, base_url: &str) -> Result<Self> {
        Ok(Self {
            base_url: base_url.to_string(),
            cli,
        })
    }

    pub fn new_from_env() -> Result<Self> {
        let base_url = std::env::var("XML_RSS_BASE_URL")?;
        Self::new_with_client(ReqwestClient::builder().build()?, &base_url)
    }

    pub fn new() -> Result<Self> {
        Self::new_with_client(ReqwestClient::builder().build()?, "https://acgrip.art")
    }

    /// 搜索资源
    ///
    /// # 参数
    ///
    /// * `keyword` - 搜索关键词
    #[instrument(name = "搜索资源", skip(self), fields(keyword = %keyword))]
    pub async fn search(&self, keyword: &str) -> Result<Vec<RssItem>> {
        let url = Url::parse_with_params(
            &format!("{}/topics/rss/rss.xml", self.base_url),
            &[("keyword", keyword)],
        )?;

        let response = self.cli.get(url).send().await?.text().await?;

        let resp: RssResponse = quick_xml::de::from_str(&response)
            .with_context(|| format!("解析 RSS 响应失败: {}", response))?;

        Ok(resp.channel.item)
    }

    // TODO 如果文件名中有具体某一季，那么可以优先按这个关键字搜索，否则按照去掉季后的关键字搜索
    pub async fn search_with_bangumi_tv_id(
        &self,
        keyword: &str,
        bangumi_tv_id: i32,
    ) -> Result<Vec<RssItem>> {
        let result = self.search(keyword).await?;
        for item in result {
            // 该字幕组发布的资源，会带有bgm.tv的链接
            if item.title.contains("Up to 21°C") {
                let bgm_tv_id = self.search_bangumi_tv_id_in_link(&item.link).await?;
                if bgm_tv_id == Some(bangumi_tv_id) {
                    return Ok(vec![item]);
                }
            }
        }
        Ok(vec![])
    }

    async fn search_bangumi_tv_id_in_link(&self, link: &str) -> Result<Option<i32>> {
        let url = Url::parse(link)?;
        let html = self.cli.get(url).send().await?.text().await?;

        // 匹配 bangumi_tv_id=数字 的模式
        let re = Regex::new(r"bangumi_tv_id=(\d+)").unwrap();
        let caps = re.captures(&html);

        // 如果没有找到第一种模式，尝试匹配 bgm.tv/subject/数字 的模式
        if caps.is_none() {
            let bgm_re = Regex::new(r"bgm\.tv/subject/(\d+)").unwrap();
            let bgm_caps = bgm_re.captures(&html);
            return Ok(bgm_caps.and_then(|c| c[1].parse::<i32>().ok()));
        }

        Ok(caps.and_then(|c| c[1].parse::<i32>().ok()))
    }

    /// 下载种子文件
    pub async fn download_torrent(&self, torrent_url: &str) -> Result<Vec<u8>> {
        let url = if torrent_url.starts_with("http") {
            torrent_url.to_string()
        } else {
            format!("{}{}", self.base_url, torrent_url)
        };

        let response = self.cli.get(url).send().await?;
        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use anyhow::Result;
    use raw_parser::parser::Parser;

    async fn create_client() -> Result<Client> {
        dotenv::dotenv().ok();
        let cli = if std::env::var("XML_RSS_BASE_URL").is_ok() {
            Client::new_from_env()?
        } else {
            Client::new()?
        };
        Ok(cli)
    }

    #[tokio::test]
    #[ignore]
    async fn test_search() -> Result<()> {
        let cli = create_client().await?;
        let resp = cli.search("我的青春恋爱物语果然有问题").await?;
        println!("找到 {} 个结果", resp.len());

        let parser = Parser::new();
        let mut group_by_season = HashMap::new();
        for item in resp.iter() {
            let result = parser.parse(&item.title);
            if let Ok(result) = result {
                if let Some(season) = result.season {
                    group_by_season
                        .entry(season)
                        .or_insert(Vec::new())
                        .push((item.title.clone(), result));
                } else {
                    group_by_season
                        .entry(1)
                        .or_insert(Vec::new())
                        .push((item.title.clone(), result));
                }
            }
        }
        for (season, items) in group_by_season {
            println!("第 {} 季", season);
            for (title, result) in items {
                println!(
                    "     \t{:?} ------>\t {:?} \tSEASON: {:?} \tEPISODE: {:?}",
                    title, result.name_zh, result.season, result.episode
                );
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_search_with_bangumi_tv_id() -> Result<()> {
        let cli = create_client().await?;
        let resp = cli
            .search_bangumi_tv_id_in_link("https://acgrip.art/t/324676")
            .await?;
        assert_eq!(resp, Some(471793));
        Ok(())
    }
}
