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
        torrents: &[(torrents::Model, file_name_parse_record::Model)],
    ) -> Option<torrents::Model> {
        if torrents.is_empty() {
            return None;
        }

        // 根据订阅设置过滤种子
        let mut candidates: Vec<_> = torrents
            .iter()
            .filter(|(_, parse_result)| {
                self.match_resolution_filter(parse_result)
                    && self.match_language_filter(parse_result)
                    && self.match_release_group_filter(parse_result)
            })
            .filter(|(torrent, _)| {
                // 过滤掉小于100MB的种子
                torrent.size >= 100 * 1024 * 1024
            })
            .collect();

        if candidates.is_empty() {
            return None;
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
        Some(candidates[0].0.clone())
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
