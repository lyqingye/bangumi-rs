use anyhow::Result;
use bangumi_tv::{
    self,
    model::{EpisodeList, EpisodeType},
};
use chrono::NaiveDateTime;
use mikan::client::EpisodeItem;
use model::{bangumi, episodes, sea_orm_active_enums::SubscribeStatus};
use tmdb::api::tvshow::{SeasonShort, TVShow};
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

    pub async fn fill_bangumi_metadata(&self, bgm: &mut bangumi::Model) -> Result<()> {
        info!(
            "正在更新番剧元数据: {} mikan_id: {:?}",
            bgm.name.clone(),
            bgm.mikan_id
        );
        let now = chrono::Local::now().naive_utc();

        // 1. 获取或更新 bangumi_tv_id
        let (bgm_tv_id, mikan_info) = self.get_or_fetch_bangumi_tv_id(bgm).await?;
        bgm.bangumi_tv_id = Some(bgm_tv_id);

        // 2. 匹配TMDB
        if bgm.tmdb_id.is_none() {
            info!("TMDB 尝试匹配: {} {:?}", bgm.name, bgm.air_date);
            match self.match_tmdb(bgm).await? {
                Some((tv, season)) => {
                    bgm.tmdb_id = Some(tv.inner.id);
                    bgm.season_number = Some(season.inner.season_number);
                    info!("TMDB 匹配成功: {} id: {}", bgm.name, bgm.tmdb_id.unwrap());

                    if let Some(poster_path) = tv.inner.poster_path {
                        let file_name = format!("bangumi_poster_{}", bgm.id);
                        bgm.poster_image_url = Some(
                            self.download_image_from_tmdb(&poster_path, &file_name)
                                .await?,
                        );
                    }
                    if let Some(backdrop_path) = tv.inner.backdrop_path {
                        let file_name = format!("bangumi_backdrop_{}", bgm.id);
                        bgm.backdrop_image_url = Some(
                            self.download_image_from_tmdb(&backdrop_path, &file_name)
                                .await?,
                        );
                    }
                }
                None => {
                    warn!("未找到 TMDB 匹配结果: {}", bgm.name);
                }
            }
        }

        // 3. 只在缺少必要信息时更新元数据
        if self.needs_metadata_update(bgm) {
            info!("从 bangumi.tv 获取元数据: {} id: {}", bgm.name, bgm_tv_id);
            self.update_bangumi_metadata(bgm, bgm_tv_id, mikan_info, now)
                .await?;
        }

        Ok(())
    }

    fn needs_metadata_update(&self, bgm: &bangumi::Model) -> bool {
        bgm.ep_count == 0
            || bgm.air_date.is_none()
            || bgm.rating.is_none()
            || bgm.name.is_empty()
            || bgm.poster_image_url.is_none()
    }

    async fn update_bangumi_metadata(
        &self,
        bgm: &mut bangumi::Model,
        bgm_tv_id: i32,
        mikan_info: Option<mikan::client::BangumiInfo>,
        now: NaiveDateTime,
    ) -> Result<()> {
        info!("番剧: {} 缺少元数据，尝试补齐", bgm.name);

        // 尝试从 bangumi.tv 获取信息
        if let Some(subject) = self.bgm_tv.get_subject(bgm_tv_id).await? {
            self.fetch_metadata_from_bangumi_tv(bgm, &subject, now)
                .await?;
        } else {
            warn!("未在 bangumi.tv 找到番剧信息: {}", bgm.name);
            // 如果没有封面，尝试从 mikan 获取
            if bgm.poster_image_url.is_none() {
                self.fetch_image_from_mikan(bgm, mikan_info).await?;
            }
        }
        Ok(())
    }

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
    async fn fetch_metadata_from_bangumi_tv(
        &self,
        bgm: &mut bangumi::Model,
        subject: &bangumi_tv::model::Subject,
        now: NaiveDateTime,
    ) -> Result<()> {
        info!(
            "尝试从 bangumi tv 中拉取元数据: {} bangumi_tv_id: {:?}",
            bgm.name, subject.id
        );

        bgm.ep_count = subject.get_eps();
        bgm.description = Some(subject.summary.clone());
        bgm.rating = Some(subject.rating.score);
        if bgm.air_date.is_none() {
            bgm.air_date = subject.get_air_date();
        }
        bgm.updated_at = now;
        if bgm.name.is_empty() {
            bgm.name = subject.name_cn.clone().unwrap_or(subject.name.clone());
        }

        if bgm.poster_image_url.is_none() {
            match self
                .download_image(
                    &subject.images.common,
                    format!("bangumi_{}", bgm.id).as_str(),
                )
                .await
            {
                Ok(filename) => bgm.poster_image_url = Some(filename),
                Err(err) => error!("下载图片失败 {} 错误: {}", subject.images.common, err),
            }
        }
        Ok(())
    }

    async fn fetch_image_from_mikan(
        &self,
        bgm: &mut bangumi::Model,
        cached_info: Option<mikan::client::BangumiInfo>,
    ) -> Result<()> {
        info!(
            "尝试从 mikan 中下载图片: {} mikan_id: {:?}",
            bgm.name, bgm.mikan_id
        );
        let info = match cached_info {
            Some(info) => info,
            None => self.mikan.get_bangumi_info(bgm.mikan_id.unwrap()).await?,
        };

        if let Some(image_url) = info.image_url {
            match self
                .download_image(&image_url, format!("bangumi_{}", bgm.id).as_str())
                .await
            {
                Ok(filename) => bgm.poster_image_url = Some(filename),
                Err(err) => error!("下载图片失败 {} 错误: {}", image_url, err),
            }
        }
        Ok(())
    }

    async fn match_tmdb(&self, bgm: &mut bangumi::Model) -> Result<Option<(TVShow, SeasonShort)>> {
        info!("尝试匹配 TMDB: {}", bgm.name);
        let air_date = bgm.air_date.map(|dt| dt.and_utc().date_naive());
        self.tmdb.match_bangumi(&bgm.name, air_date).await
    }

    async fn get_or_fetch_bangumi_tv_id(
        &self,
        bgm: &bangumi::Model,
    ) -> Result<(i32, Option<mikan::client::BangumiInfo>)> {
        if let Some(id) = bgm.bangumi_tv_id {
            return Ok((id, None));
        }

        let mikan_id = match bgm.mikan_id {
            Some(id) => id,
            None => {
                warn!("番剧 {} 缺少 bangumi_tv_id 和 mikan_id", bgm.name);
                anyhow::bail!("未找到有效的ID");
            }
        };

        let info = self.mikan.get_bangumi_info(mikan_id).await?;
        match info.bangumi_tv_id {
            Some(id) => Ok((id, Some(info))),
            None => {
                warn!("未找到 mikan_id: {} 对应的 bangumi.tv ID", mikan_id);
                anyhow::bail!("未找到 bangumi.tv ID");
            }
        }
    }

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

        // 检查文件是否已存在
        if fs::try_exists(&filepath).await? {
            return Ok(filename);
        }

        // 下载图片
        let response = self.client.get(url).send().await?;
        let bytes = response.bytes().await?;

        // 保存文件
        fs::write(&filepath, bytes).await?;

        Ok(filename)
    }

    async fn download_image_from_tmdb(
        &self,
        tmdb_file_path: &str,
        file_name: &str,
    ) -> Result<String> {
        let ext = tmdb_file_path
            .split('?')
            .next()
            .and_then(|path| path.split('.').last())
            .unwrap_or("jpg");

        info!("尝试从 TMDB 中下载图片: {} {}", tmdb_file_path, file_name);

        fs::create_dir_all(&self.assets_path).await?;

        let write_file_name = format!("{}.{}", file_name, ext);
        let write_path = format!("{}/{}", &self.assets_path, write_file_name);
        if fs::try_exists(&write_path).await? {
            return Ok(write_file_name);
        }
        self.tmdb
            .download_image(tmdb_file_path, &write_path)
            .await?;

        Ok(write_file_name)
    }
}
