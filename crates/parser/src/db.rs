use anyhow::Result;
use model::{
    file_name_parse_record::{self, Column},
    prelude::FileNameParseRecord,
    sea_orm_active_enums::ParserStatus,
};
use sea_orm::{
    ColumnTrait, ConnectOptions, Database, DatabaseConnection, EntityTrait, IntoActiveModel,
    QueryFilter, sea_query::OnConflict,
};
use std::{sync::Arc, time::Duration};

use crate::{Language, ParseResult, VideoResolution};

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

    /// 保存解析结果
    pub async fn save_parse_results(&self, results: &[ParseResult]) -> Result<()> {
        let records = Self::result_to_record(results);
        self.batch_upsert(records).await
    }

    /// 保存解析错误
    pub async fn save_parse_errors(&self, file_names: &[String], error: &str) -> Result<()> {
        let now = chrono::Utc::now().naive_utc();
        let records: Vec<file_name_parse_record::Model> = file_names
            .iter()
            .map(|file_name| file_name_parse_record::Model {
                file_name: file_name.to_string(),
                parser_status: ParserStatus::Failed,
                err_msg: Some(error.to_string()),
                created_at: now,
                updated_at: now,
                release_group: None,
                bangumi_name: None,
                year: None,
                episode_number: None,
                season_number: None,
                language: None,
                parser_name: String::new(),
                video_resolution: None,
            })
            .collect();
        self.batch_upsert(records).await
    }

    /// 根据状态查询解析结果
    pub async fn get_parse_results_by_status(
        &self,
        status: ParserStatus,
    ) -> Result<Vec<ParseResult>> {
        let records = self.list_by_status(status).await?;
        Ok(Self::record_to_result(&records))
    }

    // 内部方法
    async fn list_by_status(
        &self,
        status: ParserStatus,
    ) -> Result<Vec<file_name_parse_record::Model>> {
        let db = self.conn();
        let records = FileNameParseRecord::find()
            .filter(Column::ParserStatus.eq(status))
            .all(db)
            .await?;
        Ok(records)
    }

    pub async fn list_by_file_names(
        &self,
        file_names: &[String],
    ) -> Result<Vec<file_name_parse_record::Model>> {
        let db = self.conn();
        let records = FileNameParseRecord::find()
            .filter(Column::FileName.is_in(file_names))
            .all(db)
            .await?;
        Ok(records)
    }

    #[cfg(test)]
    pub async fn list_all(&self) -> Result<Vec<file_name_parse_record::Model>> {
        let db = self.conn();
        let records = FileNameParseRecord::find().all(db).await?;
        Ok(records)
    }

    async fn batch_upsert(&self, records: Vec<file_name_parse_record::Model>) -> Result<()> {
        if records.is_empty() {
            return Ok(());
        }
        let db = self.conn();
        FileNameParseRecord::insert_many(records.into_iter().map(|r| r.into_active_model()))
            .on_conflict(
                OnConflict::column(Column::FileName)
                    .update_columns([
                        Column::ParserStatus,
                        Column::ReleaseGroup,
                        Column::BangumiName,
                        Column::Year,
                        Column::EpisodeNumber,
                        Column::SeasonNumber,
                        Column::Language,
                        Column::VideoResolution,
                        Column::ParserName,
                        Column::ErrMsg,
                        Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(db)
            .await?;
        Ok(())
    }

    fn result_to_record(results: &[ParseResult]) -> Vec<file_name_parse_record::Model> {
        let now = chrono::Utc::now().naive_utc();
        results
            .iter()
            .map(|r| file_name_parse_record::Model {
                file_name: r.file_name.clone(),
                parser_status: ParserStatus::Completed,
                release_group: r.release_group.clone(),
                bangumi_name: r.title.clone(),
                year: r.year,
                episode_number: r.episode,
                season_number: r.season,
                language: Some(
                    r.languages
                        .iter()
                        .map(|l| l.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                ),
                parser_name: String::new(),
                err_msg: None,
                created_at: now,
                updated_at: now,
                video_resolution: r.video_resolution.clone().map(|v| v.to_string()),
            })
            .collect()
    }

    /// 将数据库记录转换为解析结果
    pub fn record_to_result(records: &[file_name_parse_record::Model]) -> Vec<ParseResult> {
        records
            .iter()
            .filter(|r| r.parser_status == ParserStatus::Completed)
            .map(|record| ParseResult {
                file_name: record.file_name.clone(),
                release_group: record.release_group.clone(),
                title: record.bangumi_name.clone(),
                year: record.year,
                episode: record.episode_number,
                season: record.season_number,
                video_resolution: record
                    .video_resolution
                    .as_ref()
                    .map(|v| VideoResolution::from(v.as_str())),
                languages: record
                    .language
                    .as_ref()
                    .map(|l| l.split(',').map(Language::from).collect())
                    .unwrap_or_default(),
            })
            .collect()
    }
}
