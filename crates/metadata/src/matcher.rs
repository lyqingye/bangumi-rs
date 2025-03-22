use anyhow::{Context, Result};
use tracing::{info, warn};

use model::{bangumi, sea_orm_active_enums::BgmKind};
use tmdb::api::{
    movie::MovieShort,
    tvshow::{SeasonShort, TVShow},
};

#[derive(Clone)]
pub struct Matcher {
    pub tmdb: tmdb::client::Client,
    pub bgm_tv: bangumi_tv::client::Client,
    pub mikan: mikan::client::Client,
}

impl Matcher {
    pub fn new(
        tmdb: tmdb::client::Client,
        bgm_tv: bangumi_tv::client::Client,
        mikan: mikan::client::Client,
    ) -> Self {
        Self {
            tmdb,
            bgm_tv,
            mikan,
        }
    }
    pub async fn match_tmdb(
        &self,
        bgm: &mut bangumi::Model,
    ) -> Result<Option<(TVShow, SeasonShort)>> {
        let result = match (bgm.tmdb_id, bgm.season_number) {
            (Some(tmdb_id), Some(season_number)) => {
                self.tmdb
                    .get_bangumi_and_season(tmdb_id, season_number)
                    .await?
            }
            _ => {
                let air_date = bgm.air_date.map(|dt| dt.and_utc().date_naive());
                info!("尝试匹配 TMDB: {} {:?}", bgm.name, air_date);
                self.tmdb.match_bangumi(&bgm.name, air_date).await?
            }
        };

        if let Some((tv, season)) = result {
            info!(
                "匹配到TMDB番剧: {} {:?}",
                tv.inner.name, season.inner.season_number
            );
            bgm.tmdb_id = Some(tv.inner.id);
            bgm.season_number = Some(season.inner.season_number);
            bgm.bgm_kind = Some(BgmKind::Anime);
            Ok(Some((tv, season)))
        } else {
            warn!("找不到对应的TMDB番剧: {}", bgm.name);
            Ok(None)
        }
    }

    pub async fn match_tmdb_movie(&self, bgm: &mut bangumi::Model) -> Result<Option<MovieShort>> {
        let air_date = bgm.air_date.map(|dt| dt.and_utc().date_naive());
        info!("尝试匹配 TMDB 电影: {} {:?}", bgm.name, air_date);
        let movies = self.tmdb.seach_movie(&bgm.name, air_date).await?;
        let mut movies = movies.into_iter().filter(|m| m.genre_ids.contains(&16));
        let movie = movies.next();
        if movie.is_none() {
            return Ok(None);
        }
        let movie = movie.unwrap();
        info!("匹配到TMDB电影: {} {:?}", movie.inner.title, air_date);
        bgm.tmdb_id = Some(movie.inner.id);
        bgm.bgm_kind = Some(BgmKind::Movie);
        Ok(Some(movie.clone()))
    }

    pub async fn match_bgm_tv(&self, bgm: &mut bangumi::Model) -> Result<()> {
        info!("尝试匹配 bgm.tv: {}", bgm.name);
        if bgm.mikan_id.is_none() {
            warn!("[MIKAN] 没有 mikan_id ，跳过匹配");
            return Ok(());
        }

        let info = self.mikan.get_bangumi_info(bgm.mikan_id.unwrap()).await?;
        bgm.bangumi_tv_id = info.bangumi_tv_id;

        if bgm.bangumi_tv_id.is_none() {
            // 尝试搜索, 这里可能用的是mikan的air_date（并不准确）
            let air_date = bgm.air_date.map(|dt| dt.and_utc().date_naive());
            let subject = self
                .bgm_tv
                .match_bangumi(&bgm.name, air_date)
                .await
                .with_context(|| {
                    format!(
                        "[bgm.tv] 在BangumiTV 搜索番剧失败, name: {}, air_date: {}",
                        bgm.name,
                        air_date.unwrap_or_default()
                    )
                })?;
            if let Some(subject) = subject {
                bgm.bangumi_tv_id = Some(subject.id);
                bgm.air_date = subject.get_air_date();
                info!(
                    "[bgm.tv] 在BangumiTV 搜索到相关番剧，name: {}, mikan_id: {}, bangumi_tv_id: {} air_date: {:?}",
                    subject.name_cn.clone().unwrap_or(subject.name.clone()),
                    bgm.mikan_id.unwrap(),
                    subject.id,
                    bgm.air_date
                );
                return Ok(());
            } else {
                warn!(
                    "[bgm.tv] 无法根据 MikanId 关联到 bangumi_tv_id，跳过匹配, name: {}, mikan_id: {}, 在BangumiTV 无法搜索到相关番剧",
                    bgm.name,
                    bgm.mikan_id.unwrap(),
                );
                return Ok(());
            }
        }

        if bgm.air_date.is_none() {
            let subject = self.bgm_tv.get_subject(bgm.bangumi_tv_id.unwrap()).await?;
            if let Some(subject) = subject.as_ref() {
                bgm.air_date = subject.get_air_date();
            }
        }
        Ok(())
    }
}
