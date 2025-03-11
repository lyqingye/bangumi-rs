use anyhow::Result;
use model::{
    episode_download_tasks, file_name_parse_record, sea_orm_active_enums::State, subscriptions,
    torrents,
};
use sea_orm::Set;
use sea_orm::{
    ColumnTrait, Condition, ConnectOptions, Database, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use std::collections::HashSet;
use std::{sync::Arc, time::Duration};

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

    /// 获取指定番剧的订阅设置
    pub async fn get_subscription(&self, bangumi_id: i32) -> Result<Option<subscriptions::Model>> {
        use model::subscriptions::Column as SubscriptionColumn;
        use model::subscriptions::Entity as Subscriptions;

        let subscription = Subscriptions::find()
            .filter(SubscriptionColumn::BangumiId.eq(bangumi_id))
            .one(self.conn())
            .await?;

        Ok(subscription)
    }

    /// 获取所有未完成下载任务
    pub async fn get_all_unfinished_tasks(&self) -> Result<Vec<episode_download_tasks::Model>> {
        use model::episode_download_tasks::Column as TaskColumn;
        use model::episode_download_tasks::Entity as Tasks;

        let unfinished_states = vec![State::Ready, State::Downloading, State::Retrying];

        let tasks = Tasks::find()
            .filter(Condition::all().add(TaskColumn::State.is_in(unfinished_states)))
            .order_by_asc(TaskColumn::CreatedAt)
            .all(self.conn())
            .await?;

        Ok(tasks)
    }

    /// 获取指定番剧的未完成下载任务
    pub async fn get_unfinished_tasks_by_bangumi(
        &self,
        bangumi_id: i32,
    ) -> Result<Vec<episode_download_tasks::Model>> {
        use model::episode_download_tasks::Column as TaskColumn;
        use model::episode_download_tasks::Entity as Tasks;

        let unfinished_states = vec![
            State::Missing,
            State::Ready,
            State::Downloading,
            State::Failed,
            State::Retrying,
        ];

        let tasks = Tasks::find()
            .filter(
                Condition::all()
                    .add(TaskColumn::BangumiId.eq(bangumi_id))
                    .add(TaskColumn::State.is_in(unfinished_states)),
            )
            .order_by_asc(TaskColumn::CreatedAt)
            .all(self.conn())
            .await?;

        Ok(tasks)
    }

    /// 获取所有活跃的订阅
    pub async fn get_active_subscriptions(&self) -> Result<Vec<subscriptions::Model>> {
        use model::sea_orm_active_enums::SubscribeStatus;
        use model::subscriptions::Column as SubscriptionColumn;
        use model::subscriptions::Entity as Subscriptions;

        let subscriptions = Subscriptions::find()
            .filter(SubscriptionColumn::SubscribeStatus.eq(SubscribeStatus::Subscribed))
            .all(self.conn())
            .await?;

        Ok(subscriptions)
    }

    /// 更新任务状态
    pub async fn update_task_state(
        &self,
        bangumi_id: i32,
        episode_number: i32,
        state: State,
    ) -> Result<()> {
        use model::episode_download_tasks::Column as TaskColumn;
        use model::episode_download_tasks::Entity as Tasks;

        Tasks::update_many()
            .col_expr(TaskColumn::State, state.into())
            .filter(
                Condition::all()
                    .add(TaskColumn::BangumiId.eq(bangumi_id))
                    .add(TaskColumn::EpisodeNumber.eq(episode_number)),
            )
            .exec(self.conn())
            .await?;

        Ok(())
    }

    /// 更新任务状态为就绪，并设置选中的种子
    pub async fn update_task_ready(
        &self,
        bangumi_id: i32,
        episode_number: i32,
        info_hash: &str,
    ) -> Result<()> {
        use model::episode_download_tasks::Column as TaskColumn;
        use model::episode_download_tasks::Entity as Tasks;

        Tasks::update_many()
            .col_expr(TaskColumn::State, State::Ready.into())
            .col_expr(TaskColumn::RefTorrentInfoHash, info_hash.into())
            .filter(
                Condition::all()
                    .add(TaskColumn::BangumiId.eq(bangumi_id))
                    .add(TaskColumn::EpisodeNumber.eq(episode_number)),
            )
            .exec(self.conn())
            .await?;

        Ok(())
    }

    /// 获取番剧的所有种子
    pub async fn get_bangumi_torrents(&self, bangumi_id: i32) -> Result<Vec<torrents::Model>> {
        use model::torrents::Column as TorrentColumn;
        use model::torrents::Entity as Torrents;

        let torrents = Torrents::find()
            .filter(TorrentColumn::BangumiId.eq(bangumi_id))
            .order_by_desc(TorrentColumn::PubDate)
            .all(self.conn())
            .await?;

        Ok(torrents)
    }

    /// 通过 info_hash 获取种子信息
    pub async fn get_torrent_by_info_hash(
        &self,
        info_hash: &str,
    ) -> Result<Option<torrents::Model>> {
        use model::torrents::Column as TorrentColumn;
        use model::torrents::Entity as Torrents;

        let torrent = Torrents::find()
            .filter(TorrentColumn::InfoHash.eq(info_hash))
            .one(self.conn())
            .await?;

        Ok(torrent)
    }

    /// 获取番剧的所有种子及其解析结果
    pub async fn get_bangumi_torrents_with_parse_results(
        &self,
        bangumi_id: i32,
    ) -> Result<Vec<(torrents::Model, file_name_parse_record::Model)>> {
        use model::file_name_parse_record::Column as ParseColumn;
        use model::file_name_parse_record::Entity as ParseRecord;
        use model::sea_orm_active_enums::ParserStatus;
        use model::torrents::Column as TorrentColumn;
        use model::torrents::Entity as Torrents;
        use sea_orm::{JoinType, QuerySelect};

        let results = Torrents::find()
            .select_only()
            .columns([
                TorrentColumn::BangumiId,
                TorrentColumn::Title,
                TorrentColumn::Size,
                TorrentColumn::InfoHash,
                TorrentColumn::Magnet,
                TorrentColumn::PubDate,
                TorrentColumn::Source,
            ])
            .filter(TorrentColumn::BangumiId.eq(bangumi_id))
            .join(
                JoinType::InnerJoin,
                Torrents::belongs_to(ParseRecord)
                    .from(TorrentColumn::Title)
                    .to(ParseColumn::FileName)
                    .into(),
            )
            .select_also(ParseRecord)
            .filter(ParseColumn::ParserStatus.eq(ParserStatus::Completed))
            .order_by_desc(TorrentColumn::PubDate)
            .all(self.conn())
            .await?;

        // 转换结果格式，过滤掉没有解析结果的记录
        let pairs = results
            .into_iter()
            .filter_map(|(torrent, parse_result)| {
                parse_result.map(|parse_result| (torrent, parse_result))
            })
            .collect();

        Ok(pairs)
    }

    pub async fn list_torrent_download_tasks_by_info_hashes(
        &self,
        info_hashes: &[String],
    ) -> Result<HashSet<String>> {
        use model::torrent_download_tasks::Column as TorrentDownloadTasksColumn;
        use model::torrent_download_tasks::Entity as TorrentDownloadTasks;
        let tasks_hashes = TorrentDownloadTasks::find()
            .select_only()
            .columns([TorrentDownloadTasksColumn::InfoHash])
            .filter(TorrentDownloadTasksColumn::InfoHash.is_in(info_hashes))
            .into_tuple::<String>()
            .all(self.conn())
            .await?
            .into_iter()
            .collect();
        Ok(tasks_hashes)
    }

    pub async fn get_bangumi_by_id(
        &self,
        bangumi_id: i32,
    ) -> Result<Option<model::bangumi::Model>> {
        use model::bangumi::Column as BangumiColumn;
        use model::bangumi::Entity as Bangumis;

        let bangumi = Bangumis::find()
            .filter(BangumiColumn::Id.eq(bangumi_id))
            .one(self.conn())
            .await?;
        Ok(bangumi)
    }

    /// 获取番剧的所有剧集信息
    pub async fn get_bangumi_episodes(
        &self,
        bangumi_id: i32,
    ) -> Result<Vec<model::episodes::Model>> {
        use model::episodes::Column as EpisodeColumn;
        use model::episodes::Entity as Episodes;

        let episodes = Episodes::find()
            .filter(EpisodeColumn::BangumiId.eq(bangumi_id))
            .order_by_asc(EpisodeColumn::Number)
            .all(self.conn())
            .await?;

        Ok(episodes)
    }

    /// 批量创建下载任务
    pub async fn batch_create_tasks(
        &self,
        tasks: Vec<episode_download_tasks::ActiveModel>,
    ) -> Result<()> {
        use model::episode_download_tasks::{Column, Entity as Tasks};
        use sea_orm::sea_query::OnConflict;

        if !tasks.is_empty() {
            Tasks::insert_many(tasks)
                .on_conflict(
                    OnConflict::columns([Column::BangumiId, Column::EpisodeNumber])
                        .update_column(Column::UpdatedAt)
                        .to_owned(),
                )
                .exec(self.conn())
                .await?;
        }

        Ok(())
    }

    pub async fn get_episode_task_by_bangumi_id_and_episode_number(
        &self,
        bangumi_id: i32,
        episode_number: i32,
    ) -> Result<Option<episode_download_tasks::Model>> {
        use model::episode_download_tasks::Column as TaskColumn;
        use model::episode_download_tasks::Entity as Tasks;
        let task = Tasks::find()
            .filter(
                Condition::all()
                    .add(TaskColumn::BangumiId.eq(bangumi_id))
                    .add(TaskColumn::EpisodeNumber.eq(episode_number)),
            )
            .one(self.conn())
            .await?;
        Ok(task)
    }

    pub async fn get_episode_task_by_info_hash(
        &self,
        info_hash: &str,
    ) -> Result<Option<episode_download_tasks::Model>> {
        use model::episode_download_tasks::Column as TaskColumn;
        use model::episode_download_tasks::Entity as Tasks;
        let task = Tasks::find()
            .filter(TaskColumn::RefTorrentInfoHash.eq(info_hash))
            .one(self.conn())
            .await?;
        Ok(task)
    }

    /// 更新或创建订阅记录
    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_subscription(
        &self,
        bangumi_id: i32,
        start_episode_number: Option<i32>,
        resolution_filter: Option<String>,
        language_filter: Option<String>,
        release_group_filter: Option<String>,
        collector_interval: Option<i32>,
        metadata_interval: Option<i32>,
        enforce_torrent_release_after_broadcast: bool,
    ) -> Result<()> {
        use model::subscriptions::Column as SubscriptionColumn;
        use model::subscriptions::Entity as Subscriptions;

        let subscription = model::subscriptions::ActiveModel {
            bangumi_id: Set(bangumi_id),
            subscribe_status: Set(model::sea_orm_active_enums::SubscribeStatus::Subscribed),
            resolution_filter: Set(resolution_filter),
            language_filter: Set(language_filter),
            release_group_filter: Set(release_group_filter),
            collector_interval: Set(collector_interval),
            metadata_interval: Set(metadata_interval),
            start_episode_number: Set(start_episode_number),
            enforce_torrent_release_after_broadcast: Set(
                enforce_torrent_release_after_broadcast as i8
            ),
            ..Default::default()
        };

        // 尝试插入新记录，如果已存在则更新状态
        Subscriptions::insert(subscription)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(SubscriptionColumn::BangumiId)
                    .update_column(SubscriptionColumn::SubscribeStatus)
                    .update_column(SubscriptionColumn::ResolutionFilter)
                    .update_column(SubscriptionColumn::LanguageFilter)
                    .update_column(SubscriptionColumn::ReleaseGroupFilter)
                    .update_column(SubscriptionColumn::CollectorInterval)
                    .update_column(SubscriptionColumn::MetadataInterval)
                    .update_column(SubscriptionColumn::StartEpisodeNumber)
                    .update_column(SubscriptionColumn::EnforceTorrentReleaseAfterBroadcast)
                    .to_owned(),
            )
            .exec(self.conn())
            .await?;

        Ok(())
    }

    /// 更新订阅状态为未订阅
    pub async fn unsubscribe(&self, bangumi_id: i32) -> Result<()> {
        use model::sea_orm_active_enums::SubscribeStatus;
        use model::subscriptions::Column as SubscriptionColumn;
        use model::subscriptions::Entity as Subscriptions;

        Subscriptions::update_many()
            .col_expr(
                SubscriptionColumn::SubscribeStatus,
                SubscribeStatus::None.into(),
            )
            .filter(SubscriptionColumn::BangumiId.eq(bangumi_id))
            .exec(self.conn())
            .await?;

        Ok(())
    }

    pub async fn update_subscription_as_downloaded(&self, bangumi_id: i32) -> Result<()> {
        use model::sea_orm_active_enums::SubscribeStatus;
        use model::subscriptions::Column as SubscriptionColumn;
        use model::subscriptions::Entity as Subscriptions;

        Subscriptions::update_many()
            .col_expr(
                SubscriptionColumn::SubscribeStatus,
                SubscribeStatus::Downloaded.into(),
            )
            .filter(SubscriptionColumn::BangumiId.eq(bangumi_id))
            .exec(self.conn())
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_example() -> Result<()> {
        dotenv::dotenv().ok();
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_target(true)
            .init();
        let db = Db::new_from_env().await?;
        let torrents = db.get_bangumi_torrents_with_parse_results(478).await?;
        let info_hashes = torrents
            .iter()
            .map(|t| t.0.info_hash.clone())
            .collect::<Vec<String>>();
        let info_hashes = db
            .list_torrent_download_tasks_by_info_hashes(&info_hashes)
            .await?;
        println!("torrents: {:?}", torrents);
        println!("info_hashes: {:?}", info_hashes);
        Ok(())
    }
}
