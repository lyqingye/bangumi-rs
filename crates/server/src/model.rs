use chrono::{NaiveDate, NaiveDateTime};
use model::sea_orm_active_enums::{DownloadStatus, Kind, ParserStatus, State, SubscribeStatus};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Resp<T> {
    code: i32,
    msg: Option<String>,
    data: Option<T>,
}

impl<T> Resp<T> {
    pub fn ok(data: T) -> Self {
        Self {
            code: 0,
            msg: None,
            data: Some(data),
        }
    }
    pub fn err(code: i32, msg: String) -> Self {
        Self {
            code: code,
            msg: Some(msg),
            data: None,
        }
    }

    pub fn err_msg<M: AsRef<str>>(msg: M) -> Self {
        Self {
            code: -1,
            msg: Some(msg.as_ref().to_string()),
            data: None,
        }
    }
}

/// API
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, FromQueryResult)]
pub struct Bangumi {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub bangumi_tv_id: Option<i32>,
    pub tmdb_id: Option<u64>,
    pub mikan_id: Option<i32>,
    pub poster_image_url: Option<String>,
    pub air_date: Option<NaiveDateTime>,
    pub air_week: Option<i32>,
    pub rating: Option<f64>,
    pub ep_count: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub backdrop_image_url: Option<String>,
    pub season_number: Option<u64>,
    pub subscribe_status: Option<SubscribeStatus>,
    pub start_episode_number: Option<i32>,
    pub resolution_filter: Option<String>,
    pub language_filter: Option<String>,
    pub release_group_filter: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromQueryResult)]
pub struct Episode {
    pub id: i32,
    pub bangumi_id: i32,
    pub number: i32,
    pub sort_number: Option<i32>,
    pub name: Option<String>,
    pub image_url: Option<String>,
    pub description: Option<String>,
    pub air_date: Option<NaiveDate>,
    pub duration_seconds: Option<u64>,
    pub kind: Kind,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub download_state: Option<State>,
    pub ref_torrent_info_hash: Option<String>,
    pub task_created_at: Option<NaiveDateTime>,
    pub task_updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, FromQueryResult)]
pub struct Torrent {
    // 种子基本信息
    pub info_hash: String,
    pub title: String,
    pub size: i64,
    pub magnet: String,
    pub pub_date: NaiveDateTime,

    // 文件名解析信息
    pub release_group: Option<String>,
    pub season_number: Option<i32>,
    pub episode_number: Option<i32>,
    pub language: Option<String>,
    pub video_resolution: Option<String>,
    pub parser_status: Option<ParserStatus>,

    // 下载任务信息
    pub download_status: Option<DownloadStatus>,
    pub downloader_name: Option<String>,
    pub task_created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscribeParams {
    pub status: SubscribeStatus,
    pub start_episode_number: Option<i32>,
    pub resolution_filter: Option<String>,
    pub language_filter: Option<String>,
    pub release_group_filter: Option<String>,
    pub collector_interval: Option<i32>,
    pub metadata_interval: Option<i32>,
}

// 定义一个结构体来接收查询结果
#[derive(Debug, serde::Serialize, FromQueryResult)]
pub struct DownloadTask {
    pub name: String,
    pub episode_number: i32,
    pub info_hash: String,
    pub file_name: String,
    pub download_status: DownloadStatus,
    pub downloader: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub err_msg: Option<String>,
    pub retry_count: i32,
}

#[derive(Debug, Deserialize)]
pub struct QueryDownloadTask {
    pub offset: u64,
    pub limit: u64,
    pub status: Option<DownloadStatus>,
}
