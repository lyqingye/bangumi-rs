use std::str::FromStr;

use crate::utils::smart_parse_date;
use anyhow::Result;
use chrono::NaiveDateTime;
use reqwest::Url;
use scraper::Selector;
use tracing::{info, instrument};

lazy_static::lazy_static! {
    static ref TR_SELECTOR: Selector = Selector::parse("table > tbody > tr").unwrap();
    static ref TD_A_SELECTOR: Selector = Selector::parse("td > a").unwrap();
    static ref LINK_SELECTOR: Selector = Selector::parse("td > a").unwrap();
    static ref TITLE_SELECTOR: Selector = Selector::parse(
        "#sk-container > div.pull-left.leftbar-container > p.bangumi-title > a.w-other-c"
    ).unwrap();
    static ref SUB_GROUP_SELECTOR: Selector = Selector::parse(
        "#sk-container > div.pull-left.leftbar-container > p.bangumi-info > a.magnet-link-wrap"
    ).unwrap();
    static ref BUTTON_SELECTOR: Selector = Selector::parse(
        "#sk-container > div.pull-left.leftbar-container > div.leftbar-nav > a.episode-btn"
    ).unwrap();
    static ref BANGUMI_TV_LINK_SELECTOR: Selector = Selector::parse(
        "#sk-container > div.pull-left.leftbar-container > p.bangumi-info > a.w-other-c"
    ).unwrap();
    static ref TD_SELECTOR: Selector = Selector::parse("td").unwrap();

    static ref BANGUMI_POSTER_SELECTOR: Selector = Selector::parse("div.bangumi-poster").unwrap();
    static ref BANGUMI_TITLE_SELECTOR: Selector = Selector::parse("a.an-text").unwrap();
    static ref WEEK_BANGUMI_SELECTOR: Selector = Selector::parse("div.sk-bangumi").unwrap();
    static ref WEEK_BANGUMI_ITEM_SELECTOR: Selector = Selector::parse("ul.an-ul > li").unwrap();
    static ref WEEK_BANGUMI_INFO_SELECTOR: Selector = Selector::parse("div.an-info").unwrap();
    static ref WEEK_BANGUMI_IMAGE_SELECTOR: Selector = Selector::parse("span.js-expand_bangumi").unwrap();
    static ref WEEK_BANGUMI_AIR_DATE_SELECTOR: Selector = Selector::parse("div.date-text").unwrap();
    static ref WEEK_BANGUMI_SEASON_SELECTOR: Selector = Selector::parse("#sk-data-nav > div > ul.navbar-nav.date-select > li > div > div.sk-col.date-text").unwrap();
}

#[derive(Debug, Clone, Default)]
pub struct EpisodeItem {
    pub file_name: Option<String>,
    pub sub_group: Option<String>,
    pub pub_date: Option<NaiveDateTime>,
    pub magnet_link: String,
    pub info_hash: String,
    pub torrent_download_url: Option<Url>,
    pub file_size: usize,
}

impl EpisodeItem {
    pub fn validate(&self) -> bool {
        self.file_name.is_some()
            && self.torrent_download_url.is_some()
            && !self.info_hash.is_empty()
            && !self.magnet_link.is_empty()
    }
}

#[derive(Debug)]
pub struct BangumiInfo {
    pub bangumi_tv_id: Option<i32>,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Client {
    cli: reqwest::Client,
    endpoint: Url,
}

#[derive(Debug, Clone)]
pub struct Calendar {
    pub season: Option<String>,
    pub bangumis: Vec<MikanBangumi>,
}

#[derive(Debug, Clone)]
pub struct MikanBangumi {
    pub id: i32,
    pub title: Option<String>,
    pub weekday: i32,
    pub image_url: Option<String>,
    pub air_date: Option<NaiveDateTime>,
}
impl Client {
    pub fn new_with_client(cli: reqwest::Client, endpoint: &str) -> Result<Client> {
        Ok(Client {
            cli,
            endpoint: endpoint.parse()?,
        })
    }

    pub fn from_env() -> Result<Client> {
        let cli = reqwest::Client::new();
        let endpoint = std::env::var("MIKAN_ENDPOINT")?;
        Ok(Client {
            cli,
            endpoint: endpoint.parse()?,
        })
    }

    #[instrument(name = "爬取番剧种子信息", skip(self), fields(bangumi_id = %bangumi_id))]
    pub async fn collect_by_bangumi_id(&self, bangumi_id: i32) -> Result<Vec<EpisodeItem>> {
        let url = self
            .endpoint
            .join(format!("/Home/Bangumi/{}", bangumi_id).as_str())?;
        info!("url: {}", url);

        let html = self.cli.get(url).send().await?.text().await?;

        let document = scraper::Html::parse_document(&html);
        let tr_list: Vec<_> = document.select(&TR_SELECTOR).collect();
        let mut result = Vec::new();
        for tr in tr_list {
            let mut item = EpisodeItem::default();

            let links: Vec<_> = tr.select(&TD_A_SELECTOR).collect();
            for link in links {
                if let Some("magnet-link-wrap") = link.attr("class") {
                    item.file_name = Some(link.inner_html());
                }

                match link.attr("class") {
                    Some("magnet-link-wrap") => {
                        item.file_name = Some(link.inner_html());
                    }
                    Some("js-magnet magnet-link") => {
                        if let Some(magnet_link) = link.attr("data-clipboard-text") {
                            if magnet_link.starts_with("magnet:?xt=urn:btih:") {
                                item.magnet_link = magnet_link.to_string();
                                item.info_hash = magnet_link
                                    .split("xt=urn:btih:")
                                    .nth(1)
                                    .and_then(|s| s.split('&').next())
                                    .filter(|hash| hash.len() == 40)
                                    .map(|s| s.to_string())
                                    .unwrap_or_default();
                            }
                        }
                    }
                    _ => {}
                };

                if let Some(value) = link.attr("href") {
                    if value.starts_with("/Download") && value.ends_with(".torrent") {
                        if let Ok(uri) = self.endpoint.join(value) {
                            item.torrent_download_url = Some(uri);
                        }
                    }
                }
            }
            let td_list: Vec<_> = tr.select(&TD_SELECTOR).collect();
            for td in td_list {
                if let Ok(date) = smart_parse_date(td.inner_html().as_str()) {
                    item.pub_date = Some(date);
                    break;
                }

                match huby::ByteSize::from_str(td.inner_html().as_str()) {
                    Ok(size) => {
                        item.file_size = size.in_bytes() as usize;
                    }
                    _ => {}
                };
            }
            if item.validate() {
                result.push(item);
            }
        }
        Ok(result)
    }

    #[instrument(name = "爬取番剧信息", skip(self), fields(bangumi_id = %bangumi_id))]
    pub async fn get_bangumi_info(&self, bangumi_id: i32) -> Result<BangumiInfo> {
        let url = self
            .endpoint
            .join(format!("/Home/Bangumi/{}", bangumi_id).as_str())?;
        info!("url: {}", url);

        let search_result_page_html = self.cli.get(url).send().await?.text().await?;
        let document = scraper::Html::parse_document(&search_result_page_html);

        // 从 Bangumi 链接中提取 ID
        let bangumi_tv_id = document.select(&BANGUMI_TV_LINK_SELECTOR).find_map(|el| {
            el.attr("href")
                .filter(|href| href.contains("bgm.tv/subject/"))
                .and_then(|href| Self::extract_subject_id_from_link(href))
        });

        let image_url = document
            .select(&BANGUMI_POSTER_SELECTOR)
            .next()
            .and_then(|el| el.attr("style"))
            .and_then(|style| {
                // 从 style 属性中提取 url
                let start = style.find("url('").map(|i| i + 5)?;
                let end = style[start..].find("'").map(|i| i + start)?;
                Some(style[start..end].to_string())
            })
            .map(|url| {
                self.endpoint
                    .join(&url)
                    .map(|u| u.to_string())
                    .unwrap_or_default()
            });

        Ok(BangumiInfo {
            bangumi_tv_id,
            image_url,
        })
    }

    #[instrument(name = "获取番剧放映表", skip(self))]
    pub async fn get_calendar(&self) -> Result<Calendar> {
        info!("get week bangumi");
        let search_result_page_html = self
            .cli
            .get(self.endpoint.clone())
            .send()
            .await?
            .text()
            .await?;
        self.parse_home_page(search_result_page_html.as_str())
    }

    pub fn parse_home_page(&self, page_html_content: &str) -> Result<Calendar> {
        let document = scraper::Html::parse_document(page_html_content);
        let mut bangumis = Vec::new();
        let season = document
            .select(&WEEK_BANGUMI_SEASON_SELECTOR)
            .next()
            .map(|el| el.text().collect::<String>())
            .map(|s| s.trim().to_string());

        // 遍历每个星期的番剧区块
        for week_item in document.select(&WEEK_BANGUMI_SELECTOR) {
            // 获取星期几
            // 从 HTML 元素中获取星期几的值并解析为整数，解析失败则跳过当前项
            // 使用 and_then 链式调用来简化嵌套的 if let 结构
            let Some(weekday) = week_item
                .attr("data-dayofweek")
                .and_then(|day| day.parse::<i32>().ok())
            else {
                continue;
            };

            // 遍历该星期的所有番剧
            for bangumi_item in week_item.select(&WEEK_BANGUMI_ITEM_SELECTOR) {
                // 获取番剧ID和图片URL
                if let Some(img_span) = bangumi_item.select(&WEEK_BANGUMI_IMAGE_SELECTOR).next() {
                    let id = img_span
                        .attr("data-bangumiid")
                        .and_then(|id| id.parse::<i32>().ok())
                        .unwrap_or_default();

                    let image_url = img_span.attr("data-src").map(|src| {
                        self.endpoint
                            .join(src)
                            .map(|t| t.to_string())
                            .unwrap_or_default()
                    });

                    // 获取标题
                    let title = bangumi_item
                        .select(&BANGUMI_TITLE_SELECTOR)
                        .next()
                        .map(|el| el.text().collect::<String>());

                    let air_date = bangumi_item
                        .select(&WEEK_BANGUMI_AIR_DATE_SELECTOR)
                        .next()
                        .map(|el| el.text().collect::<String>())
                        .and_then(|date_text| {
                            smart_parse_date(&date_text.replace(" 更新", "")).ok()
                        });

                    if id > 0 {
                        bangumis.push(MikanBangumi {
                            id,
                            title,
                            weekday,
                            image_url,
                            air_date,
                        });
                    }
                }
            }
        }

        Ok(Calendar { season, bangumis })
    }

    fn extract_subject_id_from_link(link: &str) -> Option<i32> {
        // 从链接中提取 subject id
        link.split("subject/")
            .nth(1)
            .and_then(|id| id.parse::<i32>().ok())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn create_clinet() -> Result<Client> {
        dotenv::dotenv()?;
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_target(true) // 不显示目标模块
            .init();
        Client::from_env()
    }

    #[tokio::test]
    async fn test_search() -> Result<()> {
        let mikan = create_clinet()?;
        let result = mikan.collect_by_bangumi_id(3422).await?;
        println!("result: {:?}", result);
        Ok(())
    }

    #[tokio::test]
    async fn test_week_bangumi() -> Result<()> {
        let mikan = create_clinet()?;
        let result = mikan.get_calendar().await?;
        println!("result: {:?}", result);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_bangumi_info() -> Result<()> {
        let mikan = create_clinet()?;
        let result = mikan.get_bangumi_info(681).await?;
        println!("result: {:?}", result);
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_by_bangumi_id_with_info_hash() -> Result<()> {
        let mikan = create_clinet()?;
        let result = mikan.collect_by_bangumi_id(105).await?;
        println!("result: {:?}", result);
        Ok(())
    }

    #[tokio::test]
    async fn test_parser_bytes() -> Result<()> {
        let size = huby::ByteSize::from_str("992.7 MB").unwrap();
        println!("size: {:?}", size.in_bytes());
        Ok(())
    }
}
