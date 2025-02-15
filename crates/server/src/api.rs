use std::sync::Arc;

use actix_web::{
    get, post,
    web::{self, Json},
    HttpRequest, HttpResponse,
};
use dict::DictCode;
use model::{sea_orm_active_enums::State, sea_orm_active_enums::SubscribeStatus};
use parser::{Language, VideoResolution};
use sea_orm::{prelude::Expr, Condition};
use tracing::{info, instrument};

use crate::{
    error::ServerError,
    model::{Bangumi, Episode, Resp, SubscribeParams, Torrent},
    server::{AppState, ASSETS_MOUNT_PATH},
};

#[instrument(skip(state))]
#[get("/api/calendar")]
pub async fn calendar(
    state: web::Data<Arc<AppState>>,
) -> Result<Json<Resp<Vec<Bangumi>>>, ServerError> {
    use model::bangumi::Column as BangumiColumn;
    use model::bangumi::Entity as Bangumis;
    use model::subscriptions::Column as SubscriptionColumn;
    use model::subscriptions::Entity as Subscriptions;
    use sea_orm::{ColumnTrait, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect};

    let calendar_season = state
        .dict
        .get_value(DictCode::CurrentSeasonSchedule)
        .await?
        .unwrap_or_default();

    let bangumis = Bangumis::find()
        .select_only()
        // Bangumi 字段
        .column(BangumiColumn::Id)
        .column(BangumiColumn::Name)
        .column(BangumiColumn::Description)
        .column(BangumiColumn::BangumiTvId)
        .column(BangumiColumn::TmdbId)
        .column(BangumiColumn::MikanId)
        .column(BangumiColumn::PosterImageUrl)
        .column(BangumiColumn::AirDate)
        .column(BangumiColumn::AirWeek)
        .column(BangumiColumn::Rating)
        .column(BangumiColumn::EpCount)
        .column(BangumiColumn::CreatedAt)
        .column(BangumiColumn::UpdatedAt)
        .column(BangumiColumn::BackdropImageUrl)
        .column(BangumiColumn::SeasonNumber)
        // Subscription 字段
        .column(SubscriptionColumn::SubscribeStatus)
        .column(SubscriptionColumn::StartEpisodeNumber)
        .column(SubscriptionColumn::ResolutionFilter)
        .column(SubscriptionColumn::LanguageFilter)
        .column(SubscriptionColumn::ReleaseGroupFilter)
        // 联表查询
        .join_rev(
            JoinType::LeftJoin,
            Subscriptions::belongs_to(Bangumis)
                .from(SubscriptionColumn::BangumiId)
                .to(BangumiColumn::Id)
                .into(),
        )
        // 时间范围过滤
        .filter(BangumiColumn::CalendarSeason.eq(calendar_season))
        .order_by_asc(BangumiColumn::AirDate)
        .into_model::<Bangumi>()
        .all(state.db.conn())
        .await
        .map_err(|e| ServerError::Internal3(e))?;

    // 处理图片路径
    let bangumis = bangumis
        .into_iter()
        .map(|mut bangumi| {
            if let Some(image) = &mut bangumi.poster_image_url {
                *image = format!("{}/{}", ASSETS_MOUNT_PATH, image);
            }
            if let Some(image) = &mut bangumi.backdrop_image_url {
                *image = format!("{}/{}", ASSETS_MOUNT_PATH, image);
            }
            bangumi
        })
        .collect();

    Ok(Json(Resp::ok(bangumis)))
}

#[instrument(skip(state), fields(id = %id))]
#[get("/api/bangumi/{id}")]
pub async fn get_bangumi_by_id(
    state: web::Data<Arc<AppState>>,
    id: web::Path<i32>,
) -> Result<Json<Resp<Bangumi>>, ServerError> {
    use model::bangumi::Column as BangumiColumn;
    use model::bangumi::Entity as Bangumis;
    use model::subscriptions::Column as SubscriptionColumn;
    use model::subscriptions::Entity as Subscriptions;
    use sea_orm::{ColumnTrait, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect};

    let bangumi = Bangumis::find()
        .select_only()
        // Bangumi 字段
        .column(BangumiColumn::Id)
        .column(BangumiColumn::Name)
        .column(BangumiColumn::Description)
        .column(BangumiColumn::BangumiTvId)
        .column(BangumiColumn::TmdbId)
        .column(BangumiColumn::MikanId)
        .column(BangumiColumn::PosterImageUrl)
        .column(BangumiColumn::AirDate)
        .column(BangumiColumn::AirWeek)
        .column(BangumiColumn::Rating)
        .column(BangumiColumn::EpCount)
        .column(BangumiColumn::CreatedAt)
        .column(BangumiColumn::UpdatedAt)
        .column(BangumiColumn::BackdropImageUrl)
        .column(BangumiColumn::SeasonNumber)
        // Subscription 字段
        .column(SubscriptionColumn::SubscribeStatus)
        .column(SubscriptionColumn::StartEpisodeNumber)
        .column(SubscriptionColumn::ResolutionFilter)
        .column(SubscriptionColumn::LanguageFilter)
        .column(SubscriptionColumn::ReleaseGroupFilter)
        // 联表查询
        .join_rev(
            JoinType::LeftJoin,
            Subscriptions::belongs_to(Bangumis)
                .from(SubscriptionColumn::BangumiId)
                .to(BangumiColumn::Id)
                .into(),
        )
        .filter(BangumiColumn::Id.eq(id.into_inner()))
        .order_by_asc(BangumiColumn::AirDate)
        .into_model::<Bangumi>()
        .one(state.db.conn())
        .await
        .map_err(|e| ServerError::Internal3(e))?;

    match bangumi {
        Some(mut bgm) => {
            if let Some(image) = &mut bgm.poster_image_url {
                *image = format!("{}/{}", ASSETS_MOUNT_PATH, image);
            }
            if let Some(image) = &mut bgm.backdrop_image_url {
                *image = format!("{}/{}", ASSETS_MOUNT_PATH, image);
            }
            Ok(Json(Resp::ok(bgm)))
        }
        None => Err(ServerError::BangumiNotFound),
    }
}

#[instrument(skip(state), fields(id = %id))]
#[get("/api/bangumi/{id}/episodes")]
pub async fn get_bangumi_episodes_by_id(
    state: web::Data<Arc<AppState>>,
    id: web::Path<i32>,
) -> Result<Json<Resp<Vec<Episode>>>, ServerError> {
    use model::episode_download_tasks::Column as TaskColumn;
    use model::episode_download_tasks::Entity as Tasks;
    use model::episodes::Column as EpisodeColumn;
    use model::episodes::Entity as Episodes;
    use sea_orm::{ColumnTrait, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect};

    let bangumi_id = id.into_inner();
    let episodes = Episodes::find()
        .select_only()
        // Episodes 字段
        .column(EpisodeColumn::Id)
        .column(EpisodeColumn::BangumiId)
        .column(EpisodeColumn::Number)
        .column(EpisodeColumn::SortNumber)
        .column(EpisodeColumn::Name)
        .column(EpisodeColumn::ImageUrl)
        .column(EpisodeColumn::Description)
        .column(EpisodeColumn::AirDate)
        .column(EpisodeColumn::DurationSeconds)
        .column(EpisodeColumn::Kind)
        .column(EpisodeColumn::CreatedAt)
        .column(EpisodeColumn::UpdatedAt)
        // Tasks 字段
        .column_as(TaskColumn::State, "download_state")
        .column(TaskColumn::RefTorrentInfoHash)
        .column_as(TaskColumn::CreatedAt, "task_created_at")
        .column_as(TaskColumn::UpdatedAt, "task_updated_at")
        // 联表查询
        .join_rev(
            JoinType::LeftJoin,
            Tasks::belongs_to(Episodes) // Define the relationship
                .from(TaskColumn::BangumiId) // from Task table
                .to(EpisodeColumn::BangumiId) // to Episode table
                .on_condition(|left_table, right_table| {
                    Condition::all().add(
                        Expr::col((left_table, TaskColumn::EpisodeNumber))
                            .eq(Expr::col((right_table, EpisodeColumn::Number))),
                    )
                })
                .into(),
        )
        .filter(EpisodeColumn::BangumiId.eq(bangumi_id))
        .order_by_asc(EpisodeColumn::Number)
        .into_model::<Episode>()
        .all(state.db.conn())
        .await
        .map_err(|e| ServerError::Internal3(e))?;

    Ok(Json(Resp::ok(episodes)))
}

#[instrument(skip(state), fields(id = %id))]
#[post("/api/bangumi/{id}/subscribe")]
pub async fn subscribe_bangumi(
    state: web::Data<Arc<AppState>>,
    id: web::Path<i32>,
    params: Json<SubscribeParams>,
) -> Result<Json<Resp<()>>, ServerError> {
    match params.status {
        SubscribeStatus::Subscribed => {
            // 将分辨率字符串转换为VideoResolution枚举列表
            let resolution_filter = params.resolution_filter.as_ref().map(|filter| {
                filter
                    .split(',')
                    .filter_map(|res| {
                        if !res.is_empty() {
                            Some(VideoResolution::from(res.trim()))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            });

            // 将语言字符串转换为Language枚举列表
            let language_filter = params.language_filter.as_ref().map(|filter| {
                filter
                    .split(',')
                    .filter_map(|lang| {
                        if !lang.is_empty() {
                            Some(Language::from(lang.trim()))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            });

            state
                .scheduler
                .subscribe(
                    id.into_inner(),
                    params.start_episode_number,
                    resolution_filter,
                    language_filter,
                    params.release_group_filter.clone(),
                    params.collector_interval,
                    params.metadata_interval,
                    None,
                )
                .await?;
        }
        SubscribeStatus::None => {
            state.scheduler.unsubscribe(id.into_inner()).await?;
        }
        _ => {}
    }
    Ok(Json(Resp::ok(())))
}

#[instrument(skip(state), fields(id = %id))]
#[get("/api/bangumi/{id}/delete_download_tasks")]
pub async fn delete_bangumi_download_tasks(
    state: web::Data<Arc<AppState>>,
    id: web::Path<i32>,
) -> Result<Json<Resp<()>>, ServerError> {
    let bangumi_id = id.into_inner();
    // 先取消订阅
    state.scheduler.unsubscribe(bangumi_id).await?;
    state.db.delete_bangumi_download_tasks(bangumi_id).await?;
    Ok(Json(Resp::ok(())))
}

#[instrument(skip(state), fields(id = %id))]
#[get("/api/bangumi/{id}/torrents")]
pub async fn get_bangumi_torrents_by_id(
    state: web::Data<Arc<AppState>>,
    id: web::Path<i32>,
) -> Result<Json<Resp<Vec<Torrent>>>, ServerError> {
    use model::bangumi::Column as BangumiColumn;
    use model::bangumi::Entity as Bangumis;
    use model::file_name_parse_record::Column as ParseColumn;
    use model::file_name_parse_record::Entity as ParseRecord;
    use model::torrent_download_tasks::Column as TaskColumn;
    use model::torrent_download_tasks::Entity as Tasks;
    use model::torrents::Column as TorrentColumn;
    use model::torrents::Entity as Torrents;
    use sea_orm::{ColumnTrait, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect};

    // 1. 获取番剧的所有剧集信息，找到最小剧集编号
    let bangumi_id = id.into_inner();
    let bangumi = Bangumis::find()
        .filter(BangumiColumn::Id.eq(bangumi_id))
        .one(state.db.conn())
        .await
        .map_err(|e| ServerError::Internal3(e))?
        .ok_or(ServerError::BangumiNotFound)?;

    let min_ep = bangumi.ep_start_number;

    // 2. 获取种子信息及其解析结果
    let mut torrents = Torrents::find()
        .select_only()
        .column(TorrentColumn::InfoHash)
        .column(TorrentColumn::Title)
        .column(TorrentColumn::Size)
        .column(TorrentColumn::Magnet)
        .column(TorrentColumn::PubDate)
        // 文件名解析信息
        .column(ParseColumn::ReleaseGroup)
        .column(ParseColumn::SeasonNumber)
        .column(ParseColumn::EpisodeNumber)
        .column(ParseColumn::Language)
        .column(ParseColumn::VideoResolution)
        .column(ParseColumn::ParserStatus)
        // 下载任务信息
        .column(TaskColumn::DownloadStatus)
        .column(TaskColumn::Downloader)
        .column_as(TaskColumn::CreatedAt, "task_created_at")
        // Joins
        .join_rev(
            JoinType::LeftJoin,
            ParseRecord::belongs_to(Torrents)
                .from(ParseColumn::FileName)
                .to(TorrentColumn::Title)
                .into(),
        )
        .join_rev(
            JoinType::LeftJoin,
            Tasks::belongs_to(Torrents)
                .from(TaskColumn::InfoHash)
                .to(TorrentColumn::InfoHash)
                .into(),
        )
        .filter(TorrentColumn::BangumiId.eq(bangumi_id))
        .order_by_desc(TorrentColumn::PubDate)
        .into_model::<Torrent>()
        .all(state.db.conn())
        .await
        .map_err(|e| ServerError::Internal3(e))?;

    // 3. 处理剧集编号映射
    for torrent in &mut torrents {
        if let Some(ep) = torrent.episode_number {
            // 剧集修复:
            // 例如: 某些番剧第二季可能从第13集开始,但种子标记为第1集
            // ep_start_number = 13, ep = 1 时:
            // actual_ep = 1 + 13 - 1 = 13,修正为实际的第13集
            if min_ep > 1 && ep < min_ep {
                torrent.episode_number = Some(min_ep + ep - 1);
            }
        }
    }

    Ok(Json(Resp::ok(torrents)))
}

#[instrument(skip(state), fields(id = %id))]
#[get("/api/bangumi/{id}/refresh")]
pub async fn refresh_bangumi(
    state: web::Data<Arc<AppState>>,
    id: web::Path<i32>,
) -> Result<Json<Resp<()>>, ServerError> {
    let id = id.into_inner();
    state.scheduler.trigger_collection(id).await?;
    Ok(Json(Resp::ok(())))
}

#[get("/api/bangumi/{id}/{episode_number}/online_watch")]
pub async fn online_watch(
    req: HttpRequest,
    state: web::Data<Arc<AppState>>,
    path: web::Path<(i32, i32)>,
) -> Result<HttpResponse, ServerError> {
    use model::episode_download_tasks::Column as TaskColumn;
    use model::episode_download_tasks::Entity as EpisodeDownloadTasks;
    use sea_orm::ColumnTrait;
    use sea_orm::EntityTrait;
    use sea_orm::QueryFilter;

    let (id, episode_number) = path.into_inner();

    // 获取 User-Agent，如果没有则使用默认值
    let user_agent = req
        .headers()
        .get("User-Agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Unknown Browser");

    // 查找下载任务
    let task = EpisodeDownloadTasks::find()
        .filter(TaskColumn::BangumiId.eq(id))
        .filter(TaskColumn::EpisodeNumber.eq(episode_number))
        .filter(TaskColumn::State.eq(State::Downloaded))
        .one(state.db.conn())
        .await?
        .ok_or_else(|| ServerError::Internal2(anyhow::anyhow!("剧集未下载")))?;

    // 获取种子信息
    let info_hash = task
        .ref_torrent_info_hash
        .ok_or_else(|| ServerError::Internal2(anyhow::anyhow!("种子未找到")))?;

    // 获取下载信息
    let download_info = state
        .scheduler
        .get_downloader()
        .download_file(&info_hash, user_agent)
        .await?;

    info!("在线播放成功: {}", download_info.file_name);

    // 构建响应
    Ok(HttpResponse::Found()
        .content_type("text/html; charset=utf-8")
        .append_header(("Referrer-Policy", "no-referrer"))
        .append_header((
            "Cache-Control",
            "max-age=0, no-cache, no-store, must-revalidate",
        ))
        .append_header(("Location", download_info.url.url.clone()))
        .body(format!(
            r#"<a href="{url}">Found</a>"#,
            url = download_info.url.url
        )))
}

#[get("/health")]
pub async fn health() -> Result<Json<Resp<()>>, ServerError> {
    Ok(Json(Resp::ok(())))
}
