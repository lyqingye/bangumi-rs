use std::{
    collections::{HashMap, HashSet},
    time::{SystemTime, UNIX_EPOCH},
};

use tracing::debug;

use super::{
    client::{Client, APP_VER},
    decode, encode,
    errors::Pan115Error,
    gen_key,
    model::{
        BasicResp, DownloadResp, OfflineAddUrlResponse, OfflineTask, OfflineTaskResp,
        OfflineTaskResponse,
    },
};

pub const API_LIST_OFFLINE_TASKS: &str = "https://lixian.115.com/lixian/?ct=lixian&ac=task_lists";
pub const API_DELETE_OFFLINE_TASK: &str = "https://lixian.115.com/lixian/?ct=lixian&ac=task_del";
pub const API_ADD_OFFLINE_TASK: &str = "https://lixian.115.com/lixianssp/?ac=add_task_urls";

impl Client {
    pub async fn list_offline_tasks_page(&self, page: i64) -> Result<OfflineTaskResp, Pan115Error> {
        self.acquire().await;

        let resp: OfflineTaskResp = self
            .cli
            .post(API_LIST_OFFLINE_TASKS)
            .query(&[("page", page.to_string().as_str())])
            .send()
            .await?
            .json()
            .await?;

        resp.basic_resp.is_ok()?;

        Ok(resp)
    }

    pub async fn list_offline_tasks_by_hashes<T: AsRef<str>>(
        &self,
        hashes: &[T],
    ) -> Result<Vec<OfflineTask>, Pan115Error> {
        self.acquire().await;

        let mut tasks = vec![];
        let mut page = 0;
        let target_count = hashes.len();
        let hash_set: HashSet<&str> = hashes.iter().map(|h| h.as_ref()).collect();
        loop {
            let tasks_page = self.list_offline_tasks_page(page).await?;

            for task in tasks_page.tasks {
                if hash_set.contains(task.info_hash.as_str()) {
                    tasks.push(task);

                    if tasks.len() == target_count {
                        return Ok(tasks);
                    }
                }
            }

            if tasks_page.page_count == tasks_page.page {
                break;
            }

            page = tasks_page.page + 1;
        }

        Ok(tasks)
    }

    pub async fn delete_offline_task<T: AsRef<str>>(
        &self,
        hashes: &[T],
        delete_files: bool,
    ) -> Result<(), Pan115Error> {
        self.acquire().await;

        let mut form = HashMap::new();

        for hash in hashes {
            form.insert("hash".to_owned(), hash.as_ref());
        }

        if delete_files {
            form.insert("flag".to_owned(), "1");
        } else {
            form.insert("flag".to_owned(), "0");
        }

        let resp: BasicResp = self
            .cli
            .post(API_DELETE_OFFLINE_TASK)
            .form(&form)
            .send()
            .await?
            .json()
            .await?;

        resp.is_ok()?;

        Ok(())
    }

    pub async fn add_offline_task<T: AsRef<str>>(
        &self,
        urls: &[T],
        save_dir_id: T,
    ) -> Result<Vec<OfflineTaskResponse>, Pan115Error> {
        debug!(
            "add_offline_task: {:?} {:?}",
            save_dir_id.as_ref(),
            urls.iter().map(|u| u.as_ref()).collect::<Vec<&str>>()
        );
        self.acquire().await;

        if urls.is_empty() {
            return Ok(vec![]);
        }

        // 1. 构建请求参数
        let mut params = HashMap::new();

        for (i, url) in urls.iter().enumerate() {
            let key = format!("url[{}]", i);
            params.insert(key, url.as_ref());
        }
        params.insert("ac".to_owned(), "add_task_urls");
        params.insert("wp_path_id".to_owned(), save_dir_id.as_ref());
        params.insert("app_ver".to_owned(), APP_VER);
        let uid = self.user_id.to_string();
        params.insert("uid".to_owned(), uid.as_str());

        // 2. 加密参数
        let key = gen_key();
        let data = serde_json::to_vec(&params).unwrap();

        let mut form = HashMap::new();
        form.insert("data".to_owned(), encode(&data, &key));

        // 3. 发送请求
        let resp: DownloadResp = self
            .cli
            .post(API_ADD_OFFLINE_TASK)
            .query(&[(
                "_",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .to_string(),
            )])
            .form(&form)
            .send()
            .await?
            .json()
            .await?;

        resp.basic_resp.is_ok()?;

        // 4. 解密返回数据
        let data = decode(&resp.data, &key)?;

        let resp: OfflineAddUrlResponse = serde_json::from_slice(&data)?;
        resp.basic_resp.is_ok()?;

        Ok(resp.result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use reqwest::{header, Url};
    use std::{env, sync::Arc};
    use tokio_stream::{self as stream, StreamExt};

    async fn create_client() -> Result<Client> {
        dotenv::dotenv().ok();
        let mut client = Client::new_from_env()?;
        Ok(client)
    }

    #[tokio::test]
    #[ignore]
    async fn test_list_offline_tasks() -> Result<()> {
        let mut client = create_client().await?;
        client.login_check().await?;
        let tasks = client.list_offline_tasks_page(1).await?;
        println!("{:?}", tasks);
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_delete_offline_task() -> Result<()> {
        let mut client = create_client().await?;
        client.login_check().await?;
        client
            .delete_offline_task(&["1a7639fe4fd0afdbbb98f616a64f258d6b7a37af"], true)
            .await?;
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_add_offline_task() -> Result<()> {
        let mut client = create_client().await?;
        client.login_check().await?;
        client
            .add_offline_task(
                &["magnet:?xt=urn:btih:cf778b1c9b25ae87b5629e405b290df602aa9036"],
                "0",
            )
            .await?;
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_list_offline_tasks_by_hashes() -> Result<()> {
        let mut client = create_client().await?;
        client.login_check().await?;
        let resp = client
            .list_offline_tasks_by_hashes(&["d64ce4bb6b17a66cabf85ac2fad68bbf9e71abbb"])
            .await?;
        println!("{:?}", resp);
        Ok(())
    }
}
