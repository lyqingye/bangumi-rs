use std::sync::Arc;

use actix_web::{
    get, post,
    web::{self, Json},
    HttpRequest, HttpResponse,
};
use dict::DictCode;
use model::sea_orm_active_enums::{BgmKind, State, SubscribeStatus};
use parser::{Language, VideoResolution};
use sea_orm::{prelude::Expr, Condition};
use tracing::{info, instrument};

use crate::{
    config::Config,
    model::{
        BangumiListResp, CalendarQuery, DownloadTask, Metrics, ProcessMetrics, QueryBangumiParams,
        QueryDownloadTask, TMDBMetadata, TMDBSeason, UpdateMDBParams,
    },
};
use crate::{
    error::ServerError,
    model::{Bangumi, Episode, Resp, SubscribeParams, Torrent},
    router::ASSETS_MOUNT_PATH,
    server::AppState,
};

#[get("/api/calendar/season")]
pub async fn current_calendar_season(
    state: web::Data<Arc<AppState>>,
) -> Result<Json<Resp<String>>, ServerError> {
    let calendar_season = state
        .dict
        .get_value(DictCode::CurrentSeasonSchedule)
        .await?
        .unwrap_or_default();

    Ok(Json(Resp::ok(calendar_season)))
}

#[instrument(skip(state))]
#[get("/api/calendar")]
pub async fn calendar(
    state: web::Data<Arc<AppState>>,
    query: web::Query<CalendarQuery>,
) -> Result<Json<Resp<Vec<Bangumi>>>, ServerError> {
    use model::bangumi::Column as BangumiColumn;
    use model::bangumi::Entity as Bangumis;
    use model::subscriptions::Column as SubscriptionColumn;
    use model::subscriptions::Entity as Subscriptions;
    use sea_orm::{ColumnTrait, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect};

    let calendar_season = match query.season.as_ref() {
        Some(season) if !season.is_empty() => season.clone(),
        _ => state
            .dict
            .get_value(DictCode::CurrentSeasonSchedule)
            .await?
            .unwrap_or_default(),
    };

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
        .column(SubscriptionColumn::EnforceTorrentReleaseAfterBroadcast)
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
        .await?;

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
        .column(SubscriptionColumn::EnforceTorrentReleaseAfterBroadcast)
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
        .await?;

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
        .await?;

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
                    params.enforce_torrent_release_after_broadcast,
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
        .await?
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
        .await?;

    // 3. 处理剧集编号映射和过滤非法种子
    torrents.retain_mut(|torrent| {
        if let Some(ep) = torrent.episode_number {
            // 剧集修复:
            // 例如: 某些番剧第二季可能从第13集开始,但种子标记为第1集
            // ep_start_number = 13, ep = 1 时:
            // actual_ep = 1 + 13 - 1 = 13,修正为实际的第13集
            // let actual_ep = ep;
            if min_ep > 1 && ep < min_ep {
                let actual_ep = min_ep + ep - 1;
                torrent.episode_number = Some(actual_ep);
            }
        }
        true
    });

    Ok(Json(Resp::ok(torrents)))
}

#[instrument(skip(state), fields(id = %params.0))]
#[get("/api/bangumi/{id}/refresh/{force}")]
pub async fn refresh_bangumi(
    state: web::Data<Arc<AppState>>,
    params: web::Path<(i32, bool)>,
) -> Result<Json<Resp<()>>, ServerError> {
    let (id, force) = params.into_inner();
    state.scheduler.trigger_collection(id).await?;
    state.metadata.request_refresh_metadata(id, force).await?;
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
        .ok_or_else(|| ServerError::Internal(anyhow::anyhow!("剧集未下载")))?;

    // 获取种子信息
    let info_hash = task
        .ref_torrent_info_hash
        .ok_or_else(|| ServerError::Internal(anyhow::anyhow!("种子未找到")))?;

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

#[get("/api/bangumi/{bangumi_id}/{episode_number}/manual_select_torrent/{info_hash}")]
pub async fn manual_select_torrent(
    state: web::Data<Arc<AppState>>,
    path: web::Path<(i32, i32, String)>,
) -> Result<Json<Resp<()>>, ServerError> {
    let (bangumi_id, episode_number, info_hash) = path.into_inner();
    state
        .scheduler
        .manual_select_episode_torrent(bangumi_id, episode_number, &info_hash)
        .await?;
    Ok(Json(Resp::ok(())))
}

#[post("/api/downloads")]
pub async fn list_download_tasks(
    state: web::Data<Arc<AppState>>,
    params: Json<QueryDownloadTask>,
) -> Result<Json<Resp<Vec<DownloadTask>>>, ServerError> {
    let downloads = state
        .db
        .query_downloads_info(params.offset, params.limit, params.status.clone())
        .await?;

    Ok(Json(Resp::ok(downloads)))
}

#[get("/api/calendar/refresh/{force}")]
pub async fn refresh_calendar(
    state: web::Data<Arc<AppState>>,
    query: web::Query<CalendarQuery>,
    force: web::Path<bool>,
) -> Result<Json<Resp<()>>, ServerError> {
    let force = force.into_inner();
    let season = query.season.as_ref().filter(|s| !s.is_empty()).cloned();
    state
        .metadata
        .request_refresh_calendar(season, force)
        .await?;
    Ok(Json(Resp::ok(())))
}

#[get("/health")]
pub async fn health() -> Result<Json<Resp<()>>, ServerError> {
    Ok(Json(Resp::ok(())))
}

#[post("/api/bangumi/{bangumi_id}/mdb/update")]
pub async fn update_bangumi_mdb(
    state: web::Data<Arc<AppState>>,
    params: Json<UpdateMDBParams>,
) -> Result<Json<Resp<()>>, ServerError> {
    state
        .metadata
        .update_bangumi_mdb(
            params.bangumi_id,
            params.tmdb_id,
            params.mikan_id,
            params.bangumi_tv_id,
            params.season_number,
            params.kind.clone(),
        )
        .await?;
    Ok(Json(Resp::ok(())))
}

#[get("/api/tmdb/search/{name}")]
pub async fn seach_bangumi_at_tmdb(
    state: web::Data<Arc<AppState>>,
    name: web::Path<String>,
) -> Result<Json<Resp<Vec<TMDBMetadata>>>, ServerError> {
    let name = name.into_inner();
    let tv_shows = state
        .metadata
        .fetcher()
        .seach_bangumi_at_tmdb(&name)
        .await?;
    let mut metadatas = Vec::new();
    for tv_show in tv_shows {
        let mut metadata = TMDBMetadata {
            id: tv_show.inner.id,
            name: tv_show.inner.name,
            poster_image_url: tv_show
                .inner
                .poster_path
                .map(|path| format!("/api/tmdb/image/{}", path.trim_start_matches('/'))),
            air_date: tv_show.inner.first_air_date,
            description: tv_show.inner.overview,
            kind: BgmKind::Anime,
            seasons: Vec::new(),
        };
        for season in tv_show.seasons {
            metadata.seasons.push(TMDBSeason {
                number: season.inner.season_number,
                name: season.inner.name,
                air_date: season.inner.air_date,
                ep_count: season.episode_count,
            });
        }
        metadatas.push(metadata);
    }

    let movies = state.metadata.fetcher().seach_movie_at_tmdb(&name).await?;
    for movie in movies {
        let metadata = TMDBMetadata {
            id: movie.inner.id,
            name: movie.inner.title,
            poster_image_url: movie
                .inner
                .poster_path
                .map(|path| format!("/api/tmdb/image/{}", path.trim_start_matches('/'))),
            air_date: movie.inner.release_date,
            description: Some(movie.inner.overview),
            kind: BgmKind::Movie,
            seasons: Vec::new(),
        };
        metadatas.push(metadata);
    }
    Ok(Json(Resp::ok(metadatas)))
}

#[get("/api/tmdb/image/{path}")]
pub async fn tmdb_image_proxy(
    state: web::Data<Arc<AppState>>,
    path: web::Path<String>,
) -> Result<HttpResponse, ServerError> {
    let path = path.into_inner();
    let response = state
        .metadata
        .fetcher()
        .download_image_from_tmdb_as_response(&path)
        .await?;
    let bytes = response.bytes().await?;
    Ok(HttpResponse::Ok()
        .content_type("image/jpeg")
        .append_header(("Cache-Control", "public, max-age=86400")) // 1天缓存
        .body(bytes))
}

#[get("/api/metrics")]
pub async fn metrics(state: web::Data<Arc<AppState>>) -> Result<Json<Resp<Metrics>>, ServerError> {
    let scheduler_metrics = state.scheduler.metrics().await;
    let downloader_metrics = state.scheduler.get_downloader().metrics().await;

    let mut sys = sysinfo::System::new();
    let pid = sysinfo::Pid::from(std::process::id() as usize);

    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::Some(&[pid]),
        true,
        sysinfo::ProcessRefreshKind::nothing().with_memory(),
    );

    let process = if let Some(process) = sys.process(pid) {
        ProcessMetrics {
            used: process.memory(),
            run_time_sec: process.run_time(),
        }
    } else {
        ProcessMetrics {
            used: 0,
            run_time_sec: 0,
        }
    };

    Ok(Json(Resp::ok(Metrics {
        scheduler: scheduler_metrics,
        downloader: downloader_metrics,
        process,
    })))
}

#[get("/api/config")]
pub async fn get_config(
    state: web::Data<Arc<AppState>>,
) -> Result<Json<Resp<Config>>, ServerError> {
    let config = state.config.read().unwrap();
    Ok(Json(Resp::ok(config.clone())))
}

#[post("/api/config")]
pub async fn update_config(
    state: web::Data<Arc<AppState>>,
    body: Json<Config>,
) -> Result<Json<Resp<()>>, ServerError> {
    let mut config = state.config.write().unwrap();
    *config = body.into_inner();
    state.config_writer.write(&config)?;
    Ok(Json(Resp::ok(())))
}

#[instrument(skip(state))]
#[post("/api/bangumi/list")]
pub async fn list_bangumi(
    state: web::Data<Arc<AppState>>,
    params: Json<QueryBangumiParams>,
) -> Result<Json<Resp<BangumiListResp>>, ServerError> {
    use model::bangumi::Column as BangumiColumn;
    use model::bangumi::Entity as Bangumis;
    use model::subscriptions::Column as SubscriptionColumn;
    use model::subscriptions::Entity as Subscriptions;
    use sea_orm::PaginatorTrait;
    use sea_orm::{
        ColumnTrait, Condition, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect,
    };

    // 构建查询条件
    let mut condition = Condition::all();

    // 添加订阅状态过滤条件
    if let Some(status) = &params.status {
        condition = condition.add(SubscriptionColumn::SubscribeStatus.eq(status.clone()));
    }

    // 添加季度过滤条件
    if let Some(season) = &params.calendar_season {
        condition = condition.add(BangumiColumn::CalendarSeason.eq(season.clone()));
    }

    // 查询总条数
    let total = Bangumis::find()
        .join_rev(
            JoinType::LeftJoin,
            Subscriptions::belongs_to(Bangumis)
                .from(SubscriptionColumn::BangumiId)
                .to(BangumiColumn::Id)
                .into(),
        )
        .filter(condition.clone())
        .count(state.db.conn())
        .await?;

    // 查询分页数据
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
        .column(SubscriptionColumn::EnforceTorrentReleaseAfterBroadcast)
        // 联表查询
        .join_rev(
            JoinType::LeftJoin,
            Subscriptions::belongs_to(Bangumis)
                .from(SubscriptionColumn::BangumiId)
                .to(BangumiColumn::Id)
                .into(),
        )
        // 应用过滤条件
        .filter(condition)
        // 分页
        .offset(params.offset)
        .limit(params.limit)
        // 排序
        .order_by_desc(BangumiColumn::AirDate)
        .into_model::<Bangumi>()
        .all(state.db.conn())
        .await?;

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

    Ok(Json(Resp::ok(BangumiListResp {
        list: bangumis,
        total,
    })))
}
