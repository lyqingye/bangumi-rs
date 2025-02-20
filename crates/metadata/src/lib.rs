#![allow(unused)]
mod db;
pub mod fetcher;
mod mdb_bgmtv;
mod mdb_mikan;
mod mdb_tmdb;
pub mod worker;
use anyhow::Result;
use async_trait::async_trait;
use model::bangumi;

fn format_poster_image_file_name(bgm: &bangumi::Model) -> String {
    format!("bangumi_poster_{}", bgm.id)
}

fn format_backdrop_image_file_name(bgm: &bangumi::Model) -> String {
    format!("bangumi_backdrop_{}", bgm.id)
}

#[derive(Debug, Clone)]
enum MetadataAttr {
    /// 基本信息
    Name,
    Description,

    /// 评分
    Rating,

    // 放送信息
    AirDate,
    AirWeek,

    /// Season基本信息
    EpCount,
    SeasonNumber,
    EpStartNumber,

    /// 封面以及海报信息
    Poster,
    Backdrop,
}

#[async_trait]
pub trait MetadataDb {
    /// 更新番剧元数据
    async fn update_bangumi_metadata(&self, bgm: &mut bangumi::Model, force: bool) -> Result<()>;
}
