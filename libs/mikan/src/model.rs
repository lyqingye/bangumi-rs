use anyhow::Result;
use chrono::NaiveDateTime;
use quick_xml::de::from_str;
use serde::{Deserialize, Serialize};
use utils::date::smart_parse_date;

/// Mikan RSS 根元素
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MikanRss {
    #[serde(rename = "@version")]
    pub version: String,
    pub channel: Channel,
}

/// RSS 频道信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Channel {
    pub title: String,
    pub link: String,
    pub description: String,
    #[serde(rename = "item", default)]
    pub items: Vec<Item>,
}

/// RSS 条目，表示一个种子
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Item {
    #[serde(rename = "guid")]
    pub guid: Guid,
    pub link: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "torrent")]
    pub torrent: Option<Torrent>,
    pub enclosure: Option<Enclosure>,
}

/// GUID 标识
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Guid {
    #[serde(rename = "@isPermaLink")]
    pub is_perma_link: String,
    #[serde(rename = "$value")]
    pub value: String,
}

/// 种子信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Torrent {
    #[serde(rename = "@xmlns")]
    pub xmlns: Option<String>,
    pub link: String,
    #[serde(rename = "contentLength")]
    pub content_length: Option<u64>,
    #[serde(rename = "pubDate")]
    pub pub_date: Option<String>,
}

/// 附件信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Enclosure {
    #[serde(rename = "@type")]
    pub enclosure_type: String,
    #[serde(rename = "@length")]
    pub length: u64,
    #[serde(rename = "@url")]
    pub url: String,
}

impl MikanRss {
    /// 从XML字符串解析RSS
    pub fn from_xml(xml: &str) -> Result<Self> {
        let rss: MikanRss = from_str(xml)?;
        Ok(rss)
    }
}

impl Item {
    /// 获取种子发布日期
    pub fn get_pub_date(&self) -> Option<NaiveDateTime> {
        // 尝试解析日期字符串
        self.torrent.as_ref().and_then(|torrent| {
            torrent
                .pub_date
                .as_ref()
                .and_then(|date| smart_parse_date(date).ok())
        })
    }

    /// 从种子链接中提取信息哈希
    pub fn get_info_hash(&self) -> Option<String> {
        // 从链接中提取信息哈希
        // 例如：https://mikanani.me/Home/Episode/8420088902677e8e142b35e2bc325e82c6cafb04
        self.link.as_ref().and_then(|link| {
            link.split('/').next_back().and_then(|hash| {
                // 验证 info_hash 长度，标准 SHA-1 哈希应为 40 个字符
                if hash.len() == 40 {
                    Some(hash.to_string())
                } else {
                    None
                }
            })
        })
    }

    /// 获取种子下载链接
    pub fn get_torrent_url(&self) -> Option<String> {
        self.enclosure
            .as_ref()
            .map(|enclosure| enclosure.url.clone())
    }

    /// 获取文件大小（字节）
    pub fn get_file_size(&self) -> Option<u64> {
        self.enclosure.as_ref().map(|enclosure| enclosure.length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rss() {
        let xml = r#"<rss version="2.0">
        <channel>
        <title>Mikan Project - 缘结甘神家</title>
        <link>http://mikanani.me/RSS/Bangumi?bangumiId=3422</link>
        <description>Mikan Project - 缘结甘神家</description>
        <item>
        <guid isPermaLink="false">[LoliHouse] 结缘甘神神社 / Amagami-san Chi no Enmusubi - 20 [WebRip 1080p HEVC-10bit AAC][简繁内封字幕]</guid>
        <link>https://mikanani.me/Home/Episode/8420088902677e8e142b35e2bc325e82c6cafb04</link>
        <title>[LoliHouse] 结缘甘神神社 / Amagami-san Chi no Enmusubi - 20 [WebRip 1080p HEVC-10bit AAC][简繁内封字幕]</title>
        <description>[LoliHouse] 结缘甘神神社 / Amagami-san Chi no Enmusubi - 20 [WebRip 1080p HEVC-10bit AAC][简繁内封字幕][283.06 MB]</description>
        <torrent xmlns="https://mikanani.me/0.1/">
        <link>https://mikanani.me/Home/Episode/8420088902677e8e142b35e2bc325e82c6cafb04</link>
        <contentLength>296809920</contentLength>
        <pubDate>2025-02-26T22:25:16.212</pubDate>
        </torrent>
        <enclosure type="application/x-bittorrent" length="296809920" url="https://mikanani.me/Download/20250226/8420088902677e8e142b35e2bc325e82c6cafb04.torrent"/>
        </item>
        </channel>
        </rss>"#;

        let rss = MikanRss::from_xml(xml).unwrap();
        assert_eq!(rss.version, "2.0");
        assert_eq!(rss.channel.title, "Mikan Project - 缘结甘神家");
        assert_eq!(rss.channel.items.len(), 1);

        let item = &rss.channel.items[0];
        assert_eq!(item.title, Some("[LoliHouse] 结缘甘神神社 / Amagami-san Chi no Enmusubi - 20 [WebRip 1080p HEVC-10bit AAC][简繁内封字幕]".to_string()));
        assert_eq!(item.get_file_size(), Some(296809920));
        assert_eq!(
            item.get_info_hash().unwrap(),
            "8420088902677e8e142b35e2bc325e82c6cafb04"
        );
    }
}
