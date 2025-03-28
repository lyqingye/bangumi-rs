use model::sea_orm_active_enums::ResourceType;

use crate::actor;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("任务不存在: info_hash={0}")]
    TaskNotFound(String),

    #[error("资源不存在: info_hash={0}")]
    ResourceNotFound(String),

    #[error("种子文件不存在: info_hash={0}")]
    TorrentNotFound(String),

    #[error("不支持的资源类型: {0}")]
    UnsupportedResourceType(ResourceType),

    #[error("种子文件为空")]
    EmptyTorrent,

    #[error("磁力链接为空")]
    EmptyMagnet,

    #[error("磁力链接格式错误: {0}")]
    MagnetFormat(String),

    #[error("InfoHash格式错误: {0}")]
    InfoHashFormat(String),

    #[error("种子文件URL为空: info_hash={0}")]
    EmptyTorrentUrl(String),

    #[error("指定的下载器不存在: {0}")]
    DownloaderNotFound(String),

    #[error("数据库错误: {0}")]
    DB(#[from] sea_orm::DbErr),

    #[error("错误: {0}")]
    Generic(#[from] anyhow::Error),

    #[error("停止Actor超时")]
    ShutdownTimeout,

    #[error("通道已关闭")]
    ChannelClosed,

    #[error("文件ID格式错误: {0}")]
    InvalidFileId(String),

    #[error("没有下载结果: {0}")]
    NoDownloadResult(String),

    #[error("文件不存在: {0}")]
    FileNotFound(String),

    #[error("序列化错误: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("qBittorrent错误: {0}")]
    Qbittorrent(#[from] qbittorrent::error::Error),

    #[error("Transmission错误: {0}")]
    Transmission(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("Pan115错误: {0}")]
    Pan115(#[from] pan_115::errors::Pan115Error),

    #[error("Alist错误: {0}")]
    Alist(#[from] alist::Error),

    #[error("URL解析错误: {0}")]
    URLParse(#[from] url::ParseError),

    #[error("发送事件失败: {0}")]
    SendEvent(#[from] tokio::sync::mpsc::error::SendError<actor::Tx>),

    #[error("非法的下载目录: {0}")]
    DownloadDir(String),
}

pub type Result<T> = std::result::Result<T, Error>;
