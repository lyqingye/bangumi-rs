use anyhow::Result;
use bangumi_tv::model::Subject;
use model::bangumi;
use tmdb::api::tvshow::{SeasonShort, TVShow};
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
        let air_date = bgm.air_date.map(|dt| dt.and_utc().date_naive());
        self.tmdb.match_bangumi(&bgm.name, air_date).await
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

        if loaded {
            return Ok(None);
        }

        let subject = self.bgm_tv.get_subject(bgm.bangumi_tv_id.unwrap()).await?;
        Ok(subject)
    }
}
