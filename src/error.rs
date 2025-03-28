use thiserror::Error;

#[derive(Debug, Error)]
pub enum DownloaderError {
    #[error("下载任务不存在: {0}")]
    TaskNotFound(String),

    #[error("下载器不存在: {0}")]
    DownloaderNotFound(String),

    #[error("资源格式错误: {0}")]
    InvalidResource(String),

    #[error("第三方下载失败: {0}")]
    ThirdPartyError(String),

    #[error("数据库错误: {0}")]
    DatabaseError(String),
}
