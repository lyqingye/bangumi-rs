mod auth;
/// AList API封装库
///
/// 这个库提供了对AList任务管理API的封装，包括以下功能：
/// - 上传任务管理
/// - 复制任务管理
/// - 离线下载任务管理
/// - 离线下载转存任务管理
/// - 解压任务管理
/// - 解压转存任务管理
/// - 用户认证与权限
/// - 文件系统操作
///
/// 每种任务类型都支持以下操作：
/// - 获取任务信息
/// - 获取已完成/未完成任务
/// - 删除任务
/// - 取消任务
/// - 清除已完成/已成功任务
/// - 重试任务
/// - 批量操作任务
///
/// 文件系统操作包括：
/// - 获取文件列表
/// - 获取文件下载链接
///
/// # 例子
///
/// ```rust,no_run
/// use alist::{AListClient, TaskType, Response, TaskInfo};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // 登录并获取token
///     let client = AListClient::new("https://alist.example.com", "");
///     let login_result = client.login("admin", "password", None).await?;
///     let token = login_result.data.token;
///     
///     // 使用token创建新客户端
///     let client = AListClient::new("https://alist.example.com", token);
///     
///     // 获取所有上传任务
///     let upload_tasks = client.get_task_info(TaskType::Upload, None).await?;
///     
///     // 获取特定任务详情
///     let task_id = "task_id";
///     let task_info = client.get_task_info(TaskType::Upload, Some(task_id)).await?;
///     
///     // 取消任务
///     client.cancel_task(TaskType::Upload, task_id).await?;
///     
///     Ok(())
/// }
/// ```
mod client;
mod error;
mod fs;
mod model;

pub use client::AListClient;
pub use error::*;
pub use model::*;
