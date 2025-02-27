use anyhow::{Context, Result};
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
    async fn test_search() -> Result<()> {
        let cli = create_client().await?;
        let resp = cli.search("我独自升级", 1).await?;
        println!("找到 {} 个结果", resp.len());

        for (i, item) in resp.iter().enumerate() {
            println!("{}. {}", i + 1, item.title);
            println!("   发布时间: {}", item.pub_date);
            println!("   种子链接: {}", item.get_torrent_url());
            println!();
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_search_all() -> Result<()> {
        let cli = create_client().await?;
        let resp = cli.search_all("我独自升级").await?;
        println!("找到 {} 个结果", resp.len());
        Ok(())
    }

    #[tokio::test]
    async fn test_parse_torrent() -> Result<()> {
        let cli = create_client().await?;
        let resp = cli.search("我独自升级", 1).await?;
        let torrent_url = resp.first().unwrap().get_torrent_url();
        let bytes = cli.download_torrent(torrent_url).await?;
        println!("bytes: {:?}", bytes.len());
        Ok(())
    }
}
