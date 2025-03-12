#![deny(clippy::unused_async)]
pub mod api;
pub mod config;
pub mod db;
pub mod error;
mod logger;
pub mod model;
mod router;
pub mod server;
pub mod tracing;
pub mod ws;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
