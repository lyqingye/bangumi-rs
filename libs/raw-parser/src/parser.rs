use crate::config::Config;
use crate::models::{ParseResult, CHINESE_NUMBER_MAP};
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;

// 使用 lazy_static 定义全局正则表达式，避免重复编译
lazy_static! {
    // 匹配集数的正则表达式
    static ref EPISODE_PATTERN: Regex = Regex::new(r"\d+").unwrap();

    // 匹配标题格式的正则表达式，包含标题、集数和其他信息
    static ref TITLE_PATTERN: Regex = Regex::new(
        r"(.*|\[.*])( -? \d+|\[\d+]|\[\d+.?[vV]\d]|第\d+[话話集]|\[第?\d+[话話集]]|\[\d+.?END]|[Ee][Pp]?\d+)(.*)"
    ).unwrap();

    // 匹配视频分辨率的正则表达式
    static ref RESOLUTION_PATTERN: Regex = Regex::new(r"1080|720|2160|4K").unwrap();

    // 匹配简体中文字幕的正则表达式
    static ref SUB_CHS_PATTERN: Regex = Regex::new(r"[简中]|CHS|SC").unwrap();

    // 匹配繁体中文字幕的正则表达式
    static ref SUB_CHT_PATTERN: Regex = Regex::new(r"繁|CHT|BIG5|GB").unwrap();

    // 匹配日语字幕的正则表达式
    static ref SUB_JPN_PATTERN: Regex = Regex::new(r"[日]|JP|JPSC").unwrap();

    // 匹配英语字幕的正则表达式
    static ref SUB_ENG_PATTERN: Regex = Regex::new(r"ENG|英语|英文").unwrap();

    // 匹配非法前缀字符的正则表达式
    static ref PREFIX_PATTERN: Regex =
        Regex::new(r"[^\w\s\u4e00-\u9fff\u3040-\u309f\u30a0-\u30ff-]").unwrap();

    // 匹配季度信息的正则表达式
    static ref SEASON_PATTERN: Regex =
        Regex::new(r"S\d{1,2}|Season \d{1,2}|[第].[季期]").unwrap();

    // 匹配方括号的正则表达式
    static ref BRACKET_PATTERN: Regex = Regex::new(r"[\[\]]").unwrap();

    // 匹配中文字符的正则表达式
    static ref CHS_PATTERN: Regex = Regex::new(r"^[\u4e00-\u9fa5]{2,}").unwrap();

    // 匹配日文字符的正则表达式
    static ref JP_PATTERN: Regex = Regex::new(r"[\u0800-\u4e00]{2,}").unwrap();

    // 匹配英文字符的正则表达式
    static ref EN_PATTERN: Regex = Regex::new(r"[a-zA-Z]{3,}").unwrap();
}

/// 动画文件名解析器
#[derive(Debug, Clone)]
pub struct Parser {
    config: Config,
}

impl Parser {
    /// 创建新的解析器实例
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// 从方括号中提取字幕组信息
    fn get_group(name: &str) -> Option<String> {
        BRACKET_PATTERN.split(name).nth(1).map(|s| s.to_string())
    }

    /// 预处理文件名，将中文方括号转换为英文方括号
    fn pre_process(raw_name: &str) -> String {
        raw_name.replace('【', "[").replace('】', "]")
    }

    /// 处理文件名前缀，移除字幕组信息和无关标记
    fn prefix_process(raw: &str, group: &Option<String>) -> String {
        let mut result = raw.to_string();
        // 移除字幕组信息
        if let Some(group) = group {
            result = result.replace(&format!("[{}]", group), "");
        }

        // 处理前缀
        let raw_process = PREFIX_PATTERN.replace_all(&result, "/").into_owned();
        let arg_group: Vec<&str> = raw_process.split('/').filter(|s| !s.is_empty()).collect();

        let arg_group = if arg_group.len() == 1 {
            arg_group[0].split_whitespace().collect::<Vec<_>>()
        } else {
            arg_group
        };

        // 移除特定标记
        for &arg in &arg_group {
            if (arg.contains("新番") || arg.contains("月番")) && arg.len() <= 5
                || arg.contains("港澳台地区")
            {
                result = result.replace(arg, "");
            }
        }

        result
    }

    /// 处理季度信息，返回处理后的名称、原始季度文本和季度数字
    fn season_process(season_info: &str) -> (String, Option<i32>) {
        let name_season = BRACKET_PATTERN.replace_all(season_info, " ").into_owned();
        let seasons: Vec<_> = SEASON_PATTERN
            .find_iter(&name_season)
            .map(|m| m.as_str())
            .collect();

        if seasons.is_empty() {
            return (name_season, None);
        }

        let name = SEASON_PATTERN.replace_all(&name_season, "").into_owned();
        let season_raw = seasons[0].to_string();

        // 解析季度数字
        let season = if season_raw.contains("Season") || season_raw.contains('S') {
            season_raw
                .replace("Season", "")
                .replace('S', "")
                .trim()
                .parse()
                .unwrap_or(1)
        } else {
            let season_text = season_raw
                .replace(['第', '季', '期'], "")
                .trim()
                .to_string();

            match season_text.parse::<i32>() {
                Ok(num) => num,
                Err(_) => *CHINESE_NUMBER_MAP.get(season_text.as_str()).unwrap_or(&1),
            }
        };

        (name, Some(season))
    }

    /// 处理动画名称，分离出英文、中文和日文标题
    fn name_process(name: &str) -> (Option<String>, Option<String>, Option<String>) {
        let name = name.trim();
        let name = name
            .replace("(仅限港澳台地区)", "")
            .replace("（仅限港澳台地区）", "");

        // 分割标题
        let mut splits: Vec<String> = name
            .split(['/', '-', '_'])
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // 处理单个标题的情况
        if splits.len() == 1 {
            let split_space: Vec<String> = splits[0]
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();

            for &idx in &[0, split_space.len() - 1] {
                if let Some(word) = split_space.get(idx) {
                    if CHS_PATTERN.is_match(word) {
                        let joined_space = split_space.join(" ");
                        splits = vec![word.clone(), joined_space];
                        break;
                    }
                }
            }
        }

        let mut name_en = None;
        let mut name_zh = None;
        let mut name_jp = None;

        // 识别不同语言的标题
        for item in &splits {
            if JP_PATTERN.is_match(item) && name_jp.is_none() {
                name_jp = Some(item.clone());
            } else if CHS_PATTERN.is_match(item) && name_zh.is_none() {
                name_zh = Some(item.clone());
            } else if EN_PATTERN.is_match(item) && name_en.is_none() {
                name_en = Some(item.clone());
            }
        }

        (name_en, name_zh, name_jp)
    }

    /// 从其他信息中提取字幕类型和分辨率
    fn find_tags(other: &str) -> (Vec<String>, Option<String>) {
        let replaced = BRACKET_PATTERN.replace_all(other, " ").into_owned();
        let elements: Vec<&str> = replaced.split_whitespace().collect();

        let mut subs = Vec::new();
        let mut resolution = None;

        for &element in &elements {
            if SUB_CHS_PATTERN.is_match(element) {
                subs.push("CHS".to_string());
            }
            if SUB_CHT_PATTERN.is_match(element) {
                subs.push("CHT".to_string());
            }
            if SUB_JPN_PATTERN.is_match(element) {
                subs.push("JPN".to_string());
            }
            if SUB_ENG_PATTERN.is_match(element) {
                subs.push("ENG".to_string());
            }
            if RESOLUTION_PATTERN.is_match(element) {
                resolution = Some(element.to_string());
            }
        }

        // 去重
        subs.sort();
        subs.dedup();

        (subs, resolution)
    }

    /// 清理字幕类型信息，移除无关后缀
    fn clean_sub(subs: Vec<String>) -> Vec<String> {
        subs.into_iter()
            .map(|s| s.replace("_MP4", "").replace("_MKV", ""))
            .collect()
    }

    /// 解析动画文件名，提取所有相关信息
    pub fn parse(&self, file_name: &str) -> Result<ParseResult> {
        let raw_title = file_name.trim().replace('\n', " ");
        let content_title = Self::pre_process(&raw_title);

        // 获取字幕组信息
        let group = Self::get_group(&content_title);

        // 在严格模式下检查字幕组信息
        if self.config.strict_mode && group.is_none() {
            return Err(anyhow!("无法解析字幕组信息"));
        }

        // 解析标题格式
        let captures = TITLE_PATTERN
            .captures(&content_title)
            .ok_or_else(|| anyhow!("无法解析标题格式"))?;

        let season_info = captures.get(1).map(|m| m.as_str().trim()).unwrap_or("");

        let episode_info = captures.get(2).map(|m| m.as_str().trim()).unwrap_or("");

        let other = captures.get(3).map(|m| m.as_str().trim()).unwrap_or("");

        // 处理前缀
        let process_raw = Self::prefix_process(season_info, &group);

        // 处理季度信息
        let (raw_name, season) = Self::season_process(&process_raw);

        // 处理名称
        let (name_en, name_zh, name_jp) = Self::name_process(&raw_name);

        // 处理集数
        let episode = EPISODE_PATTERN
            .find(episode_info)
            .and_then(|m| m.as_str().parse().ok());

        // 在严格模式下检查集数信息
        if self.config.strict_mode && episode.is_none() {
            return Err(anyhow!("无法解析集数信息"));
        }

        // 处理其他标签
        let (sub, resolution) = Self::find_tags(other);
        let sub = Self::clean_sub(sub);

        // 返回解析结果
        Ok(ParseResult {
            name_en,
            name_zh,
            name_jp,
            episode,
            season,
            subtitle_group: group,
            resolution,
            sub_type: sub,
        })
    }
}
