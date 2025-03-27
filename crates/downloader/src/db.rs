use crate::errors::{Error, Result};
use async_trait::async_trait;
use bytes::Bytes;
use chrono::{Local, NaiveDateTime};
use model::{
    sea_orm_active_enums::{DownloadStatus, ResourceType},
    torrent_download_tasks::{self, Column, Model},
    torrents::{self, Column as TorrentColumn, Model as TorrentModel},
};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter, QueryTrait,
    prelude::Expr,
    sea_query::{OnConflict, SimpleExpr},
};
use std::{path::PathBuf, sync::Arc};

use crate::{Store, Tid, resource::Resource};

#[derive(Clone)]
pub struct Db {
    conn: Arc<DatabaseConnection>,
}

impl Db {
    pub fn new(conn: Arc<DatabaseConnection>) -> Self {
        Self { conn }
    }

    pub async fn new_from_env() -> Result<Self> {
        let conn =
            Arc::new(sea_orm::Database::connect(std::env::var("DATABASE_URL").unwrap()).await?);
        Ok(Self::new(conn))
    }

    pub fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }

    async fn get_task_resource(&self, task: &Model) -> Result<Resource> {
        Ok(match task.resource_type {
            ResourceType::Torrent => {
                let torrent = self
                    .get_torrent(&task.info_hash)
                    .await?
                    .ok_or_else(|| Error::TorrentNotFound(task.info_hash.to_string()))?;
                let data = torrent.data.ok_or_else(|| Error::EmptyTorrent)?;
                Resource::TorrentFileBytes(Bytes::from(data), task.info_hash.clone())
            }
            ResourceType::Magnet => {
                let magnet = task.magnet.as_ref().ok_or_else(|| Error::EmptyMagnet)?;
                Resource::from_magnet_link(magnet)?
            }
            ResourceType::InfoHash => Resource::from_info_hash(task.info_hash.clone())?,
            ResourceType::TorrentURL => {
                let torrent_url = task
                    .torrent_url
                    .as_ref()
                    .ok_or_else(|| Error::EmptyTorrentUrl(task.info_hash.to_string()))?;
                Resource::from_torrent_url(torrent_url, &task.info_hash)?
            }
        })
    }
}

#[async_trait]
impl Store for Db {
    async fn list_by_hashes(&self, info_hashes: &[String]) -> Result<Vec<Model>> {
        Ok(torrent_download_tasks::Entity::find()
            .filter(Column::InfoHash.is_in(info_hashes))
            .all(&*self.conn)
            .await?)
    }

    async fn list_by_status(&self, status: &[DownloadStatus]) -> Result<Vec<Model>> {
        Ok(torrent_download_tasks::Entity::find()
            .filter(Column::DownloadStatus.is_in(status.to_vec()))
            .all(&*self.conn)
            .await?)
    }

    async fn list_by_dlr_and_status(
        &self,
        downloader: &str,
        status: &[DownloadStatus],
    ) -> Result<Vec<Model>> {
        Ok(torrent_download_tasks::Entity::find()
            .filter(Column::Downloader.eq(downloader))
            .filter(Column::DownloadStatus.is_in(status.to_vec()))
            .all(&*self.conn)
            .await?)
    }

    async fn update_status(
        &self,
        info_hash: &str,
        status: DownloadStatus,
        err_msg: Option<String>,
        context: Option<String>,
    ) -> Result<()> {
        let now = Local::now().naive_utc();

        torrent_download_tasks::Entity::update_many()
            .col_expr(Column::DownloadStatus, SimpleExpr::from(status))
            .col_expr(Column::UpdatedAt, SimpleExpr::from(now))
            .col_expr(Column::ErrMsg, SimpleExpr::from(err_msg))
            .apply_if(context, |q, c| {
                q.col_expr(Column::Context, SimpleExpr::from(Some(c)))
            })
            .filter(Column::InfoHash.eq(info_hash))
            .exec(&*self.conn)
            .await?;
        Ok(())
    }

    async fn create(
        &self,
        resource: &Resource,
        dir: PathBuf,
        downloader: String,
        allow_fallback: bool,
    ) -> Result<()> {
        let now = Local::now().naive_utc();
        let task = Model {
            info_hash: resource.info_hash().to_string(),
            download_status: DownloadStatus::Pending,
            downloader,
            allow_fallback,
            context: None,
            err_msg: None,
            created_at: now,
            updated_at: now,
            dir: dir.to_string_lossy().into_owned(),
            retry_count: 0,
            next_retry_at: now,
            resource_type: resource.get_type(),
            magnet: resource.magnet(),
            torrent_url: resource.torrent_url(),
            tid: None,
        };
        torrent_download_tasks::Entity::insert(task.into_active_model())
            .on_conflict(
                OnConflict::column(Column::InfoHash)
                    .update_columns([
                        Column::UpdatedAt,
                        Column::DownloadStatus,
                        Column::TorrentUrl,
                        Column::ErrMsg,
                    ])
                    .to_owned(),
            )
            .exec(&*self.conn)
            .await?;
        Ok(())
    }

    async fn update_retry_status(
        &self,
        info_hash: &str,
        next_retry_at: NaiveDateTime,
        err_msg: Option<String>,
    ) -> Result<()> {
        let now = Local::now().naive_utc();

        torrent_download_tasks::Entity::update_many()
            .col_expr(
                Column::DownloadStatus,
                SimpleExpr::from(DownloadStatus::Retrying),
            )
            .col_expr(Column::RetryCount, Expr::col(Column::RetryCount).add(1))
            .col_expr(Column::NextRetryAt, SimpleExpr::from(next_retry_at))
            .col_expr(Column::ErrMsg, SimpleExpr::from(err_msg))
            .col_expr(Column::UpdatedAt, SimpleExpr::from(now))
            .col_expr(
                Column::DownloadStatus,
                SimpleExpr::from(DownloadStatus::Retrying),
            )
            .filter(Column::InfoHash.eq(info_hash))
            .exec(&*self.conn)
            .await?;
        Ok(())
    }

    async fn update_tid(&self, info_hash: &str, tid: &Tid) -> Result<()> {
        let now = Local::now().naive_utc();

        torrent_download_tasks::Entity::update_many()
            .col_expr(Column::Tid, SimpleExpr::from(tid.0.clone()))
            .col_expr(Column::UpdatedAt, SimpleExpr::from(now))
            .filter(Column::InfoHash.eq(info_hash))
            .exec(&*self.conn)
            .await?;
        Ok(())
    }

    async fn get_torrent(&self, info_hash: &str) -> Result<Option<TorrentModel>> {
        Ok(torrents::Entity::find()
            .filter(TorrentColumn::InfoHash.eq(info_hash))
            .one(&*self.conn)
            .await?)
    }

    async fn assign_dlr(&self, info_hash: &str, downloader: String) -> Result<()> {
        let now = Local::now().naive_utc();

        torrent_download_tasks::Entity::update_many()
            .col_expr(Column::Downloader, SimpleExpr::from(downloader))
            .col_expr(Column::UpdatedAt, SimpleExpr::from(now))
            .col_expr(Column::RetryCount, SimpleExpr::from(0))
            .filter(Column::InfoHash.eq(info_hash))
            .exec(&*self.conn)
            .await?;
        Ok(())
    }

    async fn get_by_hash(&self, info_hash: &str) -> Result<Option<Model>> {
        self.list_by_hashes(&[info_hash.to_string()])
            .await
            .map(|tasks| tasks.first().cloned())
    }

    async fn load_resource(&self, info_hash: &str) -> Result<Option<Resource>> {
        let task = self.get_by_hash(info_hash).await?;
        if let Some(task) = task {
            Ok(Some(self.get_task_resource(&task).await?))
        } else {
            Ok(None)
        }
    }
}
