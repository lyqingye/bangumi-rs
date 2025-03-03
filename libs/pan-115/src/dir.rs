use std::collections::HashMap;
use std::path::PathBuf;

use crate::client::Client;
use crate::errors::Pan115Error;
use crate::model::MkdirResp;
use anyhow::Result;
use tracing::debug;

pub const API_CREATE_DIR: &str = "https://webapi.115.com/files/add";
impl Client {
    pub async fn mkdir(&self, cid: &str, name: &str) -> Result<String, Pan115Error> {
        debug!("mkdir: {:?} {:?}", cid, name);

        self.acquire().await;

        let mut form = HashMap::new();
        form.insert("pid".to_owned(), cid.to_owned());
        form.insert("cname".to_owned(), name.to_owned());

        let resp: MkdirResp = self
            .cli
            .post(API_CREATE_DIR)
            .form(&form)
            .send()
            .await?
            .json()
            .await?;
        resp.basic_resp.is_ok()?;
        Ok(resp.file_id)
    }

    /// 根据路径创建目录，如果路径中的某些目录不存在会自动创建
    ///
    /// # 参数
    /// * `path` - Unix 风格的文件路径，例如 "/downloads/anime"
    ///
    /// # 返回值
    /// * `Ok(String)` - 创建的目录的 cid
    /// * `Err(Pan115Error)` - 创建失败的错误
    pub async fn mkdir_by_path(&self, path: PathBuf) -> Result<String, Pan115Error> {
        debug!("mkdir_by_path: {:?}", path);

        self.acquire().await;

        // 检查路径是否合法
        if path.is_relative() {
            return Err(Pan115Error::InvalidPath(path.to_string_lossy().to_string()));
        }

        // 如果是根路径，直接返回 "0"
        if path == PathBuf::from("/") {
            return Ok("0".to_string());
        }

        let mut current_cid = "0".to_string();
        let components: Vec<_> = path.components().skip(1).collect(); // skip(1) 跳过根路径

        // 遍历路径的每个组件
        for (i, component) in components.iter().enumerate() {
            let dir_name = component.as_os_str().to_string_lossy();
            let current_path = PathBuf::from("/").join(
                components[0..=i]
                    .iter()
                    .map(|c| c.as_os_str())
                    .collect::<PathBuf>(),
            );

            // 检查当前层级的目录是否存在
            if let Some(cid) = self.path_to_cid(current_path).await? {
                current_cid = cid;
            } else {
                // 目录不存在，创建它
                current_cid = self.mkdir(&current_cid, &dir_name).await?;
            }
        }

        Ok(current_cid)
    }

    /// 将文件路径转换为 115 网盘的 cid
    ///
    /// # 参数
    /// * `path` - Unix 风格的文件路径，例如 "/downloads/anime"
    ///
    /// # 返回值
    /// * `Ok(String)` - 路径对应的 cid
    /// * `Err(Pan115Error)` - 转换失败的错误
    pub async fn path_to_cid(&self, path: PathBuf) -> Result<Option<String>, Pan115Error> {
        debug!("path_to_cid: {:?}", path);

        self.acquire().await;

        // 根目录的 cid 是 "0"
        let mut current_cid = "0".to_string();

        // 检查路径是否合法
        if path.is_relative() {
            return Err(Pan115Error::InvalidPath(path.to_string_lossy().to_string()));
        }

        // 如果是根路径，直接返回 "0"
        if path == PathBuf::from("/") {
            return Ok(Some(current_cid));
        }

        // 遍历路径的每个组件
        for component in path.components().skip(1) {
            // skip(1) 跳过根路径
            let dir_name = component.as_os_str().to_string_lossy();
            let mut found = false;
            let mut offset = 0;
            let limit = 100;

            // 分页查询直到找到匹配的文件夹
            loop {
                let files = self
                    .list_files(&current_cid, Some(offset), Some(limit))
                    .await?;
                if files.is_empty() {
                    break;
                }

                // 在当前页中查找匹配的文件夹
                for file in files.iter() {
                    if file.name == dir_name {
                        current_cid = file.file_id();
                        found = true;
                        break;
                    }
                }

                if found {
                    break;
                }

                if files.len() < limit as usize {
                    break;
                }

                offset += limit;
            }

            // 如果没有找到对应的文件夹，返回空
            if !found {
                return Ok(None);
            }
        }

        Ok(Some(current_cid))
    }
}

#[cfg(test)]
mod tests {
    use std::{num::NonZero, sync::Arc, time};

    use governor::{Quota, RateLimiter};

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_mkdir() -> Result<()> {
        dotenv::dotenv().ok();
        let client = Client::new_from_env()?;
        let file_id = client.mkdir("0", "卧槽").await?;
        println!("file_id: {}", file_id);
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_path_to_cid() -> Result<()> {
        dotenv::dotenv().ok();
        let client = Client::new_from_env()?;

        // 测试根路径
        let cid = client.path_to_cid("/".into()).await?;
        assert_eq!(cid, Some("0".to_string()));

        // 测试相对路径（应该返回错误）
        let result = client.path_to_cid("/test_dir2/test_dir3".into()).await?;
        println!("result: {:?}", result);

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_mkdir_by_path() -> Result<()> {
        dotenv::dotenv().ok();
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_target(true) // 不显示目标模块
            .init();
        let client = Client::new_from_env()?;
        let cid = client.mkdir_by_path("/animes/11".into()).await?;
        println!("cid: {}", cid);
        Ok(())
    }

    #[tokio::test]
    async fn test_rate_limit() -> Result<()> {
        dotenv::dotenv().ok();
        let quota =
            Quota::per_second(NonZero::new(1).unwrap()).allow_burst(NonZero::new(1).unwrap());
        let limiter = Arc::new(RateLimiter::direct(quota));
        for _ in 0..10 {
            limiter.until_n_ready(NonZero::new(1).unwrap()).await;
            println!(
                "{}",
                time::SystemTime::now()
                    .duration_since(time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            );
        }
        Ok(())
    }
}
