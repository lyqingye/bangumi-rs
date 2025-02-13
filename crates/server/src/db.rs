use anyhow::Result;
use chrono::NaiveDate;
use model::bangumi;
use sea_orm::{
    ColumnTrait, ConnectOptions, Database, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
};
use std::{sync::Arc, time::Duration};
#[derive(Clone)]
pub struct Db {
    conn: Arc<DatabaseConnection>,
}

impl Db {
    /// 创建新的数据库连接
    pub async fn new(database_url: &str) -> Result<Self> {
        let mut opt = ConnectOptions::new(database_url);
        opt.max_connections(5)
            .min_connections(1)
            .connect_timeout(Duration::from_secs(5))
            .acquire_timeout(Duration::from_secs(5))
            // 设置 SQL 语句日志级别
            .sqlx_logging(true)
            .sqlx_logging_level(tracing::log::LevelFilter::Debug);

        let conn = Database::connect(opt).await?;
        Ok(Self {
            conn: Arc::new(conn),
        })
    }

    pub async fn from_env() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL")?;
        Self::new(&database_url).await
    }

    /// 获取数据库连接
    pub fn conn_pool(&self) -> Arc<DatabaseConnection> {
        self.conn.clone()
    }

    pub fn conn(&self) -> &DatabaseConnection {
        self.conn.as_ref()
    }
}

impl Db {
    pub async fn list_calendar_by_date(
        &self,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<bangumi::Model>> {
        let db = self.conn();

        let calendars = bangumi::Entity::find()
            .filter(bangumi::Column::AirDate.between(
                from.format("%Y-%m-%d").to_string(),
                to.format("%Y-%m-%d").to_string(),
            ))
            .order_by_asc(bangumi::Column::AirDate)
            .all(db)
            .await?;

        Ok(calendars)
    }
}
