#![allow(dead_code)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::load_from_file;
use jemallocator::Jemalloc;
use server::server;
use std::path::PathBuf;

mod config;

#[derive(Debug, Subcommand)]
pub enum Commands {
    Start,
}

#[derive(Parser)]
#[command(name = "bangumi")]
#[command(about = "bangumi", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub commands: Commands,

    #[clap(short, long, default_value = "config.toml")]
    pub config: PathBuf,
}

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
pub async fn main() -> Result<()> {
    let cli = Cli::parse();
    let (config, writer) = load_from_file(cli.config)?;

    match cli.commands {
        Commands::Start => {
            server::Server::new(config, writer).await?.serve().await?;
        }
    }
    Ok(())
}
