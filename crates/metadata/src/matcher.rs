use anyhow::{Context, Result};
use bangumi_tv::model::Subject;
use model::{bangumi, sea_orm_active_enums::BgmKind};
use tmdb::api::{
    movie::MovieShort,
    tvshow::{SeasonShort, TVShow},
};
use tracing::{info, warn};

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
        info!("尝试匹配 TMDB: {}", bgm.name);
        let result = match (bgm.tmdb_id, bgm.season_number) {
            (Some(tmdb_id), Some(season_number)) => {
                self.tmdb
                    .get_bangumi_and_season(tmdb_id, season_number)
                    .await?
            }
            _ => {
                let air_date = bgm.air_date.map(|dt| dt.and_utc().date_naive());
                self.tmdb.match_bangumi(&bgm.name, air_date).await?
            }
        };

        if let Some((tv, season)) = result {
            bgm.tmdb_id = Some(tv.inner.id);
            bgm.season_number = Some(season.inner.season_number);
            bgm.bgm_kind = Some(BgmKind::Anime);
            Ok(Some((tv, season)))
        } else {
            Ok(None)
        }
    }

    pub async fn match_tmdb_movie(&self, bgm: &mut bangumi::Model) -> Result<Option<MovieShort>> {
        info!("尝试匹配 TMDB 电影: {}", bgm.name);
        let air_date = bgm.air_date.map(|dt| dt.and_utc().date_naive());
        let movies = self.tmdb.seach_movie(&bgm.name, air_date).await?;
        if movies.is_empty() {
            return Ok(None);
        }
        let movie = movies.first();
        if movie.is_none() {
            return Ok(None);
        }
        let movie = movie.unwrap();
        bgm.tmdb_id = Some(movie.inner.id);
        bgm.bgm_kind = Some(BgmKind::Movie);
        Ok(Some(movie.clone()))
    }

    pub async fn match_bgm_tv(
        &self,
        bgm: &mut bangumi::Model,
        loaded: bool,
    ) -> Result<Option<Subject>> {
        info!("尝试匹配 bgm.tv: {}", bgm.name);
        if bgm.mikan_id.is_none() {
            warn!("[MIKAN] 没有 mikan_id ，跳过匹配");
            return Ok(None);
        }

        let info = self.mikan.get_bangumi_info(bgm.mikan_id.unwrap()).await?;
        bgm.bangumi_tv_id = info.bangumi_tv_id;

        if !loaded {
            return Ok(None);
        }
        if bgm.bangumi_tv_id.is_none() {
            // 尝试搜索
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
                info!(
                    "[bgm.tv] 在BangumiTV 搜索到相关番剧，name: {}, mikan_id: {}, bangumi_tv_id: {}",
                    subject.name_cn.clone().unwrap_or(subject.name.clone()),
                    bgm.mikan_id.unwrap(),
                    subject.id
                );
                return Ok(Some(subject));
            } else {
                warn!(
                    "[bgm.tv] 无法根据 MikanId 关联到 bangumi_tv_id，跳过匹配, name: {}, mikan_id: {}, 在BangumiTV 无法搜索到相关番剧",
                    bgm.name,
                    bgm.mikan_id.unwrap(),
                );
                return Ok(None);
            }
        }

        let subject = self.bgm_tv.get_subject(bgm.bangumi_tv_id.unwrap()).await?;
        Ok(subject)
    }
}
