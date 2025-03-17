use actix_cors::Cors;
use actix_web::{App, HttpServer};
use core::panic;
use dict::DictCode;
use metadata::providers::mikan::MikanProvider;
use parser::Parser;
use reqwest::Url;
use std::borrow::Cow;
use std::sync::{Arc, RwLock};
use std::{net::SocketAddr, path::PathBuf, str::FromStr};
use tokio::sync::broadcast;
use tracing::{error, info};
use tracing_actix_web::TracingLogger;

use crate::config::{Config, Writer};
use crate::logger::{init_logger, LogMessage};
use crate::router;
use anyhow::Result;
use downloader::ThirdPartyDownloader;
use mikan::client::Client;
use sea_orm_migration::MigratorTrait;
#[derive(Clone)]
pub struct AppState {
    pub scheduler: scheduler::Scheduler,
    pub metadata: metadata::worker::Worker,
    pub log_tx: broadcast::Sender<LogMessage>,
    pub assets_path: PathBuf,
    pub db: crate::db::Db,
    pub dict: dict::Dict,
    pub config_writer: Arc<Box<dyn Writer>>,
    pub config: Arc<RwLock<Config>>,
    pub sentry_guard: Arc<Option<sentry::ClientInitGuard>>,
}

pub struct Server {
    config: Config,
    state: Arc<AppState>,
}

impl Server {
    pub async fn new(config: Config, config_writer: Box<dyn Writer>) -> Result<Self> {
        config.validate()?;
        let state = Self::init_state(&config, config_writer).await?;
        Self::after_init(&state).await?;
        Ok(Self { config, state })
    }

    async fn init_state(config: &Config, config_writer: Box<dyn Writer>) -> Result<Arc<AppState>> {
        // sentry
        let sentry_guard = if config.sentry.enabled {
            let git_verison = crate::built_info::GIT_VERSION.unwrap_or("unknown");
            let commit = crate::built_info::GIT_COMMIT_HASH_SHORT.unwrap_or("unknown");

            let release = format!("{}-{}", git_verison, commit);

            Some(sentry::init((
                config.sentry.dsn.as_str(),
                sentry::ClientOptions {
                    release: Some(Cow::Owned(release)),
                    environment: Some(Cow::Borrowed("development")),
                    traces_sample_rate: 1.0,
                    ..Default::default()
                },
            )))
        } else {
            None
        };

        // Logger
        let log_tx = init_logger(config)?;

        info!("Rust 版本: {}", crate::built_info::RUSTC_VERSION);
        info!(
            "Git 版本: {}",
            crate::built_info::GIT_VERSION.unwrap_or("unknown")
        );
        info!(
            "Git 提交: {}",
            crate::built_info::GIT_COMMIT_HASH.unwrap_or("unknown")
        );
        info!("构建时间: {}", crate::built_info::BUILT_TIME_UTC);

        // Database
        let db = crate::db::Db::new(&config.server.database_url).await?;

        // Execute migrations
        model::migrator::Migrator::up(db.conn(), None).await?;

        // HTTP Client
        let client = if config.proxy.enabled {
            reqwest::Client::builder()
                .proxy(reqwest::Proxy::http(&config.proxy.http)?)
                .proxy(reqwest::Proxy::https(&config.proxy.https)?)
                .build()?
        } else {
            reqwest::Client::new()
        };

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
            &config.bangumi_tv.image_base_url,
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

        // Torrent Providers
        let mikan_provider = MikanProvider::new(mikan.clone());

        // Metadata Worker
        let mut metadata_worker = metadata::worker::Worker::new_with_conn(
            db.conn_pool(),
            client.clone(),
            mikan.clone(),
            metadata::fetcher::Fetcher::new(tmdb, bgm_tv, mikan.clone(), client.clone()),
            dict.clone(),
            config.server.assets_path.clone(),
            Arc::new(vec![Box::new(mikan_provider)]),
        )?;
        metadata_worker.spawn()?;

        // Parser worker
        let parser_impl = Self::create_parser(config, client.clone());
        let mut parser_worker = parser::worker::Worker::new_with_conn(db.conn_pool());
        parser_worker.spawn(parser_impl).await?;

        // Downloader worker
        let downloader = if config.downloader.pan115.enabled {
            let mut pan115 = pan_115::client::Client::new(
                config.downloader.pan115.cookies.as_str(),
                Some(pan_115::client::RateLimitConfig {
                    max_requests_per_second: config.downloader.pan115.max_requests_per_second,
                }),
            )?;
            pan115.login_check().await?;
            Box::new(
                downloader::thirdparty::pan_115_impl::Pan115DownloaderImpl::new(
                    pan115,
                    downloader::thirdparty::pan_115_impl::Config {
                        generic: config.downloader.pan115.generic.to_downloader_config(),
                        ..Default::default()
                    },
                ),
            ) as Box<dyn ThirdPartyDownloader>
        } else if config.downloader.qbittorrent.enabled {
            let qbittorrent = qbittorrent::client::Client::new(
                client.clone(),
                Url::parse(&config.downloader.qbittorrent.url)?,
                config.downloader.qbittorrent.username.as_str(),
                config.downloader.qbittorrent.password.as_str(),
            );
            qbittorrent.login(false).await?;
            Box::new(
                downloader::thirdparty::qbittorrent_impl::QbittorrentDownloaderImpl::new(
                    qbittorrent,
                    downloader::thirdparty::qbittorrent_impl::Config {
                        generic: config.downloader.qbittorrent.generic.to_downloader_config(),
                    },
                ),
            ) as Box<dyn ThirdPartyDownloader>
        } else {
            panic!("没有启用任何下载器");
        };

        let dl_store = downloader::db::Db::new(db.conn_pool());
        let mut downloader_worker = downloader::worker::Worker::new_with_conn(
            Box::new(dl_store),
            downloader,
            downloader::config::Config::default(),
        )?;

        downloader_worker.spawn().await?;

        // Scheduler
        let mut scheduler = scheduler::Scheduler::new_with_conn(
            db.conn_pool(),
            parser_worker,
            metadata_worker.clone(),
            Arc::new(Box::new(downloader_worker)),
            notify_worker,
            client.clone(),
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
            config_writer: Arc::new(config_writer),
            config: Arc::new(RwLock::new(config.clone())),
            sentry_guard: Arc::new(sentry_guard),
        }))
    }

    fn create_parser(config: &Config, client: reqwest::Client) -> Arc<dyn Parser + Send + Sync> {
        let parser_impl: Arc<dyn Parser + Send + Sync>;
        if config.parser.siliconflow.enabled {
            parser_impl = Arc::new(parser::impls::siliconflow::Client::new(
                parser::impls::siliconflow::Config {
                    api_key: config.parser.siliconflow.api_key.clone(),
                    base_url: config.parser.siliconflow.base_url.clone(),
                    model: config.parser.siliconflow.model.clone(),
                    ..Default::default()
                },
                client,
            ));
        } else if config.parser.deepseek.enabled {
            parser_impl = Arc::new(parser::impls::deepseek::Client::new(
                parser::impls::deepseek::Config {
                    api_key: config.parser.deepseek.api_key.clone(),
                    base_url: config.parser.deepseek.base_url.clone(),
                    model: config.parser.deepseek.model.clone(),
                    ..Default::default()
                },
                client,
            ));
        } else if config.parser.deepbricks.enabled {
            parser_impl = Arc::new(parser::impls::deepbricks::Client::new(
                parser::impls::deepbricks::Config {
                    api_key: config.parser.deepbricks.api_key.clone(),
                    base_url: config.parser.deepbricks.base_url.clone(),
                    model: config.parser.deepbricks.model.clone(),
                },
                client,
            ));
        } else if config.parser.raw.enabled {
            parser_impl = Arc::new(parser::impls::raw::Raw::new());
        } else {
            panic!("No parser enabled");
        }
        parser_impl
    }

    pub async fn serve(&self) -> Result<()> {
        let addr = self.config.server.listen_addr.parse::<SocketAddr>()?;
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
                .configure(|cfg| router::configure_app(cfg, state.clone()))
        })
        .bind(addr)?
        .run();

        let server_handle = server.handle();
        let server_task = tokio::spawn(server);

        info!("服务监听: http://{}", addr);

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

    async fn after_init(state: &Arc<AppState>) -> Result<()> {
        let first_run = state.dict.get_value_as::<bool>(DictCode::FirstRun).await?;

        if first_run.is_none() || first_run.unwrap() {
            info!("检测到首次运行，开始执行初始化...");
            Self::do_first_run(state)?;
            state
                .dict
                .set_value_as::<bool>(DictCode::FirstRun, &false)
                .await?;
        }
        Ok(())
    }

    fn do_first_run(state: &Arc<AppState>) -> Result<()> {
        state.metadata.request_refresh_calendar(None, true)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{Config, NopWriter};
    use crate::server::Server;
    use anyhow::Result;

    #[tokio::test]
    #[ignore]
    async fn test_server() -> Result<()> {
        let server = Server::new(Config::default(), Box::new(NopWriter)).await?;
        server.serve().await
    }
}
