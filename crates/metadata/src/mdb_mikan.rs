use anyhow::Result;
use async_trait::async_trait;
use model::bangumi;
use tokio::fs;
use tracing::{error, info};

use crate::{format_poster_image_file_name, MetadataDb};

pub struct MdbMikan {
    pub mikan: mikan::client::Client,
    pub client: reqwest::Client,
    pub assets_path: String,
}

#[async_trait]
impl MetadataDb for MdbMikan {
    async fn update_bangumi_metadata(&self, bgm: &mut bangumi::Model, force: bool) -> Result<()> {
        info!("使用mikan填充番剧元数据: {}", bgm.name);

        if bgm.mikan_id.is_none() {
            return Err(anyhow::anyhow!("番剧缺少mikan_id"));
        }

        let need_update = bgm.bangumi_tv_id.is_none() || bgm.poster_image_url.is_none() || force;

        if !need_update {
            info!("mikan元数据已是最新，跳过更新");
            return Ok(());
        }

        let mut cache = None;
        if bgm.bangumi_tv_id.is_none() || force {
            let info = self.mikan.get_bangumi_info(bgm.mikan_id.unwrap()).await?;
            bgm.bangumi_tv_id = info.bangumi_tv_id;
            cache = Some(info);
        }

        if bgm.poster_image_url.is_none() || force {
            let info = match cache {
                Some(info) => info,
                None => self.mikan.get_bangumi_info(bgm.mikan_id.unwrap()).await?,
            };

            if let Some(image_url) = info.image_url {
                match self
                    .download_image(&image_url, format_poster_image_file_name(bgm).as_str())
                    .await
                {
                    Ok(filename) => bgm.poster_image_url = Some(filename),
                    Err(err) => error!("下载图片失败 {} 错误: {}", image_url, err),
                }
            }
        }
        Ok(())
    }
}

impl MdbMikan {
    async fn download_image(&self, url: &str, file_name: &str) -> Result<String> {
        // 创建图片存储目录
        fs::create_dir_all(&self.assets_path).await?;

        // 获取文件扩展名
        let ext = url
            .split('?')
            .next()
            .and_then(|path| path.split('.').last())
            .unwrap_or("jpg");
        let filename = format!("{}.{}", file_name, ext);
        let filepath = format!("{}/{}", &self.assets_path, filename);

        // 下载图片
        let response = self.client.get(url).send().await?;
        let bytes = response.bytes().await?;

        // 保存文件
        fs::write(&filepath, bytes).await?;

        Ok(filename)
    }
}
