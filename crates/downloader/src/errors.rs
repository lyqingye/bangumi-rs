use crate::actor;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("任务不存在: info_hash={0}")]
    TaskNotFound(String),

    #[error("资源不存在: info_hash={0}")]
    ResourceNotFound(String),

    #[error("种子文件不存在: info_hash={0}")]
    TorrentNotFound(String),

    #[error("种子文件为空")]
    EmptyTorrent,

    #[error("磁力链接为空")]
    EmptyMagnet,

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

    #[error("序列化错误: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("qBittorrent错误: {0}")]
    Qbittorrent(#[from] qbittorrent::error::Error),

    #[error("Pan115错误: {0}")]
    Pan115(#[from] pan_115::errors::Pan115Error),

    #[error("Alist错误: {0}")]
    Alist(#[from] alist::Error),

    #[error("URL解析错误: {0}")]
    URLParse(#[from] url::ParseError),

    #[error("发送事件失败: {0}")]
    SendEvent(#[from] tokio::sync::mpsc::error::SendError<actor::Tx>),
}

pub type Result<T> = std::result::Result<T, Error>;
