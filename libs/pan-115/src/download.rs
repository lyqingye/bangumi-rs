use std::{
    collections::HashMap,
    ops::Deref,
    time::{SystemTime, UNIX_EPOCH},
};

use super::{
    client::Client,
    decode, encode,
    errors::Pan115Error,
    gen_key,
    model::{DownloadData, DownloadInfo, DownloadResp},
};
use anyhow::Result;
use reqwest::{header::USER_AGENT, Url};
use serde_json::json;

const API_DOWNLOAD: &str = "https://proapi.115.com/app/chrome/downurl";

impl Client {
    pub async fn download<T: AsRef<str>>(
        &self,
        pick_code: T,
        ua: T,
    ) -> Result<Option<DownloadInfo>, Pan115Error> {
        self.acquire().await;

        let key = gen_key();
        let params = serde_json::to_vec(&json!(
            {
                "pickcode": pick_code.as_ref(),
            }
        ))?;
        let data = encode(&params, &key);

        let mut form = HashMap::new();
        form.insert("data", data);

        let http_resp = self
            .cli
            .post(API_DOWNLOAD)
            .header(USER_AGENT, ua.as_ref())
            .query(&[(
                "t",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .to_string(),
            )])
            .form(&form)
            .send()
            .await?;
        let header = http_resp.headers().clone();
        let resp: DownloadResp = serde_json::from_slice(&http_resp.bytes().await?)?;

        resp.basic_resp.is_ok()?;

        let decoded = decode(&resp.data, &key)?;
        let info: DownloadData = serde_json::from_slice(&decoded)?;

        for (_, v) in info.0.iter() {
            if v.file_size() > 0 {
                let mut copy = v.clone();
                copy.header = header;
                return Ok(Some(copy));
            }
        }

        Ok(None)
    }

    pub async fn download_file(&self, file_id: &str) -> Result<Option<DownloadInfo>, Pan115Error> {
        let file = self
            .get_file(file_id)
            .await?
            .ok_or(Pan115Error::FileNotFound(file_id.to_string()))?;
        if file.is_dir() {
            return Err(Pan115Error::UnsupportDownloadDirectory);
        }

        let download_info = self
            .download(file.pick_code.as_str(), crate::client::USER_AGENT)
            .await?;
        Ok(download_info)
    }

    pub async fn download_file_as_response(
        &self,
        file_id: &str,
    ) -> Result<Option<reqwest::Response>, Pan115Error> {
        let file = self
            .get_file(file_id)
            .await?
            .ok_or(Pan115Error::FileNotFound(file_id.to_string()))?;
        if file.is_dir() {
            return Err(Pan115Error::UnsupportDownloadDirectory);
        }

        let download_info = self
            .download(file.pick_code.as_str(), crate::client::USER_AGENT)
            .await?
            .ok_or(Pan115Error::DownloadFailed)?;
        let url = Url::parse(&download_info.url.url)
            .map_err(|_| Pan115Error::InvalidUrl(download_info.url.url.clone()))?;
        let resp = self
            .cli
            .get(url)
            .headers(download_info.header)
            .send()
            .await?;
        Ok(Some(resp))
    }
}

#[cfg(test)]
mod test {
    use std::env;

    use reqwest::Url;
    use tokio::{fs::File, io::AsyncWriteExt};
    use tokio_stream::StreamExt;

    use super::*;
    use crate::client::USER_AGENT;

    #[tokio::test]
    async fn test_download_with_ua() -> Result<()> {
        dotenv::dotenv().ok();
        let client = Client::new_from_env()?;
        let resp = client.download("b5nv5o8qwswiy8p6y", USER_AGENT).await?;
        let download_info = resp.unwrap();
        let resp = client
            .cli
            .get(download_info.url.url.parse::<Url>()?)
            .headers(download_info.header)
            .send()
            .await
            .unwrap();
        let mut file = File::create("test.mp4").await.unwrap();
        let mut stream = resp.bytes_stream();
        while let Some(item) = stream.next().await {
            match item {
                Ok(item) => {
                    file.write_all(&item).await.unwrap();
                }
                Err(e) => {
                    println!("error: {:?}", e);
                }
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_download_file() -> Result<()> {
        dotenv::dotenv().ok();
        let client = Client::new_from_env()?;
        let download_info = client.download_file("").await?;
        print!("{:?}", download_info);
        Ok(())
    }
}
