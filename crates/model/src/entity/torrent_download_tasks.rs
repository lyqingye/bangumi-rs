//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.0

use super::sea_orm_active_enums::DownloadStatus;
use super::sea_orm_active_enums::ResourceType;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "torrent_download_tasks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub info_hash: String,
    pub download_status: DownloadStatus,
    pub downloader: String,
    pub allow_fallback: bool,
    pub dir: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub context: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub err_msg: Option<String>,
    pub retry_count: i32,
    pub next_retry_at: DateTime,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub resource_type: ResourceType,
    #[sea_orm(column_type = "Text", nullable)]
    pub magnet: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub torrent_url: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
