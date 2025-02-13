//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.0

use super::sea_orm_active_enums::ScraperStatus;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "mikan_scraper_record")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub mikan_id: i32,
    pub url: String,
    pub scraper_status: ScraperStatus,
    #[sea_orm(column_type = "Text", nullable)]
    pub err_msg: Option<String>,
    pub retry_times: i32,
    pub file_name: String,
    pub file_size: Option<i64>,
    pub magnet_link: Option<String>,
    pub sub_group: Option<String>,
    pub release_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
