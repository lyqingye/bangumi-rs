use anyhow::Result;
use model::{file_name_parse_record, subscriptions, torrents};
use parser::{Language, VideoResolution};
use std::collections::HashSet;

#[derive(Clone)]
pub struct TorrentSelector {
    // 缓存解析后的过滤器
    language_filters: HashSet<Language>,
    resolution_filters: HashSet<VideoResolution>,
    release_group_filters: HashSet<String>,
}

impl TorrentSelector {
    pub fn new(subscription: &subscriptions::Model) -> Self {
        // 解析语言过滤器字符串
        let language_filters = subscription
            .language_filter
            .as_ref()
            .map(|filter| {
                filter
                    .split(',')
                    .map(Language::from)
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default();

        // 解析分辨率过滤器
        let resolution_filters = subscription
            .resolution_filter
            .as_ref()
            .map(|filter| {
                filter
                    .split(',')
                    .map(|s| VideoResolution::from(s.trim()))
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default();

        // 解析发布组过滤器
        let release_group_filters = subscription
            .release_group_filter
            .as_ref()
            .map(|filter| {
                filter
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default();

        Self {
            language_filters,
            resolution_filters,
            release_group_filters,
        }
    }

    /// 根据订阅设置和解析结果选择最合适的种子
    pub fn select(
        &self,
        torrents: Vec<(torrents::Model, file_name_parse_record::Model)>,
    ) -> Result<Option<torrents::Model>> {
        if torrents.is_empty() {
            return Ok(None);
        }

        // 根据订阅设置过滤种子
        let mut candidates: Vec<_> = torrents
            .into_iter()
            .filter(|(_, parse_result)| {
                self.match_resolution_filter(parse_result)
                    && self.match_language_filter(parse_result)
                    && self.match_release_group_filter(parse_result)
            })
            .collect();

        if candidates.is_empty() {
            return Ok(None);
        }

        // 按照优先级排序：
        // 1. 分辨率优先级（4K > 1080p > 720p）
        // 2. 语言优先级（CHS > JPN > Unknown）
        // 3. 发布时间（新 > 旧）
        candidates.sort_by(|(a_torrent, a_parse), (b_torrent, b_parse)| {
            // 先比较分辨率
            let a_resolution = a_parse
                .video_resolution
                .as_deref()
                .map(VideoResolution::from)
                .unwrap_or(VideoResolution::Unknown);
            let b_resolution = b_parse
                .video_resolution
                .as_deref()
                .map(VideoResolution::from)
                .unwrap_or(VideoResolution::Unknown);

            let resolution_cmp = b_resolution.cmp(&a_resolution);

            if resolution_cmp != std::cmp::Ordering::Equal {
                return resolution_cmp;
            }

            // 分辨率相同时，比较语言优先级
            let a_languages: Vec<Language> = a_parse
                .language
                .as_ref()
                .map(|langs| langs.split(',').map(Language::from).collect())
                .unwrap_or_default();
            let b_languages: Vec<Language> = b_parse
                .language
                .as_ref()
                .map(|langs| langs.split(',').map(Language::from).collect())
                .unwrap_or_default();

            // 获取每个种子的最高语言优先级
            let a_max_priority = a_languages
                .iter()
                .max()
                .copied()
                .unwrap_or(Language::Unknown);
            let b_max_priority = b_languages
                .iter()
                .max()
                .copied()
                .unwrap_or(Language::Unknown);

            let lang_cmp = b_max_priority.cmp(&a_max_priority);

            if lang_cmp != std::cmp::Ordering::Equal {
                return lang_cmp;
            }

            // 语言优先级相同时，按发布时间排序（新的优先）
            b_torrent.pub_date.cmp(&a_torrent.pub_date)
        });

        // 返回排序后的第一个种子
        Ok(Some(candidates.into_iter().next().unwrap().0))
    }

    /// 检查是否匹配分辨率过滤器
    fn match_resolution_filter(&self, parse_result: &file_name_parse_record::Model) -> bool {
        // 如果没有设置过滤器，接受所有分辨率
        if self.resolution_filters.is_empty() {
            return true;
        }

        let torrent_resolution = parse_result
            .video_resolution
            .as_deref()
            .map(VideoResolution::from)
            .unwrap_or(VideoResolution::Unknown);

        // 检查是否匹配任一分辨率
        self.resolution_filters.contains(&torrent_resolution)
    }

    /// 检查是否匹配语言过滤器
    fn match_language_filter(&self, parse_result: &file_name_parse_record::Model) -> bool {
        // 如果没有设置过滤器，接受所有语言
        if self.language_filters.is_empty() {
            return true;
        }

        // 解析种子的语言列表
        let torrent_languages: HashSet<Language> = parse_result
            .language
            .as_ref()
            .map(|langs| langs.split(',').map(Language::from).collect())
            .unwrap_or_default();

        // 检查是否至少匹配一个所需的语言
        !self.language_filters.is_disjoint(&torrent_languages)
    }

    /// 检查是否匹配发布组过滤器
    fn match_release_group_filter(&self, parse_result: &file_name_parse_record::Model) -> bool {
        // 如果没有设置过滤器，接受所有发布组
        if self.release_group_filters.is_empty() {
            return true;
        }

        // 检查发布组是否匹配任一过滤条件
        parse_result
            .release_group
            .as_ref()
            .map(|group| self.release_group_filters.contains(group))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use model::sea_orm_active_enums::SubscribeStatus;

    fn create_test_data(
        json_str: &str,
    ) -> (
        subscriptions::Model,
        Vec<(torrents::Model, file_name_parse_record::Model)>,
    ) {
        let data: serde_json::Value = serde_json::from_str(json_str).unwrap();

        // 创建订阅
        let subscription = subscriptions::Model {
            bangumi_id: 1,
            subscribe_status: SubscribeStatus::Subscribed,
            start_episode_number: None,
            collector_interval: None,
            metadata_interval: None,
            task_processor_interval: None,
            resolution_filter: data["subscription"]["resolution_filter"]
                .as_str()
                .map(String::from),
            language_filter: data["subscription"]["language_filter"]
                .as_str()
                .map(String::from),
            release_group_filter: data["subscription"]["release_group_filter"]
                .as_str()
                .map(String::from),
            created_at: NaiveDateTime::default(),
            updated_at: NaiveDateTime::default(),
        };

        // 创建种子列表
        let now = NaiveDateTime::default();
        let torrents: Vec<(torrents::Model, file_name_parse_record::Model)> = data["torrents"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, t)| {
                (
                    torrents::Model {
                        bangumi_id: 1,
                        title: format!("Test{}", i),
                        size: 0,
                        info_hash: t["info_hash"].as_str().unwrap().to_string(),
                        magnet: format!("magnet:{}", i),
                        data: None,
                        created_at: now
                            + chrono::Duration::seconds(
                                t["created_at_offset"].as_i64().unwrap_or(0),
                            ),
                        updated_at: now,
                        download_url: None,
                    },
                    file_name_parse_record::Model {
                        file_name: format!("Test{}", i),
                        release_group: t["release_group"].as_str().map(String::from),
                        bangumi_name: None,
                        season_number: None,
                        episode_number: Some(1),
                        language: t["language"].as_str().map(String::from),
                        video_resolution: t["resolution"].as_str().map(String::from),
                        year: None,
                        parser_name: "test".to_string(),
                        parser_status: model::sea_orm_active_enums::ParserStatus::Completed,
                        err_msg: None,
                        created_at: now,
                        updated_at: now,
                    },
                )
            })
            .collect();

        (subscription, torrents)
    }

    #[test]
    fn test_basic_filters() -> Result<()> {
        let test_data = r#"{
            "subscription": {
                "resolution_filter": "1080p",
                "language_filter": "CHS",
                "release_group_filter": "LoliHouse"
            },
            "torrents": [
                {
                    "info_hash": "1",
                    "resolution": "1080p",
                    "language": "CHS",
                    "release_group": "LoliHouse",
                    "created_at_offset": 0
                },
                {
                    "info_hash": "2",
                    "resolution": "720p",
                    "language": "CHS",
                    "release_group": "LoliHouse",
                    "created_at_offset": 0
                }
            ]
        }"#;

        let (subscription, torrents) = create_test_data(test_data);
        let selector = TorrentSelector::new(&subscription, 1);
        let selected = selector.select(torrents)?;

        assert!(selected.is_some());
        assert_eq!(selected.unwrap().info_hash, "1"); // 应该选择1080p的种子
        Ok(())
    }

    #[test]
    fn test_multiple_filters() -> Result<()> {
        let test_data = r#"{
            "subscription": {
                "resolution_filter": "1080p,720p",
                "language_filter": "CHS,JPN",
                "release_group_filter": "LoliHouse,Kamigami"
            },
            "torrents": [
                {
                    "info_hash": "1",
                    "resolution": "720p",
                    "language": "JPN",
                    "release_group": "Kamigami",
                    "created_at_offset": 0
                },
                {
                    "info_hash": "2",
                    "resolution": "1080p",
                    "language": "CHS",
                    "release_group": "LoliHouse",
                    "created_at_offset": 0
                }
            ]
        }"#;

        let (subscription, torrents) = create_test_data(test_data);
        let selector = TorrentSelector::new(&subscription);
        let selected = selector.select(torrents)?;

        assert!(selected.is_some());
        assert_eq!(selected.unwrap().info_hash, "2"); // 应该选择1080p的种子，因为分辨率优先级更高
        Ok(())
    }

    #[test]
    fn test_priority_ordering() -> Result<()> {
        let test_data = r#"{
            "subscription": {
                "resolution_filter": "1080p",
                "language_filter": "CHS,JPN",
                "release_group_filter": "LoliHouse"
            },
            "torrents": [
                {
                    "info_hash": "1",
                    "resolution": "1080p",
                    "language": "JPN",
                    "release_group": "LoliHouse",
                    "created_at_offset": 0
                },
                {
                    "info_hash": "2",
                    "resolution": "1080p",
                    "language": "CHS",
                    "release_group": "LoliHouse",
                    "created_at_offset": 0
                },
                {
                    "info_hash": "3",
                    "resolution": "1080p",
                    "language": "CHS",
                    "release_group": "LoliHouse",
                    "created_at_offset": 60
                }
            ]
        }"#;

        let (subscription, torrents) = create_test_data(test_data);
        let selector = TorrentSelector::new(&subscription);
        let selected = selector.select(torrents)?;

        assert!(selected.is_some());
        assert_eq!(selected.unwrap().info_hash, "3"); // 应该选择最新的CHS种子
        Ok(())
    }

    #[test]
    fn test_empty_filters() -> Result<()> {
        let test_data = r#"{
            "subscription": {
                "resolution_filter": null,
                "language_filter": null,
                "release_group_filter": null
            },
            "torrents": [
                {
                    "info_hash": "1",
                    "resolution": "720p",
                    "language": "ENG",
                    "release_group": "HorribleSubs",
                    "created_at_offset": 0
                },
                {
                    "info_hash": "2",
                    "resolution": "1080p",
                    "language": "CHS",
                    "release_group": "LoliHouse",
                    "created_at_offset": 0
                }
            ]
        }"#;

        let (subscription, torrents) = create_test_data(test_data);
        let selector = TorrentSelector::new(&subscription);
        let selected = selector.select(torrents)?;

        assert!(selected.is_some());
        assert_eq!(selected.unwrap().info_hash, "2"); // 应该选择1080p的种子，因为分辨率优先级更高
        Ok(())
    }

    #[test]
    fn test_no_matching_torrents() -> Result<()> {
        let test_data = r#"{
            "subscription": {
                "resolution_filter": "2160p",
                "language_filter": "CHT",
                "release_group_filter": "NonExistent"
            },
            "torrents": [
                {
                    "info_hash": "1",
                    "resolution": "1080p",
                    "language": "CHS",
                    "release_group": "LoliHouse",
                    "created_at_offset": 0
                }
            ]
        }"#;

        let (subscription, torrents) = create_test_data(test_data);
        let selector = TorrentSelector::new(&subscription);
        let selected = selector.select(torrents)?;

        assert!(selected.is_none()); // 应该没有匹配的种子
        Ok(())
    }
}
