use crate::{
    model::{BatchOperationRequest, BatchOperationResult, Response, TaskInfo, TaskType},
    AddOfflineDownloadTaskRequest, AddOfflineDownloadTaskResult,
};
use anyhow::{Context, Result};
use reqwest::{header, Client, StatusCode};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
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
    pub(crate) async fn get<T>(&self, url: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned + Default,
    {
        let response = self
            .client
            .get(url)
            .headers(self.create_auth_headers())
            .send()
            .await
            .context("发送请求失败")?;

        self.handle_response(response).await
    }

    /// 发送POST请求
    #[instrument(skip(self), err)]
    async fn post<T>(&self, url: &str, query: &[(&str, &str)]) -> Result<Option<T>>
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
            .context("发送请求失败")?;

        self.handle_response(response).await
    }

    /// 发送带请求体的POST请求
    #[instrument(skip(self, body), err)]
    pub(crate) async fn post_json<T, B>(&self, url: &str, body: &B) -> Result<Option<T>>
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
            .context("发送请求失败")?;

        self.handle_response(response).await
    }

    /// 处理API响应
    async fn handle_response<T>(&self, response: reqwest::Response) -> Result<Option<T>>
    where
        T: DeserializeOwned + Default,
    {
        let status = response.status();
        let body = response.text().await.context("读取响应内容失败")?;

        if status != StatusCode::OK {
            anyhow::bail!("API请求失败，状态码: {}, 响应内容: {}", status, body);
        }

        debug!("响应内容: {}", body);

        let response: Response<T> = serde_json::from_str(&body).context("解析响应内容失败")?;

        if response.code != 200 {
            if response.code == 404 {
                return Ok(None);
            }
            anyhow::bail!(
                "API请求失败，状态码: {}, 响应内容: {}",
                response.code,
                response.message
            );
        }

        Ok(response.data)
    }

    /// 获取任务信息
    #[instrument(skip(self), err)]
    pub async fn get_task_info(&self, task_type: TaskType, task_id: &str) -> Result<Option<TaskInfo>> {
        let url = self.build_task_url(task_type, "info");
        let mut query = Vec::new();
        query.push(("tid", task_id));

        self.post(&url, &query).await
    }

    /// 获取已完成任务
    #[instrument(skip(self), err)]
    pub async fn get_done_tasks(&self, task_type: TaskType) -> Result<Option<Vec<TaskInfo>>> {
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
    pub async fn delete_task(&self, task_type: TaskType, task_id: &str) -> Result<Option<()>> {
        let url = self.build_task_url(task_type, "delete");
        let query = [("tid", task_id)];
        self.post(&url, &query).await
    }

    /// 取消任务
    #[instrument(skip(self), err)]
    pub async fn cancel_task(&self, task_type: TaskType, task_id: &str) -> Result<Option<()>> {
        let url = self.build_task_url(task_type, "cancel");
        let query = [("tid", task_id)];
        self.post(&url, &query).await
    }

    /// 清除已完成任务
    #[instrument(skip(self), err)]
    pub async fn clear_done_tasks(&self, task_type: TaskType) -> Result<Option<()>> {
        let url = self.build_task_url(task_type, "clear_done");
        self.post(&url, &[]).await
    }

    /// 清除已成功任务
    #[instrument(skip(self), err)]
    pub async fn clear_succeeded_tasks(&self, task_type: TaskType) -> Result<Option<()>> {
        let url = self.build_task_url(task_type, "clear_succeeded");
        self.post(&url, &[]).await
    }

    /// 重试任务
    #[instrument(skip(self), err)]
    pub async fn retry_task(&self, task_type: TaskType, task_id: &str) -> Result<Option<()>> {
        let url = self.build_task_url(task_type, "retry");
        let query = [("tid", task_id)];
        self.post(&url, &query).await
    }

    /// 重试已失败任务
    #[instrument(skip(self), err)]
    pub async fn retry_failed_tasks(&self, task_type: TaskType) -> Result<Option<()>> {
        let url = self.build_task_url(task_type, "retry_failed");
        self.post(&url, &[]).await
    }

    /// 删除多个任务
    #[instrument(skip(self), err)]
    pub async fn delete_some_tasks(
        &self,
        task_type: TaskType,
        task_ids: Vec<String>,
    ) -> Result<Option<BatchOperationResult>> {
        let url = self.build_task_url(task_type, "delete_some");
        self.post_json(&url, &task_ids).await
    }

    /// 取消多个任务
    #[instrument(skip(self), err)]
    pub async fn cancel_some_tasks(
        &self,
        task_type: TaskType,
        task_ids: Vec<String>,
    ) -> Result<Option<BatchOperationResult>> {
        let url = self.build_task_url(task_type, "cancel_some");
        self.post_json(&url, &task_ids).await
    }

    /// 重试多个任务
    #[instrument(skip(self), err)]
    pub async fn retry_some_tasks(
        &self,
        task_type: TaskType,
        task_ids: Vec<String>,
    ) -> Result<Option<BatchOperationResult>> {
        let url = self.build_task_url(task_type, "retry_some");
        self.post_json(&url, &task_ids).await
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
        client.token = result.unwrap().token;
        client
    }

    #[tokio::test]
    async fn test_add_offline_download_task() {
        let client = create_client().await;
        let request = AddOfflineDownloadTaskRequest {
            urls: vec!["https://mikanani.me/Download/20250316/d870ebb2618dd3e8e7aaa27c662d1d8982d20104.torrent".to_string()],
            path: "/downloads".to_string(),
            tool: "qBittorrent".to_string(),
            delete_policy: "delete_never".to_string(),
        };
        let result = client.add_offline_download_task(request).await;
        println!("{:?}", result);
    }

    #[tokio::test]
    async fn test_get_task_info() {
        let client = create_client().await;
        let result = client
            .get_task_info(TaskType::OfflineDownload, "9qGPePxMwNyzSekcpKWgb")
            .await
            .unwrap();
        println!("{:?}", result);
    }
}
