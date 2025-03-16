use anyhow::Result;
use chrono::Duration as ChronoDuration;
use humantime_serde;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::{os::unix::net::SocketAddr, path::Path, time::Duration as StdDuration};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Config {
    pub log: LogConfig,
    pub server: ServerConfig,
    pub mikan: MikanConfig,
    pub bangumi_tv: BangumiTvConfig,
    pub tmdb: TMDBConfig,
    pub parser: ParserConfig,
    pub downloader: DownloaderConfig,
    pub notify: NotifyConfig,
    pub proxy: ProxyConfig,
    pub sentry: SentryConfig,
}
impl Config {
    pub fn validate(&self) -> Result<()> {
        self.log.validate()?;
        self.server.validate()?;
        self.mikan.validate()?;
        self.bangumi_tv.validate()?;
        self.tmdb.validate()?;
        self.parser.validate()?;
        self.downloader.validate()?;
        self.notify.validate()?;
        self.proxy.validate()?;
        self.sentry.validate()?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub http: String,
    pub https: String,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            http: "http://127.0.0.1:7890".to_owned(),
            https: "http://127.0.0.1:7890".to_owned(),
        }
    }
}

impl ProxyConfig {
    fn validate(&self) -> Result<()> {
        validate_url(&self.http, "proxy.http")?;
        validate_url(&self.https, "proxy.https")?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct NotifyConfig {
    pub telegram: TelegramConfig,
}

impl NotifyConfig {
    fn validate(&self) -> Result<()> {
        self.telegram.validate()?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct TelegramConfig {
    pub enabled: bool,
    pub token: String,
    pub chat_id: String,
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            token: "".to_owned(),
            chat_id: "".to_owned(),
        }
    }
}

impl TelegramConfig {
    fn validate(&self) -> Result<()> {
        if self.enabled {
            validate_not_empty(&self.token, "notify.telegram.token")?;
            validate_not_empty(&self.chat_id, "notify.telegram.chat_id")?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct DownloaderConfig {
    pub pan115: Pan115Config,
    pub qbittorrent: QbittorrentConfig,
    pub max_retry_count: i32,
    #[serde(
        serialize_with = "serialize_chrono_duration",
        deserialize_with = "deserialize_chrono_duration"
    )]
    pub retry_min_interval: ChronoDuration,
    #[serde(
        serialize_with = "serialize_chrono_duration",
        deserialize_with = "deserialize_chrono_duration"
    )]
    pub retry_max_interval: ChronoDuration,
    #[serde(
        serialize_with = "serialize_chrono_duration",
        deserialize_with = "deserialize_chrono_duration"
    )]
    pub download_timeout: ChronoDuration,
}

impl Default for DownloaderConfig {
    fn default() -> Self {
        Self {
            pan115: Pan115Config::default(),
            qbittorrent: QbittorrentConfig::default(),
            max_retry_count: 10,
            retry_min_interval: ChronoDuration::seconds(30),
            retry_max_interval: ChronoDuration::hours(1),
            download_timeout: ChronoDuration::minutes(30),
        }
    }
}

impl DownloaderConfig {
    fn validate(&self) -> Result<()> {
        validate_not_empty(&self.pan115.cookies, "downloader.pan115.cookies")?;
        self.pan115.validate()?;
        self.qbittorrent.validate()?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Pan115Config {
    pub enabled: bool,
    pub cookies: String,
    pub max_requests_per_second: u32,
    pub download_dir: String,
    pub delete_task_on_completion: bool,
}

impl Default for Pan115Config {
    fn default() -> Self {
        Self {
            enabled: false,
            cookies: "".to_owned(),
            max_requests_per_second: 1,
            download_dir: "/".to_owned(),
            delete_task_on_completion: true,
        }
    }
}

impl Pan115Config {
    fn validate(&self) -> Result<()> {
        if self.enabled {
            validate_not_empty(&self.cookies, "downloader.pan115.cookies")?;
            validate_abs_path_format(&self.download_dir, "downloader.pan115.download_dir")?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct QbittorrentConfig {
    pub enabled: bool,
    pub url: String,
    pub username: String,
    pub password: String,
    pub download_dir: String,
    pub delete_task_on_completion: bool,
}

impl Default for QbittorrentConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: "http://127.0.0.1:8080".to_owned(),
            username: "admin".to_owned(),
            password: "adminadmin".to_owned(),
            download_dir: "/downloads".to_owned(),
            delete_task_on_completion: false,
        }
    }
}

impl QbittorrentConfig {
    fn validate(&self) -> Result<()> {
        if self.enabled {
            validate_url(&self.url, "downloader.qbittorrent.url")?;
            validate_not_empty(&self.username, "downloader.qbittorrent.username")?;
            validate_not_empty(&self.password, "downloader.qbittorrent.password")?;
            validate_abs_path_format(&self.download_dir, "downloader.qbittorrent.download_dir")?;
        }
        Ok(())
    }
}
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct ParserConfig {
    pub raw: RawParserConfig,
    pub siliconflow: SiliconflowConfig,
    pub deepseek: DeepseekConfig,
    pub deepbricks: DeepbricksConfig,
}

impl ParserConfig {
    fn validate(&self) -> Result<()> {
        self.siliconflow.validate()?;
        self.deepseek.validate()?;
        self.deepbricks.validate()?;

        if !self.raw.enabled
            && !self.siliconflow.enabled
            && !self.deepseek.enabled
            && !self.deepbricks.enabled
        {
            return Err(anyhow::anyhow!("至少需要启用一个解析器"));
        }
        Ok(())
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct RawParserConfig {
    pub enabled: bool,
}

impl Default for RawParserConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct SiliconflowConfig {
    pub enabled: bool,
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

impl Default for SiliconflowConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: "".to_owned(),
            model: "Pro/deepseek-ai/DeepSeek-V3".to_owned(),
            base_url: "https://api.siliconflow.com".to_owned(),
        }
    }
}

impl SiliconflowConfig {
    fn validate(&self) -> Result<()> {
        if self.enabled {
            validate_not_empty(&self.api_key, "parser.siliconflow.api_key")?;
            validate_not_empty(&self.model, "parser.siliconflow.model")?;
            validate_url(&self.base_url, "parser.siliconflow.base_url")?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct DeepseekConfig {
    pub enabled: bool,
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

impl Default for DeepseekConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: "".to_owned(),
            model: "deepseek-chat".to_owned(),
            base_url: "https://api.deepseek.com".to_owned(),
        }
    }
}

impl DeepseekConfig {
    fn validate(&self) -> Result<()> {
        if self.enabled {
            validate_not_empty(&self.api_key, "parser.deepseek.api_key")?;
            validate_not_empty(&self.model, "parser.deepseek.model")?;
            validate_url(&self.base_url, "parser.deepseek.base_url")?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct DeepbricksConfig {
    pub enabled: bool,
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

impl Default for DeepbricksConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: "".to_owned(),
            model: "gpt-4o-mini".to_owned(),
            base_url: "https://api.deepbricks.ai".to_owned(),
        }
    }
}

impl DeepbricksConfig {
    fn validate(&self) -> Result<()> {
        if self.enabled {
            validate_not_empty(&self.api_key, "parser.deepbricks.api_key")?;
            validate_not_empty(&self.model, "parser.deepbricks.model")?;
            validate_url(&self.base_url, "parser.deepbricks.base_url")?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct LogConfig {
    pub level: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_owned(),
        }
    }
}

impl LogConfig {
    fn validate(&self) -> Result<()> {
        match self.level.as_str() {
            "debug" | "info" | "warn" | "error" | "trace" => Ok(()),
            _ => Err(anyhow::anyhow!(
                "log.level 必须是 debug, info, warn, error, trace 中的一个"
            )),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ServerConfig {
    pub assets_path: String,
    pub listen_addr: String,
    pub database_url: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            assets_path: "./assets".to_owned(),
            listen_addr: "0.0.0.0:3001".to_owned(),
            database_url: "mysql://root:123456@localhost:3306/bangumi".to_owned(),
        }
    }
}

impl ServerConfig {
    fn validate(&self) -> Result<()> {
        validate_listen_addr(&self.listen_addr, "server.listen_addr")?;
        validate_url(&self.database_url, "server.database_url")?;
        validate_not_empty(&self.assets_path, "server.assets_path")?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct MikanConfig {
    pub endpoint: String,
}

impl Default for MikanConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://mikanani.me".to_owned(),
        }
    }
}

impl MikanConfig {
    fn validate(&self) -> Result<()> {
        validate_url(&self.endpoint, "mikan.endpoint")?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct TMDBConfig {
    pub api_key: String,
    pub base_url: String,
    pub image_base_url: String,
    pub language: String,
}

impl Default for TMDBConfig {
    fn default() -> Self {
        Self {
            api_key: "".to_owned(),
            base_url: "https://api.themoviedb.org/3".to_owned(),
            image_base_url: "https://image.tmdb.org/t/p/original/".to_owned(),
            language: "zh-CN".to_owned(),
        }
    }
}

impl TMDBConfig {
    fn validate(&self) -> Result<()> {
        validate_url(&self.base_url, "tmdb.base_url")?;
        validate_url(&self.image_base_url, "tmdb.image_base_url")?;
        validate_not_empty(&self.api_key, "tmdb.api_key")?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BangumiTvConfig {
    pub endpoint: String,
    pub image_base_url: String,
}

impl Default for BangumiTvConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://api.bgm.tv".to_owned(),
            image_base_url: "https://lain.bgm.tv".to_owned(),
        }
    }
}

impl BangumiTvConfig {
    fn validate(&self) -> Result<()> {
        validate_url(&self.endpoint, "bangumi_tv.endpoint")?;
        validate_url(&self.image_base_url, "bangumi_tv.image_base_url")?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct SentryConfig {
    pub enabled: bool,
    pub dsn: String,
}

impl Default for SentryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            dsn: "".to_owned(),
        }
    }
}

impl SentryConfig {
    fn validate(&self) -> Result<()> {
        if self.enabled {
            validate_url(&self.dsn, "sentry.dsn")?;
        }
        Ok(())
    }
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

pub trait Writer: Send + Sync {
    fn write(&self, config: &Config) -> Result<()>;
}

pub struct NopWriter;

impl Writer for NopWriter {
    fn write(&self, _config: &Config) -> Result<()> {
        Ok(())
    }
}

fn validate_url(url: &str, field_name: &str) -> Result<()> {
    let _ = Url::parse(url)
        .map_err(|e| anyhow::anyhow!("{}:{} 不是有效的 URL: {}", field_name, url, e))?;
    Ok(())
}

fn validate_listen_addr(listen_addr: &str, field_name: &str) -> Result<()> {
    let _ = SocketAddr::from_pathname(listen_addr)
        .map_err(|e| anyhow::anyhow!("{}:{} 不是有效的监听地址: {}", field_name, listen_addr, e))?;
    Ok(())
}

fn validate_not_empty(s: &str, field_name: &str) -> Result<()> {
    if s.is_empty() {
        return Err(anyhow::anyhow!("{}:{} 不能为空", field_name, s));
    }
    Ok(())
}

fn validate_abs_path_format(path: &str, field_name: &str) -> Result<()> {
    let path = Path::new(path);
    if !path.is_absolute() {
        return Err(anyhow::anyhow!(
            "{}:{} 路径不是绝对路径",
            field_name,
            path.to_string_lossy()
        ));
    }
    Ok(())
}
