#![allow(dead_code)]
use std::fmt;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod db;
pub mod impls;
pub mod worker;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParseResult {
    #[serde(skip)]
    pub file_name: String,
    #[serde(default)]
    pub release_group: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub year: Option<i32>,
    #[serde(default)]
    pub season: Option<i32>,
    #[serde(default)]
    pub episode: Option<i32>,
    #[serde(default)]
    pub video_resolution: Option<VideoResolution>,
    #[serde(default)]
    pub languages: Vec<Language>,
}

pub const PROMPT_TEMPLATE: &str = r#"
### Task Description
Parse anime video filenames and extract structured information. Each filename may contain various components separated by delimiters like [ ], _, or -.

### Input Format
A list of filenames containing any of these components:
- Release Group: The group that released the video
- Episode: Episode number (may be prefixed with E, EP, or similar)
- Season: Season number (may be prefixed with S)
- Video Resolution: Specific video resolution information (e.g., 1080p, 720p, 1024x720, or dimensions). **Normalize this field by mapping any detected resolution to one of the following categories: 2160P, 1080P, 1440P, or 720P. If no resolution keyword is identified, set the video_resolution field to an empty array.**
- Languages: Subtitle/audio languages (e.g., Jpn, Eng, Chs, 简体, 繁体). **Normalize this field by converting each language into one of the following: JPN, ENG, CHS, or CHT. If no language keyword is identified, set the languages field to an empty array. Note: ignore case.**

### Output Requirements
1. Return a JSON array containing one object per filename.
2. Each object must only include fields that are present in the filename.
3. Use consistent field names:
   - release_group string
   - episode (as number)
   - season (as number)
   - video_resolution string
   - languages (as array)
4. Ensure all values are properly typed (numbers as integers, arrays as arrays).
5. IMPORTANT: Output ONLY the raw JSON array (plain text), no markdown formatting, no code blocks, no additional text.

### EXAMPLE JSON INPUT:
[
"[TaigaSubs]Toradora!(2008)_-01v2-Tiger_and_Dragon[1280x720_H.264_FLAC][Jpn_Chs_Cht][1234ABCD].mkv",
"【幻樱字幕组】【1月新番】【魔法制造者 ~异世界魔法的制作方法~ Magic Maker ~Isekai Mahou no Tsukurikata~】【04】【BIG5_GB】【1920X1080】"
]

### EXAMPLE JSON OUTPUT:
[
{ "release_group": "TaigaSubs", "episode": 1, "season": 0, "video_resolution": "720P", "languages": ["JPN", "CHS", "CHT"] },
{ "release_group": "幻樱字幕组", "episode": 4, "season": 0, "video_resolution": "1080P", "languages": ["CHS","CHT] },
]
"#;

#[async_trait]
pub trait Parser: Send + Sync {
    async fn parse_file_names(&self, file_names: Vec<String>) -> Result<Vec<ParseResult>>;
    fn max_file_name_length(&self) -> usize;
    fn name(&self) -> String;
}

fn remove_code_block(text: &str) -> String {
    text.replace("```json", "").replace("```", "").to_owned()
}

fn parse_msg(msg: &str) -> Result<Vec<ParseResult>> {
    let content = remove_code_block(msg.trim());

    // 处理空字符串情况
    if content.is_empty() {
        return Err(anyhow::anyhow!("empty message"));
    }

    // 尝试解析 JSON
    let json_value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("failed to parse JSON: {}", e))?;

    match json_value {
        serde_json::Value::Array(arr) => serde_json::from_value(arr.into())
            .map_err(|e| anyhow::anyhow!("failed to parse array of ParseResult: {}", e)),
        serde_json::Value::Object(_) => {
            let parse_result: ParseResult = serde_json::from_value(json_value)
                .map_err(|e| anyhow::anyhow!("failed to parse single ParseResult: {}", e))?;
            Ok(vec![parse_result])
        }
        _ => Err(anyhow::anyhow!(
            "invalid JSON format, expected array or object"
        )),
    }
}

fn fill_file_names(file_names: Vec<String>, parse_results: &mut [ParseResult]) -> Result<()> {
    if file_names.len() != parse_results.len() {
        return Err(anyhow::anyhow!(
            "file_names and parse_results length mismatch"
        ));
    }

    for (i, file_name) in file_names.iter().enumerate() {
        parse_results[i].file_name = file_name.clone();
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy, Serialize, Deserialize)]
pub enum Language {
    #[serde(rename = "CHT")]
    CHT = 4,
    #[serde(rename = "CHS")]
    CHS = 3,
    #[serde(rename = "JPN")]
    JPN = 2,
    #[serde(rename = "ENG")]
    ENG = 1,
    #[serde(other)]
    Unknown = 0,
}

impl Language {
    // 获取优先级值的辅助方法
    pub fn priority(&self) -> i32 {
        match self {
            Language::CHS => 4,
            Language::CHT => 3,
            Language::JPN => 2,
            Language::ENG => 1,
            Language::Unknown => 0,
        }
    }
}

impl PartialOrd for Language {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.priority().cmp(&other.priority()))
    }
}

impl Ord for Language {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority().cmp(&other.priority())
    }
}

impl From<&str> for Language {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "CHT" => Language::CHT,
            "CHS" => Language::CHS,
            "JPN" => Language::JPN,
            "ENG" => Language::ENG,
            _ => Language::Unknown,
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::CHT => write!(f, "CHT"),
            Language::CHS => write!(f, "CHS"),
            Language::JPN => write!(f, "JPN"),
            Language::ENG => write!(f, "ENG"),
            Language::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VideoResolution {
    #[serde(rename = "2160P")]
    P2160 = 4,
    #[serde(rename = "1440P")]
    P1440 = 3,
    #[serde(rename = "1080P")]
    P1080 = 2,
    #[serde(rename = "720P")]
    P720 = 1,
    #[serde(other)]
    Unknown = 0,
}

impl VideoResolution {
    // 获取优先级值的辅助方法
    fn priority(&self) -> i32 {
        match self {
            VideoResolution::P2160 => 4,
            VideoResolution::P1440 => 3,
            VideoResolution::P1080 => 2,
            VideoResolution::P720 => 1,
            VideoResolution::Unknown => 0,
        }
    }
}

impl PartialOrd for VideoResolution {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.priority().cmp(&other.priority()))
    }
}

impl Ord for VideoResolution {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority().cmp(&other.priority())
    }
}

impl From<&str> for VideoResolution {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "2160P" => VideoResolution::P2160,
            "1440P" => VideoResolution::P1440,
            "1080P" => VideoResolution::P1080,
            "720P" => VideoResolution::P720,
            _ => VideoResolution::Unknown,
        }
    }
}

impl fmt::Display for VideoResolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VideoResolution::P2160 => write!(f, "2160P"),
            VideoResolution::P1440 => write!(f, "1440P"),
            VideoResolution::P1080 => write!(f, "1080P"),
            VideoResolution::P720 => write!(f, "720P"),
            VideoResolution::Unknown => write!(f, "Unknown"),
        }
    }
}
