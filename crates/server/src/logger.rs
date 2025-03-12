use std::str::FromStr;

use crate::config::Config;
use anyhow::Result;
use tokio::sync::broadcast;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::{EnvFilter, Layer};
#[derive(Clone)]
pub struct LogMessage {
    pub content: String,
}

pub fn init_logger(config: &Config) -> Result<broadcast::Sender<LogMessage>> {
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
        use crate::tracing::BroadcastLayer;
        use std::borrow::Cow;
        let broadcast_layer = BroadcastLayer::new(log_tx.clone(), log_filter);

        if config.sentry.enabled {
            let git_verison = crate::built_info::GIT_VERSION.unwrap_or("unknown");
            let commit = crate::built_info::GIT_COMMIT_HASH_SHORT.unwrap_or("unknown");

            let release = format!("{}-{}", git_verison, commit);
            let _guard = sentry::init((
                config.sentry.dsn.as_str(),
                sentry::ClientOptions {
                    release: Some(Cow::Owned(release)),
                    ..Default::default()
                },
            ));

            let sentry_layer = sentry_tracing::layer().event_filter(|md| match md.level() {
                &tracing::Level::ERROR => sentry_tracing::EventFilter::Event,
                _ => sentry_tracing::EventFilter::Ignore,
            });

            let subscriber = tracing_subscriber::registry()
                .with(fmt_layer)
                .with(broadcast_layer)
                .with(sentry_layer);
            tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
        } else {
            let subscriber = tracing_subscriber::registry()
                .with(fmt_layer)
                .with(broadcast_layer);
            tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
        }
    }

    Ok(log_tx)
}
