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

    setup_panic_hook();

    match cli.commands {
        Commands::Start => {
            server::Server::new(config, writer).await?.serve().await?;
        }
    }
    Ok(())
}

fn setup_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("unnamed-thread");

        // 提取Panic的位置和消息
        let location = panic_info.location().unwrap_or_else(|| {
            eprintln!("无法获取panic位置信息");
            std::panic::Location::caller()
        });

        let message = panic_info
            .payload()
            .downcast_ref::<String>()
            .map(|s| s.as_str())
            .or_else(|| panic_info.payload().downcast_ref::<&str>().copied())
            .unwrap_or("<no message>");

        // 获取当前的调用栈
        let backtrace = std::backtrace::Backtrace::capture();

        // 输出详细错误信息
        eprintln!("=== 程序发生严重错误 ===");
        eprintln!(
            "线程 '{}' 在 {}:{} 发生panic: {}",
            thread_name,
            location.file(),
            location.line(),
            message
        );
        eprintln!("调用栈:\n{}", backtrace);

        // 在输出错误信息后结束进程
        eprintln!("程序因严重错误终止运行");
        std::process::exit(1);
    }));
}
