use anyhow::Result;
use bangumi_tv::{
    self,
    model::{EpisodeList, EpisodeType},
};
use chrono::NaiveDateTime;
use mikan::client::{BangumiInfo, EpisodeItem};
use model::{bangumi, episodes, sea_orm_active_enums::SubscribeStatus};
use tmdb::api::{
    movie::MovieShort,
    tvshow::{SeasonShort, TVShow},
};
use tokio::fs;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct Fetcher {
    pub tmdb: tmdb::client::Client,
    pub bgm_tv: bangumi_tv::client::Client,
    pub mikan: mikan::client::Client,
    pub assets_path: String,
    pub client: reqwest::Client,
}

impl Fetcher {
    pub fn new(
        tmdb: tmdb::client::Client,
        bgm_tv: bangumi_tv::client::Client,
        mikan: mikan::client::Client,
        assets_path: String,
        client: reqwest::Client,
    ) -> Self {
        Self {
            tmdb,
            bgm_tv,
            mikan,
            assets_path,
            client,
        }
    }

    pub fn new_from_env() -> Result<Self> {
        let tmdb = tmdb::client::Client::new_from_env()?;
        let bgm_tv = bangumi_tv::client::Client::new_from_env()?;
        let mikan = mikan::client::Client::from_env()?;
        let client = reqwest::Client::new();
        let assets_path = std::env::var("ASSETS_PATH")?;
        Ok(Self::new(tmdb, bgm_tv, mikan, assets_path, client))
    }
}

impl Fetcher {
    pub async fn collect_episodes(&self, bgm: &bangumi::Model) -> Result<EpisodeList> {
        match bgm.bangumi_tv_id {
            Some(subject_id) => {
                let episodes = self
                    .bgm_tv
                    .episodes(subject_id, EpisodeType::Normal, 1000, 0)
                    .await?;
                Ok(episodes)
            }
            None => {
                warn!("番剧 {} 缺少 bangumi_tv_id", bgm.name);
                Ok(EpisodeList::default())
            }
        }
    }

    pub async fn collect_torrents(&self, bgm: &bangumi::Model) -> Result<Vec<EpisodeItem>> {
        match bgm.mikan_id {
            Some(id) => {
                let torrents = self.mikan.collect_by_bangumi_id(id).await?;
                Ok(torrents)
            }
            None => {
                warn!("番剧 {} 缺少 mikan_id", bgm.name);
                Ok(vec![])
            }
        }
    }
}

impl Fetcher {
    pub async fn seach_bangumi_at_tmdb(&self, name: &str) -> Result<Vec<TVShow>> {
        self.tmdb.search_bangumi(name).await
    }

    pub async fn seach_movie_at_tmdb(&self, name: &str) -> Result<Vec<MovieShort>> {
        self.tmdb.seach_movie(name).await
    }

    pub async fn download_image_from_tmdb_as_response(
        &self,
        file_path: &str,
    ) -> Result<reqwest::Response> {
        self.tmdb.download_image_as_response(file_path).await
    }
}
