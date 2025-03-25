use anyhow::Result;
use async_trait::async_trait;
use tokio::fs;
use tracing::{error, info, warn};

use model::bangumi;

use crate::{MetadataAttr, MetadataAttrSet, MetadataDb, format_poster_image_file_name};

#[derive(Clone)]
pub struct MdbBgmTV {
    pub bgm_tv: bangumi_tv::client::Client,
    pub assets_path: String,
}

#[async_trait]
impl MetadataDb for MdbBgmTV {
    async fn update_bangumi_metadata(
        &self,
        bgm: &mut bangumi::Model,
        attrs: MetadataAttrSet,
        force: bool,
    ) -> Result<()> {
        info!("[bgm.tv] 填充番剧元数据: {}", bgm.name);
        if bgm.bangumi_tv_id.is_none() {
            warn!("[bgm.tv] 没有 bangumi_tv_id，跳过更新");
            return Ok(());
        }

        let need_update = bgm.ep_count == 0
            || bgm.description.is_none()
            || bgm.rating.is_none()
            || bgm.air_date.is_none()
            || bgm.name.is_empty()
            || bgm.poster_image_url.is_none()
            || force;

        if !need_update {
            info!("[bgm.tv] 元数据已是最新，跳过更新");
            return Ok(());
        }

        let subject = self
            .bgm_tv
            .get_subject(bgm.bangumi_tv_id.unwrap())
            .await?
            .ok_or(anyhow::anyhow!("获取bgm.tv subject失败"))?;

        if attrs.is_required(MetadataAttr::EpCount) && (bgm.ep_count == 0 || force) {
            bgm.ep_count = subject.get_eps();
        }

        if attrs.is_required(MetadataAttr::Description) && (bgm.description.is_none() || force) {
            bgm.description = Some(subject.summary.clone());
        }

        if attrs.is_required(MetadataAttr::Rating) && (bgm.rating.is_none() || force) {
            bgm.rating = Some(subject.rating.score);
        }

        if attrs.is_required(MetadataAttr::AirDate) && (bgm.air_date.is_none() || force) {
            bgm.air_date = subject.get_air_date();
        }

        if attrs.is_required(MetadataAttr::Name) && (bgm.name.is_empty() || force) {
            bgm.name = subject.name_cn.clone().unwrap_or(subject.name.clone());
        }

        if attrs.is_required(MetadataAttr::Poster) && (bgm.poster_image_url.is_none() || force) {
            match self
                .download_image_from_bangumi_tv(
                    &subject.images.large,
                    format_poster_image_file_name(bgm).as_str(),
                )
                .await
            {
                Ok(filename) => bgm.poster_image_url = Some(filename),
                Err(err) => error!("下载图片失败 {} 错误: {}", subject.images.common, err),
            }
        }
        Ok(())
    }

    fn supports(&self) -> MetadataAttrSet {
        MetadataAttrSet(vec![
            MetadataAttr::Name,
            MetadataAttr::Description,
            MetadataAttr::Rating,
            MetadataAttr::EpCount,
            MetadataAttr::AirDate,
            MetadataAttr::Poster,
        ])
    }
}

impl MdbBgmTV {
    async fn download_image_from_bangumi_tv(
        &self,
        bgm_tv_file_path: &str,
        file_name: &str,
    ) -> Result<String> {
        let ext = std::path::Path::new(bgm_tv_file_path)
            .extension()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("jpg");

        info!(
            "尝试从 bangumi.tv 中下载图片: {} {}",
            bgm_tv_file_path, file_name
        );

        fs::create_dir_all(&self.assets_path).await?;

        let write_file_name = format!("{}.{}", file_name, ext);
        let write_path = format!("{}/{}", &self.assets_path, write_file_name);

        self.bgm_tv
            .download_image(bgm_tv_file_path, &write_path)
            .await?;

        Ok(write_file_name)
    }
}
