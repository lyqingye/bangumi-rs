use crate::{api, server::AppState, ws::ws_handler};
use actix_files::Files;
use actix_web::web;
use std::sync::Arc;

pub const ASSETS_MOUNT_PATH: &str = "/api/assets";

pub fn configure_app(cfg: &mut web::ServiceConfig, state: Arc<AppState>) {
    cfg.app_data(web::Data::new(state.clone()))
        .service(
            Files::new(ASSETS_MOUNT_PATH, state.assets_path.clone())
                .show_files_listing()
                .prefer_utf8(true),
        )
        .service(api::current_calendar_season)
        .service(api::calendar)
        .service(api::get_bangumi_by_id)
        .service(api::get_bangumi_episodes_by_id)
        .service(api::subscribe_bangumi)
        .service(api::get_bangumi_torrents_by_id)
        .service(api::refresh_bangumi)
        .service(api::online_watch)
        .service(api::delete_bangumi_download_tasks)
        .service(api::list_download_tasks)
        .service(api::manual_select_torrent)
        .service(api::refresh_calendar)
        .service(api::seach_bangumi_at_tmdb)
        .service(api::update_bangumi_mdb)
        .service(api::tmdb_image_proxy)
        .service(api::get_config)
        .service(api::update_config)
        .service(api::health)
        .service(api::metrics)
        .service(api::retry_download_task)
        .service(api::list_bangumi)
        .service(api::seach_bangumi_at_mikan)
        .service(api::add_bangumi)
        .service(api::get_version)
        .route("/ws", web::get().to(ws_handler));
}
