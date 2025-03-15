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
        let base_url = std::env::var("ACGRIP_BASE_URL")?;
        Self::new_with_client(ReqwestClient::builder().build()?, &base_url)
    }

    pub fn new() -> Result<Self> {
        Self::new_with_client(ReqwestClient::builder().build()?, "https://acgrip.art")
    }

    /// 搜索资源
    ///
    /// # 参数
    ///
    /// * `term` - 搜索关键词
    /// * `page` - 页码，从 1 开始
    #[instrument(name = "搜索资源", skip(self), fields(term = %term, page = %page))]
    pub async fn search(&self, term: &str, page: u32) -> Result<Vec<RssItem>> {
        let url = Url::parse_with_params(
            &format!("{}/page/{}.xml", self.base_url, page),
            &[("term", term)],
        )?;

        let response = self.cli.get(url).send().await?.text().await?;

        let resp: RssResponse = quick_xml::de::from_str(&response)
            .with_context(|| format!("解析 RSS 响应失败: {}", response))?;

        Ok(resp.channel.item)
    }

    pub async fn search_all(&self, term: &str) -> Result<Vec<RssItem>> {
        let mut page = 1;
        let mut items = Vec::new();
        loop {
            let resp = self.search(term, page).await?;
            if resp.is_empty() {
                break;
            } else {
                items.extend(resp);
            }
            page += 1;
        }
        Ok(items)
    }

    // TODO 如果文件名中有具体某一季，那么可以优先按这个关键字搜索，否则按照去掉季后的关键字搜索
    pub async fn search_with_bangumi_tv_id(
        &self,
        term: &str,
        bangumi_tv_id: i32,
    ) -> Result<Vec<RssItem>> {
        let result = self.search_all(term).await?;
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
    use super::*;
    use anyhow::Result;

    async fn create_client() -> Result<Client> {
        dotenv::dotenv().ok();
        let cli = if std::env::var("ACGRIP_BASE_URL").is_ok() {
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
        let resp = cli.search("我独自升级", 1).await?;
        println!("找到 {} 个结果", resp.len());

        for (i, item) in resp.iter().enumerate() {
            println!("{}. {}", i + 1, item.title);
            println!("   发布时间: {}", item.get_pub_date().unwrap());
            println!("   种子链接: {}", item.get_torrent_url());
            println!();
        }

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_search_all() -> Result<()> {
        let cli = create_client().await?;
        let resp = cli.search_all("我独自升级 第二季").await?;
        println!("找到 {} 个结果", resp.len());
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_parse_torrent() -> Result<()> {
        let cli = create_client().await?;
        let resp = cli.search("我独自升级", 1).await?;
        let torrent_url = resp.first().unwrap().get_torrent_url();
        let bytes = cli.download_torrent(torrent_url).await?;
        println!("bytes: {:?}", bytes.len());
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
