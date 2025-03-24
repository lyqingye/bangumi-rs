use crate::{
    model::{Response, TaskInfo, TaskType},
    AddOfflineDownloadTaskRequest, AddOfflineDownloadTaskResult,
};
use crate::{Error, Result};
use reqwest::{header, Client};
use serde::de::DeserializeOwned;
use tracing::{debug, instrument};

/// AList API任务管理客户端
#[derive(Debug, Clone)]
pub struct AListClient {
    client: Client,
    pub(crate) base_url: String,
    token: String,
}

impl AListClient {
    /// 创建新的AList API客户端
    pub fn new(base_url: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            token: token.into(),
        }
    }

    /// 构建任务API的URL
    fn build_task_url(&self, task_type: TaskType, action: &str) -> String {
        format!(
            "{}/api/task/{}/{}",
            self.base_url.trim_end_matches('/'),
            task_type.as_str(),
            action
        )
    }

    /// 创建带认证信息的请求头
    fn create_auth_headers(&self) -> header::HeaderMap {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&self.token)
                .unwrap_or_else(|_| header::HeaderValue::from_static("")),
        );
        headers
    }

    /// 发送GET请求
    #[instrument(skip(self), err)]
    pub(crate) async fn get<T>(&self, url: &str) -> Result<T>
    where
        T: DeserializeOwned + Default,
    {
        let response = self
            .client
            .get(url)
            .headers(self.create_auth_headers())
            .send()
            .await
            .map_err(|e| Error::RequestFailed(url.to_owned(), e.to_string()))?;

        self.handle_response(response).await
    }

    /// 发送POST请求
    #[instrument(skip(self), err)]
    async fn post<T>(&self, url: &str, query: &[(&str, &str)]) -> Result<T>
    where
        T: DeserializeOwned + Default,
    {
        let response = self
            .client
            .post(url)
            .headers(self.create_auth_headers())
            .query(query)
            .send()
            .await
            .map_err(|e| Error::RequestFailed(url.to_owned(), e.to_string()))?;

        self.handle_response(response).await
    }

    /// 发送带请求体的POST请求
    #[instrument(skip(self, body), err)]
    pub(crate) async fn post_json<T, B>(&self, url: &str, body: &B) -> Result<T>
    where
        T: DeserializeOwned + Default,
        B: serde::Serialize + std::fmt::Debug,
    {
        let response = self
            .client
            .post(url)
            .headers(self.create_auth_headers())
            .json(body)
            .send()
            .await
            .map_err(|e| Error::RequestFailed(url.to_owned(), e.to_string()))?;

        self.handle_response(response).await
    }

    /// 处理API响应
    async fn handle_response<T>(&self, response: reqwest::Response) -> Result<T>
    where
        T: DeserializeOwned + Default,
    {
        let body = response
            .text()
            .await
            .map_err(|e| Error::ResponseError(e.to_string()))?;

        debug!("响应内容: {}", body);

        let response: Response<T> = serde_json::from_str(&body)
            .map_err(|e| Error::ResponseError(format!("解析响应内容失败: {}", e)))?;

        if response.code != 200 {
            if response.code == 404 {
                return Err(Error::ObjectNotFound);
            }

            if response.code == 403 {
                return Err(Error::AccessDenied(response.message));
            }

            if response.code == 400 {
                return Err(Error::BadRequest(response.message));
            }

            if response.code == 500 {
                return Err(Error::ServerError(response.message));
            }

            return Err(Error::ResponseError(format!(
                "API请求失败，状态码: {}, 响应内容: {}",
                response.code, response.message
            )));
        }

        Ok(response.data.unwrap_or_default())
    }

    /// 获取任务信息
    #[instrument(skip(self), err)]
    pub async fn get_task_info(
        &self,
        task_type: TaskType,
        task_id: &str,
    ) -> Result<Option<TaskInfo>> {
        let url = self.build_task_url(task_type, "info");
        let mut query = Vec::new();
        query.push(("tid", task_id));

        self.post(&url, &query).await
    }

    /// 获取已完成任务
    #[instrument(skip(self), err)]
    pub async fn get_done_tasks(&self, task_type: TaskType) -> Result<Vec<TaskInfo>> {
        let url = self.build_task_url(task_type, "done");
        self.get(&url).await
    }

    /// 获取未完成任务
    #[instrument(skip(self), err)]
    pub async fn get_undone_tasks(&self, task_type: TaskType) -> Result<Option<Vec<TaskInfo>>> {
        let url = self.build_task_url(task_type, "undone");
        self.get(&url).await
    }

    /// 删除任务
    #[instrument(skip(self), err)]
    pub async fn delete_task(&self, task_type: TaskType, task_id: &str) -> Result<()> {
        let url = self.build_task_url(task_type, "delete");
        let query = [("tid", task_id)];
        self.post(&url, &query).await
    }

    /// 取消任务
    #[instrument(skip(self), err)]
    pub async fn cancel_task(&self, task_type: TaskType, task_id: &str) -> Result<()> {
        let url = self.build_task_url(task_type, "cancel");
        let query = [("tid", task_id)];
        self.post(&url, &query).await
    }

    /// 重试任务
    #[instrument(skip(self), err)]
    pub async fn retry_task(&self, task_type: TaskType, task_id: &str) -> Result<()> {
        let url = self.build_task_url(task_type, "retry");
        let query = [("tid", task_id)];
        self.post(&url, &query).await
    }

    /// 重试已失败任务
    #[instrument(skip(self), err)]
    pub async fn retry_failed_tasks(&self, task_type: TaskType) -> Result<()> {
        let url = self.build_task_url(task_type, "retry_failed");
        self.post(&url, &[]).await
    }

    /// 添加离线下载任务
    #[instrument(skip(self), err)]
    pub async fn add_offline_download_task(
        &self,
        request: AddOfflineDownloadTaskRequest,
    ) -> Result<AddOfflineDownloadTaskResult> {
        let url = format!(
            "{}/api/fs/add_offline_download",
            self.base_url.trim_end_matches('/'),
        );
        self.post_json(&url, &request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_client() -> AListClient {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_target(true) // 不显示目标模块
            .init();
        let mut client = AListClient::new("http://localhost:5244", "");
        let result = client
            .login("admin", "123456", None::<String>)
            .await
            .unwrap();
        client.token = result.token;
        client
    }

    #[ignore]
    #[tokio::test]
    async fn test_login() {
        let client = create_client().await;
        println!("{:?}", client.token);
    }

    #[ignore]
    #[tokio::test]
    async fn test_add_offline_download_task() {
        let client = create_client().await;
        let request = AddOfflineDownloadTaskRequest {
            urls: vec!["magnet:?xt=urn:btih:1a76803252df64609b68e00c14b0642c6c5eaa39".to_string()],
            path: "/downloads".to_string(),
            tool: "115 Cloud".to_string(),
            delete_policy: "delete_never".to_string(),
        };
        let result = client.add_offline_download_task(request).await;
        println!("{:?}", result);
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_task_info() {
        let client = create_client().await;
        let result = client
            .get_task_info(TaskType::OfflineDownload, "joGXMIxokczmDd_Rk2W1f")
            .await
            .unwrap();
        println!("{:?}", result);
    }

    #[ignore]
    #[tokio::test]
    async fn test_retry_task() {
        let client = create_client().await;
        let result = client
            .retry_task(TaskType::OfflineDownload, "uC5XNSea-gFdgaP-i0S5L")
            .await;
        println!("{:?}", result);
    }

    #[ignore]
    #[tokio::test]
    async fn test_cancel_task() {
        let client = create_client().await;
        let result = client
            .cancel_task(TaskType::OfflineDownload, "k9xXGXtCI4EhOn3EeaECJ")
            .await;
        println!("{:?}", result);
    }

    #[ignore]
    #[tokio::test]
    async fn test_delete_task() {
        let client = create_client().await;
        let result = client
            .delete_task(TaskType::OfflineDownload, "k9xXGXtCI4EhOn3EeaECJ")
            .await;
        println!("{:?}", result);
    }
}
