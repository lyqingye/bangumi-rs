use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RssChannel {
    pub title: String,
    pub description: String,
    pub link: String,
    pub ttl: i32,
    #[serde(default)]
    pub item: Vec<RssItem>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RssItem {
    pub title: String,
    pub description: String,
    #[serde(rename = "pubDate")]
    pub pub_date: String,
    pub link: String,
    pub guid: String,
    pub enclosure: RssEnclosure,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RssEnclosure {
    #[serde(rename = "@url")]
    pub url: String,
    #[serde(rename = "@type")]
    pub enclosure_type: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RssResponse {
    #[serde(rename = "@version")]
    pub version: String,
    pub channel: RssChannel,
}

impl RssItem {
    /// 获取发布日期
    pub fn get_pub_date(&self) -> Option<DateTime<FixedOffset>> {
        DateTime::parse_from_rfc2822(&self.pub_date).ok()
    }

    /// 获取种子链接
    pub fn get_torrent_url(&self) -> &str {
        &self.enclosure.url
    }

    /// 获取详情页链接
    pub fn get_detail_url(&self) -> &str {
        &self.link
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::de::from_str;

    #[test]
    fn test_rss_deserialize() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0">
        <channel>
        <title>ACG.RIP</title>
        <description>ACG.RIP has super cow power</description>
        <link>https://acgrip.art/page/2.xml?term=%E6%88%91%E7%8B%AC%E8%87%AA%E5%8D%87%E7%BA%A7</link>
        <ttl>1800</ttl>
        <item>
        <title>[猎户压制部] 我独自升级 第二季 -起于暗影- / Ore dake Level Up na Ken S2 [14] [1080p] [繁日内嵌] [KoVer] [2025年1月番]</title>
        <description>简介：自世界各地出现连接异次元与现实世界的通路「传送门」，已过了十多年。觉醒了特殊能力，被称为「猎人」的人们，与存在于传送门里地下城内的魔兽不断厮杀。猎人的能力在觉醒后就不再有成长空间，其等级也不会再有变化。然而，被称作是人类最弱兵器的成振宇，在一次双重地下城的突击任务中得到了只有自己能够「升级」的能力，得以在战斗中不断变强。</description>
        <pubDate>Thu, 16 Jan 2025 23:46:22 -0800</pubDate>
        <link>https://acgrip.art/t/321591</link>
        <guid>https://acgrip.art/t/321591</guid>
        <enclosure url="https://acgrip.art/t/321591.torrent" type="application/x-bittorrent"/>
        </item>
        </channel>
        </rss>"#;

        let rss: RssResponse = from_str(xml).unwrap();
        assert_eq!(rss.version, "2.0");
        assert_eq!(rss.channel.title, "ACG.RIP");
        assert_eq!(rss.channel.item.len(), 1);
        assert_eq!(
            rss.channel.item[0].title,
            "[猎户压制部] 我独自升级 第二季 -起于暗影- / Ore dake Level Up na Ken S2 [14] [1080p] [繁日内嵌] [KoVer] [2025年1月番]"
        );
        assert_eq!(
            rss.channel.item[0].enclosure.url,
            "https://acgrip.art/t/321591.torrent"
        );
    }
}
