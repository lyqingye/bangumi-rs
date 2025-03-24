#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("找不到对象")]
    ObjectNotFound,

    #[error("请求失败: {0} {1}")]
    RequestFailed(String, String),

    #[error("响应错误: {0}")]
    ResponseError(String),

    #[error("登录失败: {0}")]
    LoginFailed(String),

    #[error("访问拒绝: {0}")]
    AccessDenied(String),

    #[error("服务器错误: {0}")]
    ServerError(String),

    #[error("请求参数有误: {0}")]
    BadRequest(String),
}

pub type Result<T> = std::result::Result<T, Error>;
