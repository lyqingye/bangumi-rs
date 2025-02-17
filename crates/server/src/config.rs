use serde::Deserialize;

const DEFAULT_LOG_LEVEL: &str = "info";
const DEFAULT_ASSETS_PATH: &str = "./assets";
const DEFAULT_LISTEN_ADDR: &str = "127.0.0.1:3000";
const DEFAULT_DATABASE_URL: &str = "mysql://root:123456@192.168.1.10:3306/bangumi";
const DEFAULT_MIKAN_ENDPOINT: &str = "https://mikanani.me";
const DEFAULT_BANGUMI_TV_ENDPOINT: &str = "https://api.bgm.tv";
const DEFAULT_TMDB_BASE_URL: &str = "https://api.themoviedb.org/3";
const DEFAULT_TMDB_IMAGE_BASE_URL: &str = "https://image.tmdb.org/t/p/original/";
const DEFAULT_TMDB_LANGUAGE: &str = "zh-CN";

#[derive(Debug, Deserialize, Clone, Default)]
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

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub http: String,
    pub https: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct NotifyConfig {
    pub telegram: TelegramConfig,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TelegramConfig {
    pub enabled: bool,
    pub token: String,
    pub chat_id: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Pan115Config {
    pub cookies: String,
    pub download_dir: String,
    pub max_requests_per_second: u32,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ParserConfig {
    pub siliconflow: SiliconflowConfig,
    pub deepseek: DeepseekConfig,
    pub deepbricks: DeepbricksConfig,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SiliconflowConfig {
    pub enabled: bool,
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct DeepseekConfig {
    pub enabled: bool,
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct DeepbricksConfig {
    pub enabled: bool,
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct LogConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ServerConfig {
    #[serde(default = "default_assets_path")]
    pub assets_path: String,
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,
    #[serde(default = "default_database_url")]
    pub database_url: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct MikanConfig {
    #[serde(default = "default_mikan_endpoint")]
    pub endpoint: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
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

#[derive(Debug, Deserialize, Clone, Default)]
pub struct BangumiTvConfig {
    #[serde(default = "default_bangumi_tv_endpoint")]
    pub endpoint: String,
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
