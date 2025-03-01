use anyhow::Result;
use chrono::Duration as ChronoDuration;
use humantime_serde;
use serde::{Deserialize, Serialize};
use std::time::Duration as StdDuration;
const DEFAULT_LOG_LEVEL: &str = "info";
const DEFAULT_ASSETS_PATH: &str = "./assets";
const DEFAULT_LISTEN_ADDR: &str = "127.0.0.1:3000";
const DEFAULT_DATABASE_URL: &str = "mysql://root:123456@192.168.1.10:3306/bangumi";
const DEFAULT_MIKAN_ENDPOINT: &str = "https://mikanani.me";
const DEFAULT_BANGUMI_TV_ENDPOINT: &str = "https://api.bgm.tv";
const DEFAULT_TMDB_BASE_URL: &str = "https://api.themoviedb.org/3";
const DEFAULT_TMDB_IMAGE_BASE_URL: &str = "https://image.tmdb.org/t/p/original/";
const DEFAULT_TMDB_LANGUAGE: &str = "zh-CN";

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Config {
    pub log: LogConfig,
    pub server: ServerConfig,
    pub mikan: MikanConfig,
    pub bangumi_tv: BangumiTvConfig,
    pub tmdb: TMDBConfig,
    pub parser: ParserConfig,
    pub pan115: Pan115Config,
    pub notify: NotifyConfig,
    pub proxy: ProxyConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub http: String,
    pub https: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NotifyConfig {
    pub telegram: TelegramConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TelegramConfig {
    pub enabled: bool,
    pub token: String,
    pub chat_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pan115Config {
    pub cookies: String,
    pub download_dir: String,
    pub max_requests_per_second: u32,
    pub max_retry_count: i32,
    #[serde(
        serialize_with = "serialize_chrono_duration",
        deserialize_with = "deserialize_chrono_duration"
    )]
    pub offline_download_timeout: ChronoDuration,
}

// 将 chrono::Duration 转换为 std::time::Duration 进行序列化
fn serialize_chrono_duration<S>(duration: &ChronoDuration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    // 将 chrono::Duration 转换为秒数
    let seconds = duration.num_seconds();
    let nanos = (duration.num_milliseconds() % 1000) as u32 * 1_000_000;

    // 创建 std::time::Duration
    let std_duration = StdDuration::new(seconds as u64, nanos);

    // 使用 humantime_serde 序列化
    humantime_serde::serialize(&std_duration, serializer)
}

// 从 std::time::Duration 反序列化为 chrono::Duration
fn deserialize_chrono_duration<'de, D>(deserializer: D) -> Result<ChronoDuration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // 使用 humantime_serde 反序列化为 std::time::Duration
    let std_duration: StdDuration = humantime_serde::deserialize(deserializer)?;

    // 转换为 chrono::Duration
    let chrono_duration = ChronoDuration::seconds(std_duration.as_secs() as i64)
        + ChronoDuration::nanoseconds(std_duration.subsec_nanos() as i64);

    Ok(chrono_duration)
}

impl Default for Pan115Config {
    fn default() -> Self {
        Self {
            cookies: String::new(),
            download_dir: String::new(),
            max_requests_per_second: 1,
            max_retry_count: 10,
            offline_download_timeout: ChronoDuration::minutes(10),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ParserConfig {
    pub siliconflow: SiliconflowConfig,
    pub deepseek: DeepseekConfig,
    pub deepbricks: DeepbricksConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SiliconflowConfig {
    pub enabled: bool,
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DeepseekConfig {
    pub enabled: bool,
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DeepbricksConfig {
    pub enabled: bool,
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LogConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ServerConfig {
    #[serde(default = "default_assets_path")]
    pub assets_path: String,
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,
    #[serde(default = "default_database_url")]
    pub database_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MikanConfig {
    #[serde(default = "default_mikan_endpoint")]
    pub endpoint: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TMDBConfig {
    #[serde(default = "default_tmdb_api_key")]
    pub api_key: String,
    #[serde(default = "default_tmdb_base_url")]
    pub base_url: String,
    #[serde(default = "default_tmdb_image_base_url")]
    pub image_base_url: String,
    #[serde(default = "default_tmdb_language")]
    pub language: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BangumiTvConfig {
    #[serde(default = "default_bangumi_tv_endpoint")]
    pub endpoint: String,
    pub image_base_url: String,
}

fn default_log_level() -> String {
    DEFAULT_LOG_LEVEL.to_owned()
}
fn default_assets_path() -> String {
    DEFAULT_ASSETS_PATH.to_owned()
}
fn default_listen_addr() -> String {
    DEFAULT_LISTEN_ADDR.to_owned()
}
fn default_database_url() -> String {
    DEFAULT_DATABASE_URL.to_owned()
}
fn default_mikan_endpoint() -> String {
    DEFAULT_MIKAN_ENDPOINT.to_owned()
}
fn default_bangumi_tv_endpoint() -> String {
    DEFAULT_BANGUMI_TV_ENDPOINT.to_owned()
}
fn default_tmdb_api_key() -> String {
    "".to_owned()
}
fn default_tmdb_base_url() -> String {
    DEFAULT_TMDB_BASE_URL.to_owned()
}
fn default_tmdb_image_base_url() -> String {
    DEFAULT_TMDB_IMAGE_BASE_URL.to_owned()
}
fn default_tmdb_language() -> String {
    DEFAULT_TMDB_LANGUAGE.to_owned()
}

pub trait Writer: Send + Sync {
    fn write(&self, config: &Config) -> Result<()>;
}

pub struct NopWriter;

impl Writer for NopWriter {
    fn write(&self, _config: &Config) -> Result<()> {
        Ok(())
    }
}
