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
use reqwest::header::USER_AGENT;
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
}

#[cfg(test)]
mod test {
    use std::env;

    use crate::client::USER_AGENT;

    use super::*;

    #[tokio::test]
    async fn test_download_with_ua() -> Result<()> {
        let client = Client::new_from_env()?;
        let resp = client.download("ab858lbasffmqhsiv", USER_AGENT).await?;
        let download_info = resp.unwrap();
        print!("{:?}", download_info);
        Ok(())
    }
}
