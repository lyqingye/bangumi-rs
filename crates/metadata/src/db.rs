use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use chrono::NaiveDateTime;
use sea_orm::{
    ColumnTrait, ConnectOptions, Database, DatabaseConnection, EntityTrait, IntoActiveModel,
    QueryFilter, Set, sea_query::OnConflict,
};

use bangumi_tv::model::EpisodeList;
use mikan::client::Calendar;
use model::{bangumi, episodes, torrents};

#[derive(Clone)]
pub struct Db(Arc<DatabaseConnection>);

impl Db {
    pub fn new(conn: Arc<DatabaseConnection>) -> Self {
        Self(conn)
    }

    pub async fn new_from_env() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL")?;
        let mut opt = ConnectOptions::new(database_url);
        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(5))
            .acquire_timeout(Duration::from_secs(5))
            // 设置 SQL 语句日志级别
            .sqlx_logging(true)
            .sqlx_logging_level(tracing::log::LevelFilter::Debug);

        let conn = Database::connect(opt).await?;
        Ok(Self::new(Arc::new(conn)))
    }

    pub fn conn(&self) -> &DatabaseConnection {
        &self.0
    }
}

/// Bangumi 相关
impl Db {
    pub async fn get_bangumi_by_id(&self, id: i32) -> Result<Option<bangumi::Model>> {
        let db = self.conn();
        let one = bangumi::Entity::find_by_id(id).one(db).await?;
        Ok(one)
    }

    pub async fn get_bangumi_by_mikan_id(&self, id: i32) -> Result<Option<bangumi::Model>> {
        let db = self.conn();
        let one = bangumi::Entity::find()
            .filter(bangumi::Column::MikanId.eq(id))
            .one(db)
            .await?;
        Ok(one)
    }

    pub async fn update_bangumi(&self, bgm: bangumi::Model) -> Result<()> {
        let db = self.conn();

        bangumi::Entity::update_many()
            .filter(bangumi::Column::Id.eq(bgm.id))
            .set(bangumi::ActiveModel {
                name: Set(bgm.name),
                description: Set(bgm.description),
                bangumi_tv_id: Set(bgm.bangumi_tv_id),
                tmdb_id: Set(bgm.tmdb_id),
                mikan_id: Set(bgm.mikan_id),
                air_date: Set(bgm.air_date),
                air_week: Set(bgm.air_week),
                ep_count: Set(bgm.ep_count),
                rating: Set(bgm.rating),
                backdrop_image_url: Set(bgm.backdrop_image_url),
                poster_image_url: Set(bgm.poster_image_url),
                season_number: Set(bgm.season_number),
                ep_start_number: Set(bgm.ep_start_number),
                bgm_kind: Set(bgm.bgm_kind),
                ..Default::default()
            })
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn update_bangumi_update_time(&self, bgm_id: i32, time: NaiveDateTime) -> Result<()> {
        let db = self.conn();
        bangumi::Entity::update_many()
            .filter(bangumi::Column::Id.eq(bgm_id))
            .set(bangumi::ActiveModel {
                updated_at: Set(time),
                ..Default::default()
            })
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn batch_insert_bangumi(&self, bangumis: Vec<bangumi::Model>) -> Result<Vec<i32>> {
        let count = bangumis.len() as i32;
        let db = self.conn();
        let insert_result = bangumi::Entity::insert_many(
            bangumis.into_iter().map(|model| model.into_active_model()),
        )
        .exec(db)
        .await?;
        Ok((insert_result.last_insert_id..(insert_result.last_insert_id + count)).collect())
    }

    pub async fn list_bangumi_by_mikan_ids(
        &self,
        mikan_ids: Vec<i32>,
    ) -> Result<Vec<bangumi::Model>> {
        let db = self.conn();

        let bangumis = bangumi::Entity::find()
            .filter(bangumi::Column::MikanId.is_in(mikan_ids))
            .all(db)
            .await?;

        Ok(bangumis)
    }

    pub async fn add_bangumi(
        &self,
        title: String,
        mikan_id: i32,
        bangumi_tv_id: Option<i32>,
        tmdb_id: Option<u64>,
    ) -> Result<()> {
        let now = chrono::Local::now().naive_utc();
        let _ = self
            .batch_insert_bangumi(vec![bangumi::Model {
                id: 0,
                name: title,
                description: None,
                bangumi_tv_id,
                tmdb_id,
                mikan_id: Some(mikan_id),
                air_date: None,
                air_week: None,
                rating: None,
                created_at: now,
                updated_at: now,
                poster_image_url: None,
                backdrop_image_url: None,
                season_number: None,
                ep_count: 0,
                ep_start_number: 0,
                calendar_season: None,
                bgm_kind: None,
            }])
            .await?;
        Ok(())
    }

    pub async fn save_mikan_calendar(&self, calendar: Calendar) -> Result<Vec<i32>> {
        let mikan_ids: Vec<i32> = calendar.bangumis.iter().map(|item| item.id).collect();
        let exist_bangumis = self.list_bangumi_by_mikan_ids(mikan_ids.clone()).await?;
        let exist_mikan_ids: std::collections::HashSet<i32> = exist_bangumis
            .iter()
            .filter_map(|bgm| bgm.mikan_id)
            .collect();

        let now = chrono::Local::now().naive_utc();

        // 处理新番剧
        let new_bangumis: Vec<bangumi::Model> = calendar
            .bangumis
            .iter()
            .filter(|bgm| !exist_mikan_ids.contains(&bgm.id))
            .map(|bgm| bangumi::Model {
                id: 0,
                name: bgm.title.clone().unwrap_or_default(),
                description: None,
                bangumi_tv_id: None,
                tmdb_id: None,
                mikan_id: Some(bgm.id),
                air_date: None,
                air_week: Some(bgm.weekday),
                ep_count: 0,
                rating: None,
                created_at: now,
                updated_at: now,
                backdrop_image_url: None,
                poster_image_url: None,
                season_number: None,
                ep_start_number: 1,
                calendar_season: calendar.season.clone(),
                bgm_kind: None,
            })
            .collect();

        if !new_bangumis.is_empty() {
            let ids = self.batch_insert_bangumi(new_bangumis).await?;
            return Ok(ids);
        }

        Ok(vec![])
    }
}

/// Episodes 相关
impl Db {
    pub async fn batch_upsert_episodes(&self, episodes: Vec<episodes::Model>) -> Result<()> {
        if episodes.is_empty() {
            return Ok(());
        }
        let db = self.conn();
        episodes::Entity::insert_many(episodes.into_iter().map(|model| model.into_active_model()))
            .on_conflict(
                OnConflict::columns([episodes::Column::BangumiId, episodes::Column::Number])
                    .update_columns([
                        episodes::Column::UpdatedAt,
                        episodes::Column::DurationSeconds,
                        episodes::Column::SortNumber,
                        episodes::Column::Name,
                        episodes::Column::AirDate,
                        episodes::Column::Description,
                        episodes::Column::ImageUrl,
                        episodes::Column::Kind,
                    ])
                    .to_owned(),
            )
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn save_bangumi_tv_episodes(
        &self,
        bgm: &bangumi::Model,
        episodes: EpisodeList,
    ) -> Result<()> {
        let now = chrono::Local::now().naive_utc();
        let models = episodes
            .data
            .into_iter()
            // FIXME: 这里需要支持小数类型的剧集Number
            .filter(|ep| ep.get_ep().is_some())
            .map(|ep| {
                let ep_number = ep.get_ep();
                episodes::Model {
                    id: 0,
                    bangumi_id: bgm.id,
                    number: ep_number.unwrap(),
                    name: ep.name_cn.or(ep.name),
                    air_date: ep.airdate,
                    description: ep.desc,
                    image_url: None,
                    kind: ep.ep_type.to_string().into(),
                    created_at: now,
                    updated_at: now,
                    duration_seconds: Some(ep.duration_seconds),
                    sort_number: ep_number,
                }
            })
            .collect();

        self.batch_upsert_episodes(models).await?;
        Ok(())
    }
}

/// Torrents 相关
impl Db {
    pub async fn batch_upsert_torrent(&self, torrents: Vec<torrents::Model>) -> Result<()> {
        if torrents.is_empty() {
            return Ok(());
        }
        let db = self.conn();
        let _ = torrents::Entity::insert_many(
            torrents.into_iter().map(|model| model.into_active_model()),
        )
        .on_conflict(
            OnConflict::column(torrents::Column::InfoHash)
                .update_column(torrents::Column::PubDate)
                .to_owned(),
        )
        .exec(db)
        .await?;
        Ok(())
    }
}
