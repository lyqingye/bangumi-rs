use crate::{
    format_backdrop_image_file_name, format_poster_image_file_name, MetadataAttr, MetadataAttrSet,
    MetadataDb,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use model::bangumi;
use tmdb::api::tvshow::{SeasonShort, TVShow};
use tokio::fs;
use tracing::{debug, info};

#[derive(Clone)]
pub struct MdbTmdb {
    pub tmdb: tmdb::client::Client,
    pub assets_path: String,
}

#[async_trait]
impl MetadataDb for MdbTmdb {
    async fn update_bangumi_metadata(
        &self,
        bgm: &mut bangumi::Model,
        attrs: MetadataAttrSet,
        force: bool,
    ) -> Result<()> {
        info!("使用TMDB填充番剧元数据: {}", bgm.name);
        let mut cache = None;
        if bgm.tmdb_id.is_none() {
            cache = self.match_tmdb(bgm).await?;

            if let Some((tv, _)) = &cache {
                bgm.tmdb_id = Some(tv.inner.id);
            }
        }

        let need_update = bgm.season_number.is_none()
            || bgm.poster_image_url.is_none()
            || bgm.backdrop_image_url.is_none()
            || bgm.description.is_none()
            || force;

        if !need_update {
            debug!("TMDB元数据已是最新，跳过更新");
            return Ok(());
        }

        debug!("开始更新TMDB元数据");

        if cache.is_none() {
            cache = self
                .tmdb
                .get_bangumi_and_season(bgm.tmdb_id.unwrap(), bgm.season_number.unwrap())
                .await?;
            if cache.is_none() {
                cache = self.match_tmdb(bgm).await.context("匹配TMDB失败")?;
            }
        }

        if cache.is_none() {
            return Err(anyhow::anyhow!("TMDB 更新元数据失败，未找到匹配的番剧"));
        }

        let (tv, season) = cache.unwrap();

        if attrs.is_required(MetadataAttr::SeasonNumber) && (bgm.season_number.is_none() || force) {
            bgm.season_number = Some(season.inner.season_number);
        }

        if attrs.is_required(MetadataAttr::Poster) && (bgm.poster_image_url.is_none() || force) {
            if let Some(poster_path) = tv.inner.poster_path {
                bgm.poster_image_url = Some(
                    self.download_image_from_tmdb(
                        &poster_path,
                        format_poster_image_file_name(bgm).as_str(),
                    )
                    .await
                    .context("下载海报失败")?,
                );
            }
        }

        if attrs.is_required(MetadataAttr::Backdrop) && (bgm.backdrop_image_url.is_none() || force)
        {
            if let Some(backdrop_path) = tv.inner.backdrop_path {
                bgm.backdrop_image_url = Some(
                    self.download_image_from_tmdb(
                        &backdrop_path,
                        format_backdrop_image_file_name(bgm).as_str(),
                    )
                    .await
                    .context("下载背景图失败")?,
                );
            }
        }

        if attrs.is_required(MetadataAttr::Description) && (bgm.description.is_none() || force) {
            if let Some(overview) = tv.inner.overview {
                bgm.description = Some(overview);
            }
        }

        info!("TMDB元数据更新完成");

        Ok(())
    }

    fn supports(&self) -> MetadataAttrSet {
        MetadataAttrSet(vec![
            MetadataAttr::SeasonNumber,
            MetadataAttr::Poster,
            MetadataAttr::Backdrop,
            MetadataAttr::Description,
        ])
    }
}

impl MdbTmdb {
    async fn match_tmdb(&self, bgm: &mut bangumi::Model) -> Result<Option<(TVShow, SeasonShort)>> {
        info!("尝试匹配 TMDB: {}", bgm.name);
        let air_date = bgm.air_date.map(|dt| dt.and_utc().date_naive());
        self.tmdb.match_bangumi(&bgm.name, air_date).await
    }

    async fn download_image_from_tmdb(
        &self,
        tmdb_file_path: &str,
        file_name: &str,
    ) -> Result<String> {
        info!("尝试从 TMDB 中下载图片: {} {}", tmdb_file_path, file_name);

        let ext = tmdb_file_path
            .split('?')
            .next()
            .and_then(|path| path.split('.').last())
            .unwrap_or("jpg");

        fs::create_dir_all(&self.assets_path).await?;

        let write_file_name = format!("{}.{}", file_name, ext);
        let write_path = format!("{}/{}", &self.assets_path, write_file_name);

        self.tmdb
            .download_image(tmdb_file_path, &write_path)
            .await?;

        Ok(write_file_name)
    }
}
