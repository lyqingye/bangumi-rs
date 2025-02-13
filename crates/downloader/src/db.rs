use anyhow::Result;
use chrono::{Local, NaiveDateTime};
use model::{
    sea_orm_active_enums::DownloadStatus,
    torrent_download_tasks::{self, Column, Model},
};
use sea_orm::{
    sea_query::{OnConflict, SimpleExpr},
    ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, IntoActiveModel,
    QueryFilter, TransactionTrait,
};
use std::{future::Future, sync::Arc};

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

    pub async fn begin(&self) -> Result<DatabaseTransaction> {
        Ok(self.conn().begin().await?)
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

    /// 在事务中执行操作
    pub async fn transaction<F, T, E>(&self, f: F) -> Result<T>
    where
        F: FnOnce(
            &DatabaseTransaction,
        ) -> std::pin::Pin<Box<dyn Future<Output = Result<T, E>> + Send + '_>>,
        E: Into<anyhow::Error>,
    {
        let txn = self.begin().await?;

        match f(&txn).await {
            Ok(result) => {
                txn.commit().await?;
                Ok(result)
            }
            Err(e) => {
                txn.rollback().await?;
                Err(e.into())
            }
        }
    }

    /// 更新任务状态
    ///
    /// 只更新状态、错误信息和更新时间三个字段
    ///
    /// # Arguments
    /// * `info_hash` - 任务的 info hash
    /// * `status` - 新的状态
    /// * `err_msg` - 错误信息
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
        retry_count: i32,
        next_retry_at: NaiveDateTime,
        err_msg: Option<String>,
    ) -> Result<()> {
        let now = Local::now().naive_utc();

        torrent_download_tasks::Entity::update_many()
            .col_expr(
                Column::DownloadStatus,
                SimpleExpr::from(DownloadStatus::Retrying),
            )
            .col_expr(Column::RetryCount, SimpleExpr::from(retry_count))
            .col_expr(Column::NextRetryAt, SimpleExpr::from(next_retry_at))
            .col_expr(Column::ErrMsg, SimpleExpr::from(err_msg))
            .col_expr(Column::UpdatedAt, SimpleExpr::from(now))
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
