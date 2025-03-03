use std::collections::HashMap;

use super::{
    client::Client,
    errors::Pan115Error,
    iter::FileStream,
    model::{BasicResp, FileInfo, FileListResp, GetFileInfoResponse},
};
use anyhow::Result;
use tracing::debug;

pub const API_LIST_FILES: &str = "https://webapi.115.com/files";
pub const API_MOVE_FILES: &str = "https://webapi.115.com/files/move";
pub const API_DELETE_FILES: &str = "https://webapi.115.com/rb/delete";
pub const API_RENAME_FILES: &str = "https://webapi.115.com/files/batch_rename";
pub const API_FILE_INFO: &str = "https://webapi.115.com/files/get_info";

impl Client {
    pub async fn list_files(
        &self,
        cid: &str,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<FileInfo>, Pan115Error> {
        debug!("list_files: {:?} {:?} {:?}", cid, offset, limit);
        self.acquire().await;

        let resp: FileListResp = self
            .cli
            .get(API_LIST_FILES)
            .query(&[
                ("aid", "1"),
                ("cid", cid),
                ("o", "user_ptime"),
                ("asc", "1"),
                ("offset", offset.unwrap_or(0).to_string().as_str()),
                ("show_dir", "1"),
                ("limit", limit.unwrap_or(100).to_string().as_str()),
                ("snap", "0"),
                ("natsort", "0"),
                ("record_open_time", "1"),
                ("format", "json"),
                ("fc_mix", "0"),
            ])
            .send()
            .await?
            .json()
            .await?;

        resp.basic_resp.is_ok()?;

        Ok(resp.files)
    }

    pub fn list_files_stream<'a>(&'a self, cid: &'a str, page_size: i32) -> FileStream<'a> {
        FileStream::new(self, cid, page_size)
    }

    pub async fn move_files<T: AsRef<str>>(
        &self,
        file_ids: &[T],
        to_cid: T,
    ) -> Result<(), Pan115Error> {
        self.acquire().await;

        if file_ids.is_empty() {
            return Ok(());
        }
        let mut form = HashMap::new();
        form.insert("pid".to_owned(), to_cid.as_ref());

        for (i, file_id) in file_ids.iter().enumerate() {
            let key = format!("fid[{}]", i);
            form.insert(key, file_id.as_ref());
        }

        let resp: BasicResp = self
            .cli
            .post(API_MOVE_FILES)
            .form(&form)
            .send()
            .await?
            .json()
            .await?;

        resp.is_ok()?;

        Ok(())
    }

    pub async fn delete_files<T: AsRef<str>>(&self, file_ids: &[T]) -> Result<(), Pan115Error> {
        self.acquire().await;

        if file_ids.is_empty() {
            return Ok(());
        }
        let mut form = HashMap::new();

        for (i, file_id) in file_ids.iter().enumerate() {
            let key = format!("fid[{}]", i);
            form.insert(key, file_id.as_ref());
        }

        let resp: BasicResp = self
            .cli
            .post(API_DELETE_FILES)
            .form(&form)
            .send()
            .await?
            .json()
            .await?;

        resp.is_ok()?;

        Ok(())
    }

    pub async fn rename_file<T: AsRef<str>>(
        &self,
        file_id: T,
        new_name: T,
    ) -> Result<(), Pan115Error> {
        self.acquire().await;

        let mut form = HashMap::new();
        form.insert("fid".to_owned(), file_id.as_ref());
        form.insert("file_name".to_owned(), new_name.as_ref());
        form.insert(
            format!("files_new_name[{}]", file_id.as_ref()),
            new_name.as_ref(),
        );

        let resp: BasicResp = self
            .cli
            .post(API_RENAME_FILES)
            .form(&form)
            .send()
            .await?
            .json()
            .await?;

        resp.is_ok()?;

        Ok(())
    }

    pub async fn get_file<T: AsRef<str>>(
        &self,
        file_id: T,
    ) -> Result<Option<FileInfo>, Pan115Error> {
        debug!("get_file: {:?}", file_id.as_ref());

        self.acquire().await;

        let resp: GetFileInfoResponse = self
            .cli
            .get(API_FILE_INFO)
            .query(&[("file_id", file_id.as_ref())])
            .send()
            .await?
            .json()
            .await?;
        resp.basic_resp.is_ok()?;

        Ok(resp.files.first().cloned())
    }

    pub async fn list_files_with_fn<F>(
        &self,
        cid: &str,
        filter: F,
    ) -> Result<Vec<FileInfo>, Pan115Error>
    where
        F: Fn(&FileInfo) -> bool + Send + 'static,
    {
        let mut result = Vec::new();
        let mut offset = 0;
        let limit = 100;

        loop {
            let files = self.list_files(cid, Some(offset), Some(limit)).await?;
            if files.is_empty() {
                break;
            }

            let file_len = files.len();

            // 使用 filter 过滤文件
            for file in files {
                if filter(&file) {
                    result.push(file);
                }
            }
            offset += limit;

            if file_len < limit as usize {
                break;
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::env;
    use tokio_stream::StreamExt;

    async fn create_client() -> Result<Client> {
        dotenv::dotenv().ok();
        let mut client = Client::new_from_env()?;
        Ok(client)
    }

    #[tokio::test]
    #[ignore]
    async fn test_list_files() -> Result<()> {
        let mut client = create_client().await?;
        client.login_check().await?;
        let files = client
            .list_files("2958258599551302765", Some(0), Some(100))
            .await?;
        println!("{:?}", files);
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_list_files_stream() -> Result<()> {
        let mut client = create_client().await?;
        client.login_check().await?;
        let mut stream = client.list_files_stream("2958258599551302765", 100);
        while let Some(file) = stream.next().await {
            match file {
                Ok(file) => {
                    println!("{:?}", file);
                }
                Err(e) => {
                    println!("{:?}", e);
                }
            }
        }
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_move_files() -> Result<()> {
        let mut client = create_client().await?;
        client.login_check().await?;
        client.move_files(&["3076990460988751438"], "0").await?;
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_rename_file() -> Result<()> {
        let mut client = create_client().await?;
        client.login_check().await?;
        client
            .rename_file("3076990460988751438", "test.mp4")
            .await?;
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_delete_files() -> Result<()> {
        let mut client = create_client().await?;
        client.login_check().await?;
        client.delete_files(&["3076990460988751438"]).await?;
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_file_info() -> Result<()> {
        let mut client = create_client().await?;
        client.login_check().await?;
        let file_info = client.get_file("3083713467929067080").await?;
        println!("{:?}", file_info);
        Ok(())
    }
}
