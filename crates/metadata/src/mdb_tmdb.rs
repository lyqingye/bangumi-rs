use anyhow::{Context, Result};
use async_trait::async_trait;
use tokio::fs;
use tracing::{debug, info, warn};

use model::{bangumi, sea_orm_active_enums::BgmKind};

use crate::{
    MetadataAttr, MetadataAttrSet, MetadataDb, format_backdrop_image_file_name,
    format_poster_image_file_name,
};

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
        info!(
            "[TMDB] 填充番剧元数据: {}, tmdb_id: {:?}",
            bgm.name, bgm.tmdb_id
        );
        if bgm.tmdb_id.is_none() {
            warn!("[TMDB] 没有 tmdb_id ，跳过更新");
            return Ok(());
        }

        let need_update = bgm.season_number.is_none()
            || bgm.poster_image_url.is_none()
            || bgm.backdrop_image_url.is_none()
            || bgm.description.is_none()
            || force;

        if !need_update {
            info!("[TMDB]元数据已是最新，跳过更新");
            return Ok(());
        }

        debug!("开始更新TMDB元数据");

        let tmdb_id = bgm.tmdb_id.unwrap();

        if bgm.bgm_kind == Some(BgmKind::Anime) && bgm.season_number.is_none() {
            warn!("[TMDB] 没有 season_number, 也不是电影，跳过更新");
            return Ok(());
        }

        let poster_path;
        let backdrop_path;
        let overview;
        match bgm.bgm_kind {
            Some(BgmKind::Anime) => {
                let (tv, season) = self
                    .tmdb
                    .get_bangumi_and_season(tmdb_id, bgm.season_number.unwrap())
                    .await?
                    .context("[TMDB] 更新元数据失败，未找到元数据信息")?;
                if attrs.is_required(MetadataAttr::SeasonNumber)
                    && (bgm.season_number.is_none() || force)
                {
                    bgm.season_number = Some(season.inner.season_number);
                }
                poster_path = tv.inner.poster_path;
                backdrop_path = tv.inner.backdrop_path;
                overview = tv.inner.overview;
            }
            Some(BgmKind::Movie) => {
                let movie = self.tmdb.get_movie(tmdb_id).await?;
                poster_path = movie.inner.poster_path;
                backdrop_path = movie.inner.backdrop_path;

                if movie.inner.overview.is_empty() {
                    overview = None;
                } else {
                    overview = Some(movie.inner.overview);
                }
            }
            _ => {
                warn!("[TMDB] 既不是番剧也不是电影，需要先使用Matcher进行匹配");
                return Ok(());
            }
        };

        if attrs.is_required(MetadataAttr::Poster)
            && (bgm.poster_image_url.is_none() || force)
            && poster_path.is_some()
        {
            bgm.poster_image_url = Some(
                self.download_image_from_tmdb(
                    &poster_path.unwrap(),
                    format_poster_image_file_name(bgm).as_str(),
                )
                .await
                .context("下载海报失败")?,
            );
        }

        if attrs.is_required(MetadataAttr::Backdrop)
            && (bgm.backdrop_image_url.is_none() || force)
            && backdrop_path.is_some()
        {
            bgm.backdrop_image_url = Some(
                self.download_image_from_tmdb(
                    &backdrop_path.unwrap(),
                    format_backdrop_image_file_name(bgm).as_str(),
                )
                .await
                .context("下载背景图失败")?,
            );
        }

        if attrs.is_required(MetadataAttr::Description)
            && (bgm.description.is_none() || force)
            && overview.is_some()
        {
            bgm.description = overview;
        }

        info!("[TMDB] 元数据更新完成");

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
