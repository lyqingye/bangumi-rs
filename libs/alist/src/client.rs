use crate::{
    model::{Response, TaskInfo, TaskType},
    AddOfflineDownloadTaskRequest, AddOfflineDownloadTaskResult,
};
use crate::{Error, Result};
use reqwest::{header, Client};
use serde::de::DeserializeOwned;
use tracing::{debug, instrument};

#[derive(Debug, Clone)]
pub struct AListClient {
    client: Client,
    pub(crate) base_url: String,
    pub(crate) token: String,
    pub(crate) user_name: String,
    pub(crate) user_password: String,
}

impl AListClient {
    pub fn new(
        base_url: impl Into<String>,
        user_name: impl Into<String>,
        user_password: impl Into<String>,
    ) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            token: String::new(),
            user_name: user_name.into(),
            user_password: user_password.into(),
        }
    }

    fn build_task_url(&self, task_type: TaskType, action: &str) -> String {
        format!(
            "{}/api/task/{}/{}",
            self.base_url.trim_end_matches('/'),
            task_type.as_str(),
            action
        )
    }

    fn create_auth_headers(&self) -> header::HeaderMap {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&self.token)
                .unwrap_or_else(|_| header::HeaderValue::from_static("")),
        );
        headers
    }

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

    #[instrument(skip(self), err)]
    pub async fn delete_task(&self, task_type: TaskType, task_id: &str) -> Result<()> {
        let url = self.build_task_url(task_type, "delete");
        let query = [("tid", task_id)];
        map_result_allow_404(self.post(&url, &query).await)
    }

    #[instrument(skip(self), err)]
    pub async fn cancel_task(&self, task_type: TaskType, task_id: &str) -> Result<()> {
        let url = self.build_task_url(task_type, "cancel");
        let query = [("tid", task_id)];
        map_result_allow_404(self.post(&url, &query).await)
    }

    #[instrument(skip(self), err)]
    pub async fn retry_task(&self, task_type: TaskType, task_id: &str) -> Result<()> {
        let url = self.build_task_url(task_type, "retry");
        let query = [("tid", task_id)];
        map_result_allow_404(self.post(&url, &query).await)
    }

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

fn map_result_allow_404(result: Result<()>) -> Result<()> {
    match result {
        Ok(_) => Ok(()),
        Err(Error::ObjectNotFound) => Ok(()),
        Err(e) => Err(e),
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
        let mut client = AListClient::new("http://localhost:5244", "admin", "123456");
        client.login().await.unwrap();
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
