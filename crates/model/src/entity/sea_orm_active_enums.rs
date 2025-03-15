//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.0

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "bgm_kind")]
pub enum BgmKind {
    #[sea_orm(string_value = "anime")]
    Anime,
    #[sea_orm(string_value = "movie")]
    Movie,
}
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "download_status")]
pub enum DownloadStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "downloading")]
    Downloading,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "failed")]
    Failed,
    #[sea_orm(string_value = "retrying")]
    Retrying,
    #[sea_orm(string_value = "cancelled")]
    Cancelled,
    #[sea_orm(string_value = "paused")]
    Paused,
}
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "kind")]
pub enum Kind {
    #[sea_orm(string_value = "EP")]
    Ep,
    #[sea_orm(string_value = "SP")]
    Sp,
    #[sea_orm(string_value = "OP")]
    Op,
    #[sea_orm(string_value = "ED")]
    Ed,
    #[sea_orm(string_value = "MAD")]
    Mad,
    #[sea_orm(string_value = "Other")]
    Other,
}
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "parser_status")]
pub enum ParserStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "failed")]
    Failed,
}
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "source")]
pub enum Source {
    #[sea_orm(string_value = "Mikan")]
    Mikan,
    #[sea_orm(string_value = "AcgripOrg")]
    AcgripOrg,
    #[sea_orm(string_value = "NyaaLand")]
    NyaaLand,
    #[sea_orm(string_value = "DmhyOrg")]
    DmhyOrg,
    #[sea_orm(string_value = "User")]
    User,
}
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "state")]
pub enum State {
    #[sea_orm(string_value = "missing")]
    Missing,
    #[sea_orm(string_value = "ready")]
    Ready,
    #[sea_orm(string_value = "downloading")]
    Downloading,
    #[sea_orm(string_value = "downloaded")]
    Downloaded,
    #[sea_orm(string_value = "failed")]
    Failed,
    #[sea_orm(string_value = "retrying")]
    Retrying,
}
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "subscribe_status")]
pub enum SubscribeStatus {
    #[sea_orm(string_value = "none")]
    None,
    #[sea_orm(string_value = "subscribed")]
    Subscribed,
    #[sea_orm(string_value = "downloaded")]
    Downloaded,
}
