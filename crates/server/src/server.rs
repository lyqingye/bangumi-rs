use actix_cors::Cors;
use actix_files::Files;
use actix_web::{web, App, HttpServer};
use parser::Parser;
use std::sync::Arc;
use std::{net::SocketAddr, path::PathBuf, str::FromStr};
use tokio::sync::broadcast;
use tracing::{error, info};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Layer};

use crate::ws::ws_handler;
use crate::{api, config::Config};
use anyhow::Result;
use mikan::client::Client;

#[derive(Clone)]
pub struct LogMessage {
    pub content: String,
}

#[derive(Clone)]
pub struct AppState {
    pub scheduler: scheduler::Scheduler,
    pub metadata: metadata::worker::Worker,
    pub log_tx: broadcast::Sender<LogMessage>,
    pub assets_path: PathBuf,
    pub db: crate::db::Db,
    pub dict: dict::Dict,
}

pub struct Server {
    config: Config,
    state: Arc<AppState>,
}

pub const ASSETS_MOUNT_PATH: &'static str = "/assets";

impl Server {
    pub async fn new(config: Config) -> Result<Self> {
        let state = Self::init_state(&config).await?;
        Ok(Self { config, state })
    }

    async fn init_state(config: &Config) -> Result<Arc<AppState>> {
        // Logger
        let log_tx = Self::init_logger(&config)?;

        // Database
        let db = crate::db::Db::new(&config.server.database_url).await?;

        // HTTP Client
        let client = reqwest::Client::new();

        // Mikan
        let mikan = Client::new_with_client(client.clone(), &config.mikan.endpoint)?;

        // TMDB
        let tmdb = tmdb::client::Client::new(
            client.clone(),
            config.tmdb.api_key.as_str(),
            config.tmdb.base_url.as_str(),
            config.tmdb.image_base_url.as_str(),
            config.tmdb.language.as_str(),
        )?;

        // BangumiTV
        let bgm_tv = bangumi_tv::client::Client::new_with_client(
            client.clone(),
            &config.bangumi_tv.endpoint,
        )?;

        let dict = dict::Dict::new(db.conn_pool());

        // Notify Worker
        let mut notify_worker = notify::worker::Worker::new();
        if config.notify.telegram.enabled {
            // Telegram notifier
            notify_worker.add_notifier(Box::new(
                notify::telegram::TelegramNotifier::new_with_client(
                    client.clone(),
                    &config.notify.telegram.token,
                    &config.notify.telegram.chat_id,
                )?,
            ));
        }
        notify_worker.spawn().await?;

        // Metadata Worker
        let mut metadata_worker = metadata::worker::Worker::new_with_conn(
            db.conn_pool(),
            client.clone(),
            mikan.clone(),
            metadata::fetcher::Fetcher::new(
                tmdb,
                bgm_tv,
                mikan.clone(),
                config.server.assets_path.clone(),
                client.clone(),
            ),
            dict.clone(),
        )?;
        metadata_worker.spawn().await?;

        // Parser worker
        let parser_impl = Self::create_parser(&config, client.clone());
        let mut parser_worker = parser::worker::Worker::new_with_conn(db.conn_pool());
        parser_worker.spawn(Arc::new(parser_impl)).await?;

        // Downloader worker
        let mut pan115 = pan_115::client::Client::new(
            config.pan115.cookies.as_str(),
            Some(pan_115::client::RateLimitConfig {
                max_requests_per_second: config.pan115.max_requests_per_second,
            }),
        )?;
        pan115.login_check().await?;

        let mut downloader_worker = downloader::pan_115_dl::Pan115Downloader::new_with_conn(
            db.conn_pool(),
            pan115,
            downloader::pan_115_dl::Config {
                download_dir: PathBuf::from_str(&config.pan115.download_dir)?,
                ..Default::default()
            },
        )
        .await?;
        downloader_worker.spawn().await?;

        // Scheduler
        let scheduler = scheduler::Scheduler::new_with_conn(
            db.conn_pool(),
            parser_worker,
            metadata_worker.clone(),
            Arc::new(Box::new(downloader_worker)),
            notify_worker,
        );
        scheduler.spawn().await?;

        // Assets Path
        let assets_path = PathBuf::from_str(&config.server.assets_path)?;

        Ok(Arc::new(AppState {
            scheduler,
            metadata: metadata_worker,
            log_tx,
            assets_path,
            db,
            dict,
        }))
    }

    fn create_parser(config: &Config, client: reqwest::Client) -> Box<dyn Parser + Send + Sync> {
        let parser_impl: Box<dyn Parser + Send + Sync>;
        if config.parser.siliconflow.enabled {
            parser_impl = Box::new(parser::impls::siliconflow::Client::new(
                parser::impls::siliconflow::Config {
                    api_key: config.parser.siliconflow.api_key.clone(),
                    base_url: config.parser.siliconflow.base_url.clone(),
                    model: config.parser.siliconflow.model.clone(),
                    ..Default::default()
                },
                client,
            ));
        } else if config.parser.deepseek.enabled {
            parser_impl = Box::new(parser::impls::deepseek::Client::new(
                parser::impls::deepseek::Config {
                    api_key: config.parser.deepseek.api_key.clone(),
                    base_url: config.parser.deepseek.base_url.clone(),
                    model: config.parser.deepseek.model.clone(),
                    ..Default::default()
                },
                client,
            ));
        } else if config.parser.deepbricks.enabled {
            parser_impl = Box::new(parser::impls::deepbricks::Client::new(
                parser::impls::deepbricks::Config {
                    api_key: config.parser.deepbricks.api_key.clone(),
                    base_url: config.parser.deepbricks.base_url.clone(),
                    model: config.parser.deepbricks.model.clone(),
                    ..Default::default()
                },
                client,
            ));
        } else {
            panic!("No parser enabled");
        }
        parser_impl
    }

    fn init_logger(config: &Config) -> Result<broadcast::Sender<LogMessage>> {
        let (log_tx, _) = broadcast::channel(4096);
        let log_filter = tracing::level_filters::LevelFilter::from_str(&config.log.level)?;

        let env_filter = EnvFilter::builder()
            .with_default_directive(log_filter.into())
            .parse("")?
            .add_directive("server::api=debug".parse()?)
            .add_directive("server=debug".parse()?)
            .add_directive("scheduler=debug".parse()?)
            .add_directive("parser=debug".parse()?)
            .add_directive("metadata=debug".parse()?)
            .add_directive("downloader=debug".parse()?)
            .add_directive("sea_orm=debug".parse()?)
            .add_directive("sqlx=warn".parse()?)
            .add_directive("notify=debug".parse()?)
            .add_directive("actix_web=debug".parse()?)
            .add_directive("actix_server=debug".parse()?)
            .add_directive("tracing_actix_web::middleware=debug".parse()?)
            .add_directive("tracing_actix_web=debug".parse()?);

        // let broadcast_layer = BroadcastLayer::new(log_tx.clone(), log_filter);
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(false)
            .with_level(true)
            .with_target(true)
            .with_filter(env_filter)
            .with_filter(log_filter);

        #[cfg(feature = "tokio_console")]
        {
            let subscriber = tracing_subscriber::registry()
                .with(console_subscriber::spawn())
                .with(fmt_layer);
            tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
        }

        #[cfg(not(feature = "tokio_console"))]
        {
            let subscriber = tracing_subscriber::registry().with(fmt_layer);
            tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
        }

        Ok(log_tx)
    }

    pub async fn serve(&self) -> Result<()> {
        let addr = self.config.server.listen_addr.parse::<SocketAddr>()?;
        info!("server listen at: http://{}", addr);

        let state = self.state.clone();

        // 创建 HTTP 服务器
        let server = HttpServer::new(move || {
            let cors = Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .max_age(3600);
            App::new()
                .wrap(TracingLogger::default())
                .wrap(cors)
                .configure(|cfg| Self::configure_app(cfg, state.clone()))
        })
        .bind(addr)?
        .run();

        let server_handle = server.handle();
        let server_task = tokio::spawn(server);

        // 等待中断信号
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("收到中断信号，开始优雅停机...");

                // 1. 停止接受新的连接
                info!("停止接受新的连接...");
                server_handle.stop(true).await;

                // 3. 等待所有现有连接处理完成
                match server_task.await {
                    Ok(_) => info!("HTTP 服务器已完全停止"),
                    Err(e) => error!("HTTP 服务器停止时发生错误: {}", e),
                }

                // 2. 停止调度器和其他组件
                info!("停止调度器和其他组件...");
                if let Err(e) = self.state.scheduler.shutdown().await {
                    error!("停止调度器时发生错误: {}", e);
                }

                info!("服务器优雅停机完成");
            }
            Err(err) => error!("无法监听中断信号: {}", err),
        }

        Ok(())
    }

    fn configure_app(cfg: &mut web::ServiceConfig, state: Arc<AppState>) {
        cfg.app_data(web::Data::new(state.clone()))
            .service(
                Files::new("/assets", state.assets_path.clone())
                    .show_files_listing()
                    .prefer_utf8(true),
            )
            .service(api::calendar)
            .service(api::get_bangumi_by_id)
            .service(api::get_bangumi_episodes_by_id)
            .service(api::subscribe_bangumi)
            .service(api::get_bangumi_torrents_by_id)
            .service(api::refresh_bangumi)
            .service(api::online_watch)
            .service(api::delete_bangumi_download_tasks)
            .service(api::list_download_tasks)
            .service(api::health)
            .route("/ws", web::get().to(ws_handler));
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::server::Server;
    use anyhow::Result;

    #[tokio::test]
    async fn test_server() -> Result<()> {
        let server = Server::new(Config::default()).await?;
        server.serve().await
    }
}
