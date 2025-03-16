use anyhow::Result;
use async_trait::async_trait;
use chrono::{Local, NaiveDateTime};
use model::{
    sea_orm_active_enums::DownloadStatus,
    torrent_download_tasks::{self, Column, Model},
    torrents::{self, Column as TorrentColumn, Model as TorrentModel},
};
use sea_orm::{
    sea_query::{OnConflict, SimpleExpr},
    ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
};
use std::sync::Arc;

use crate::Store;

#[derive(Clone)]
pub struct Db {
    conn: Arc<DatabaseConnection>,
}

impl Db {
    pub fn new(conn: Arc<DatabaseConnection>) -> Self {
        Self { conn }
    }

    pub async fn new_from_env() -> Result<Self> {
        let conn = Arc::new(sea_orm::Database::connect(std::env::var("DATABASE_URL")?).await?);
        Ok(Self::new(conn))
    }

    pub fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }

    pub async fn batch_upsert_download_tasks(&self, tasks: Vec<Model>) -> Result<()> {
        torrent_download_tasks::Entity::insert_many(
            tasks.into_iter().map(|t| t.into_active_model()),
        )
        .on_conflict(
            OnConflict::column(Column::InfoHash)
                .update_columns([Column::UpdatedAt])
                .to_owned(),
        )
        .exec(&*self.conn)
        .await?;
        Ok(())
    }

    pub async fn update_task_status(
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
            .col_expr(Column::Context, SimpleExpr::from(context))
            .filter(Column::InfoHash.eq(info_hash))
            .exec(&*self.conn)
            .await?;
        Ok(())
    }

    pub async fn update_task_retry_status(
        &self,
        info_hash: &str,
        next_retry_at: NaiveDateTime,
        err_msg: Option<String>,
    ) -> Result<()> {
        use sea_orm::prelude::Expr;
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

    pub async fn list_download_tasks(&self, info_hashes: Vec<String>) -> Result<Vec<Model>> {
        Ok(torrent_download_tasks::Entity::find()
            .filter(Column::InfoHash.is_in(info_hashes))
            .all(&*self.conn)
            .await?)
    }

    pub async fn list_download_tasks_by_status(
        &self,
        status: Vec<DownloadStatus>,
    ) -> Result<Vec<Model>> {
        Ok(torrent_download_tasks::Entity::find()
            .filter(Column::DownloadStatus.is_in(status))
            .all(&*self.conn)
            .await?)
    }
}

#[async_trait]
impl Store for Db {
    async fn list_by_hashes(&self, info_hashes: &[String]) -> Result<Vec<Model>> {
        self.list_download_tasks(info_hashes.to_vec()).await
    }

    async fn list_by_status(&self, status: &[DownloadStatus]) -> Result<Vec<Model>> {
        self.list_download_tasks_by_status(status.to_vec()).await
    }

    async fn update_status(
        &self,
        info_hash: &str,
        status: DownloadStatus,
        err_msg: Option<String>,
        result: Option<String>,
    ) -> Result<()> {
        self.update_task_status(info_hash, status, err_msg, result)
            .await
    }

    async fn upsert(&self, task: Model) -> Result<()> {
        self.batch_upsert_download_tasks(vec![task]).await
    }

    async fn update_retry_status(
        &self,
        info_hash: &str,
        next_retry_at: NaiveDateTime,
        err_msg: Option<String>,
    ) -> Result<()> {
        self.update_task_retry_status(info_hash, next_retry_at, err_msg)
            .await
    }

    async fn get_torrent_by_info_hash(&self, info_hash: &str) -> Result<Option<TorrentModel>> {
        Ok(torrents::Entity::find()
            .filter(TorrentColumn::InfoHash.eq(info_hash))
            .one(&*self.conn)
            .await?)
    }
}
