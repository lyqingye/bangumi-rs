use anyhow::Result;
use model::{episode_download_tasks, torrent_download_tasks, torrents};
use sea_orm::{
    ColumnTrait, ConnectOptions, Database, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect,
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
    pub async fn delete_bangumi_download_tasks(&self, bangumi_id: i32) -> Result<()> {
        let db = self.conn();
        episode_download_tasks::Entity::delete_many()
            .filter(episode_download_tasks::Column::BangumiId.eq(bangumi_id))
            .exec(db)
            .await?;

        // 获取番剧对应的所有种子
        let info_hashes = torrents::Entity::find()
            .select_only()
            .column(torrents::Column::InfoHash)
            .filter(torrents::Column::BangumiId.eq(bangumi_id))
            .into_tuple::<String>()
            .all(db)
            .await?;
        if info_hashes.is_empty() {
            return Ok(());
        }

        // 删除下载记录
        torrent_download_tasks::Entity::delete_many()
            .filter(torrent_download_tasks::Column::InfoHash.is_in(info_hashes))
            .exec(db)
            .await?;
        Ok(())
    }
}
