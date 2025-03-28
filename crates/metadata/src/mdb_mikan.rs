use anyhow::Result;
use async_trait::async_trait;
use tokio::fs;
use tracing::{error, info, warn};

use model::bangumi;

use crate::{MetadataAttr, MetadataAttrSet, MetadataDb, format_poster_image_file_name};

pub struct MdbMikan {
    pub mikan: mikan::client::Client,
    pub client: reqwest::Client,
    pub assets_path: String,
}

#[async_trait]
impl MetadataDb for MdbMikan {
    async fn update_bangumi_metadata(
        &self,
        bgm: &mut bangumi::Model,
        attrs: MetadataAttrSet,
        force: bool,
    ) -> Result<()> {
        info!("[MIKAN] 填充番剧元数据: {}", bgm.name);

        if bgm.mikan_id.is_none() {
            warn!("[MIKAN] 没有mikan_id，跳过更新");
            return Ok(());
        }

        let need_update = bgm.bangumi_tv_id.is_none() || bgm.poster_image_url.is_none() || force;

        if !need_update {
            info!("[MIKAN] 元数据已是最新，跳过更新");
            return Ok(());
        }

        let mut cache = None;
        if attrs.is_required(MetadataAttr::BgmTvId) && (bgm.bangumi_tv_id.is_none() || force) {
            let info = self.mikan.get_bangumi_info(bgm.mikan_id.unwrap()).await?;
            bgm.bangumi_tv_id = info.bangumi_tv_id;
            cache = Some(info);
        }

        if !(attrs.is_required(MetadataAttr::Poster) && (bgm.poster_image_url.is_none() || force)) {
            return Ok(());
        }

        let info = match cache {
            Some(info) => info,
            None => self.mikan.get_bangumi_info(bgm.mikan_id.unwrap()).await?,
        };

        let Some(image_url) = info.image_url else {
            return Ok(());
        };

        let filename = self
            .download_image(&image_url, format_poster_image_file_name(bgm).as_str())
            .await
            .map_err(|err| {
                error!("下载图片失败 {} 错误: {}", image_url, err);
                err
            })?;

        bgm.poster_image_url = Some(filename);
        Ok(())
    }

    fn supports(&self) -> MetadataAttrSet {
        MetadataAttrSet(vec![MetadataAttr::BgmTvId, MetadataAttr::Poster])
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
