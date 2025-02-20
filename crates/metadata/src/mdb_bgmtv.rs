use crate::{format_poster_image_file_name, MetadataDb};
use anyhow::Result;
use async_trait::async_trait;
use model::bangumi;
use tokio::fs;
use tracing::{error, info};

#[derive(Clone)]
pub struct MdbBgmTV {
    pub bgm_tv: bangumi_tv::client::Client,
    pub assets_path: String,
}

#[async_trait]
impl MetadataDb for MdbBgmTV {
    async fn update_bangumi_metadata(&self, bgm: &mut bangumi::Model, force: bool) -> Result<()> {
        info!("使用bgm.tv填充番剧元数据: {}", bgm.name);
        if bgm.bangumi_tv_id.is_none() {
            return Err(anyhow::anyhow!("番剧TV ID为空"));
        }

        let need_update = bgm.ep_count == 0
            || bgm.description.is_none()
            || bgm.rating.is_none()
            || bgm.air_date.is_none()
            || bgm.name.is_empty()
            || bgm.poster_image_url.is_none()
            || force;

        if !need_update {
            info!("bgm.tv元数据已是最新，跳过更新");
            return Ok(());
        }

        let subject = self
            .bgm_tv
            .get_subject(bgm.bangumi_tv_id.unwrap())
            .await?
            .ok_or(anyhow::anyhow!("获取bgm.tv subject失败"))?;

        if bgm.ep_count == 0 || force {
            bgm.ep_count = subject.get_eps();
        }

        if bgm.description.is_none() || force {
            bgm.description = Some(subject.summary.clone());
        }

        if bgm.rating.is_none() || force {
            bgm.rating = Some(subject.rating.score);
        }

        if bgm.air_date.is_none() || force {
            bgm.air_date = subject.get_air_date();
        }

        if bgm.name.is_empty() || force {
            bgm.name = subject.name_cn.clone().unwrap_or(subject.name.clone());
        }

        if bgm.poster_image_url.is_none() || force {
            match self
                .download_image_from_bangumi_tv(
                    &subject.images.common,
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
