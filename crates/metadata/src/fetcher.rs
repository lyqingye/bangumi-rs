use anyhow::Result;
use tracing::warn;

use bangumi_tv::{
    self,
    model::{EpisodeList, EpisodeType},
};
use mikan::client::SearchResultItem;
use model::bangumi;
use tmdb::api::{movie::MovieShort, tvshow::TVShow};

#[derive(Clone)]
pub struct Fetcher {
    pub tmdb: tmdb::client::Client,
    pub bgm_tv: bangumi_tv::client::Client,
    pub mikan: mikan::client::Client,
    pub client: reqwest::Client,
}

impl Fetcher {
    pub fn new(
        tmdb: tmdb::client::Client,
        bgm_tv: bangumi_tv::client::Client,
        mikan: mikan::client::Client,
        client: reqwest::Client,
    ) -> Self {
        Self {
            tmdb,
            bgm_tv,
            mikan,
            client,
        }
    }

    pub fn new_from_env() -> Result<Self> {
        let tmdb = tmdb::client::Client::new_from_env()?;
        let bgm_tv = bangumi_tv::client::Client::new_from_env()?;
        let mikan = mikan::client::Client::from_env()?;
        let client = reqwest::Client::new();
        Ok(Self::new(tmdb, bgm_tv, mikan, client))
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
}

impl Fetcher {
    pub async fn seach_bangumi_at_tmdb(&self, name: &str) -> Result<Vec<TVShow>> {
        self.tmdb.search_bangumi(name).await
    }

    pub async fn seach_movie_at_tmdb(&self, name: &str) -> Result<Vec<MovieShort>> {
        self.tmdb.seach_movie(name, None).await
    }

    pub async fn download_image_from_tmdb_as_response(
        &self,
        file_path: &str,
    ) -> Result<reqwest::Response> {
        self.tmdb.download_image_as_response(file_path).await
    }

    pub async fn search_bangumi_at_mikan(&self, name: &str) -> Result<Vec<SearchResultItem>> {
        self.mikan.search(name).await
    }
}
