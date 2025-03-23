use crate::model::DownloadTask;
use anyhow::Result;
use model::{
    bangumi, episode_download_tasks, sea_orm_active_enums::DownloadStatus, subscriptions,
    torrent_download_tasks, torrents,
};
use sea_orm::{
    ColumnTrait, ConnectOptions, Database, DatabaseConnection, EntityTrait, QueryFilter,
    QuerySelect,
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

    pub async fn get_bangumi_by_mikan_id(&self, mikan_id: i32) -> Result<Option<bangumi::Model>> {
        let db = self.conn();
        let bangumi = bangumi::Entity::find()
            .filter(bangumi::Column::MikanId.eq(mikan_id))
            .one(db)
            .await?;
        Ok(bangumi)
    }

    pub async fn get_subscription_by_bangumi_id(
        &self,
        bangumi_id: i32,
    ) -> Result<Option<subscriptions::Model>> {
        let db = self.conn();
        let subscription = subscriptions::Entity::find()
            .filter(subscriptions::Column::BangumiId.eq(bangumi_id))
            .one(db)
            .await?;
        Ok(subscription)
    }

    pub async fn query_downloads_info(
        &self,
        offset: u64,
        limit: u64,
        status: Option<DownloadStatus>,
    ) -> Result<Vec<DownloadTask>> {
        let db = self.conn();
        use model::bangumi::Column as BangumiColumn;
        use model::bangumi::Entity as Bangumis;
        use model::episode_download_tasks::Column as EpisodeTaskColumn;
        use model::episode_download_tasks::Entity as EpisodeTasks;
        use model::torrent_download_tasks::Column as TorrentTaskColumn;
        use model::torrent_download_tasks::Entity as TorrentTasks;
        use model::torrents::Column as TorrentColumn;
        use model::torrents::Entity as Torrents;
        use sea_orm::{ColumnTrait, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect};

        // 1. 从 torrent_download_tasks 开始查询，确保能使用 status_idx
        let mut query = TorrentTasks::find()
            .select_only()
            // 只选择需要的字段，减少数据传输
            .columns([
                TorrentTaskColumn::InfoHash,
                TorrentTaskColumn::DownloadStatus,
                TorrentTaskColumn::Downloader,
                TorrentTaskColumn::CreatedAt,
                TorrentTaskColumn::UpdatedAt,
                TorrentTaskColumn::ErrMsg,
                TorrentTaskColumn::RetryCount,
            ]);

        // 2. 添加状态筛选，利用索引的第一个字段
        if let Some(status) = status {
            query = query.filter(TorrentTaskColumn::DownloadStatus.eq(status));
        }

        // 3. 使用索引的第二个字段排序
        query = query.order_by_desc(TorrentTaskColumn::UpdatedAt);

        // 4. 优化连接顺序，使用正确的连接条件
        let tasks = query
            // 首先连接 episode_download_tasks，使用 ref_torrent_info_hash_idx
            .join(
                JoinType::InnerJoin,
                TorrentTasks::belongs_to(EpisodeTasks)
                    .from(TorrentTaskColumn::InfoHash)
                    .to(EpisodeTaskColumn::RefTorrentInfoHash)
                    .into(),
            )
            // 然后连接 torrents，使用主键
            .join(
                JoinType::InnerJoin,
                TorrentTasks::belongs_to(Torrents)
                    .from(TorrentTaskColumn::InfoHash)
                    .to(TorrentColumn::InfoHash)
                    .into(),
            )
            // 最后连接 bangumi，使用主键
            .join(
                JoinType::InnerJoin,
                EpisodeTasks::belongs_to(Bangumis)
                    .from(EpisodeTaskColumn::BangumiId)
                    .to(BangumiColumn::Id)
                    .into(),
            )
            // 添加需要的其他字段
            .column(BangumiColumn::Name)
            .column_as(BangumiColumn::Id, "bangumi_id")
            .column(EpisodeTaskColumn::EpisodeNumber)
            .column_as(TorrentColumn::Title, "file_name")
            .column_as(TorrentColumn::Size, "file_size")
            // 分页
            .offset(offset)
            .limit(limit)
            .into_model::<DownloadTask>()
            .all(db)
            .await?;

        Ok(tasks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    #[ignore]
    async fn test_query_downloads_info() -> Result<()> {
        dotenv::dotenv().ok();
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_target(true)
            .init();
        let db = Db::from_env().await?;
        let downloads = db
            .query_downloads_info(0, 10, Some(DownloadStatus::Completed))
            .await?;
        println!("{:?}", downloads);
        Ok(())
    }
}
