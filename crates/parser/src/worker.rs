use anyhow::{Context, Result};
use model::sea_orm_active_enums::ParserStatus;
use sea_orm::DatabaseConnection;
use std::{collections::HashSet, sync::Arc};
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info};

use crate::{db::Db, ParseResult, Parser};

const CHANNEL_BUFFER_SIZE: usize = 100;

#[derive(Debug)]
enum WorkerMessage {
    Parse(Vec<String>, oneshot::Sender<Result<Vec<ParseResult>>>),
    Shutdown(oneshot::Sender<()>),
}

#[derive(Clone)]
pub struct Worker {
    db: Db,
    sender: Option<mpsc::Sender<WorkerMessage>>,
    is_spawned: Arc<std::sync::atomic::AtomicBool>,
}

impl Worker {
    pub fn new(db: Db) -> Self {
        Self {
            db,
            sender: None,
            is_spawned: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn new_with_conn(conn: Arc<DatabaseConnection>) -> Self {
        Self::new(Db::new(conn))
    }

    pub async fn new_from_env() -> Result<Self> {
        let db = Db::new_from_env().await?;
        Ok(Self::new(db))
    }

    fn try_set_spawned(&self) -> bool {
        self.is_spawned
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
            )
            .is_ok()
    }

    pub async fn spawn(&mut self, parser: Arc<dyn Parser + Send + Sync>) -> Result<()> {
        if !self.try_set_spawned() {
            return Err(anyhow::anyhow!("Worker already spawned"));
        }

        let (sender, mut receiver) = mpsc::channel(CHANNEL_BUFFER_SIZE);
        self.sender = Some(sender);
        let db = self.db.clone();

        tokio::spawn(async move {
            while let Some(msg) = receiver.recv().await {
                match msg {
                    WorkerMessage::Parse(file_names, response_sender) => {
                        let res = Self::handle_parse_request(&*parser, &db, file_names).await;
                        let _ = response_sender.send(res);
                    }
                    WorkerMessage::Shutdown(done_tx) => {
                        info!("解析器 Worker 收到停机信号");
                        let _ = done_tx.send(());
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    async fn handle_parse_request(
        parser: &(dyn Parser + Send + Sync),
        db: &Db,
        file_names: Vec<String>,
    ) -> Result<Vec<ParseResult>> {
        let mut merged_results = Vec::new();

        // 分批处理，每批次按照解析器支持的最大长度
        for chunk in file_names.chunks(parser.max_file_name_length()) {
            info!("采用 {} 解析文件列表：{:#?}", parser.name(), chunk);

            match parser.parse_file_names(chunk.to_vec()).await {
                Ok(results) => {
                    info!("成功解析 {} 个文件", results.len());
                    db.save_parse_results(&results).await?;
                    merged_results.extend(results);
                }
                Err(e) => {
                    error!("{} 解析文件列表失败: {:?}", parser.name(), e);
                    db.save_parse_errors(chunk, &e.to_string()).await?;
                    return Err(e);
                }
            }
        }

        Ok(merged_results)
    }

    pub async fn parse_file_names(&self, file_names: Vec<String>) -> Result<Vec<ParseResult>> {
        info!("开始处理文件名解析请求，文件数量：{}", file_names.len());

        // 先查询所有记录（包括失败的记录）
        let all_records = self.db.get_all_parse_records(&file_names).await?;

        // 创建已存在文件的快速查找集合（除了 Pending 和 Error 状态的记录）, 意味着会重试
        let mut completed_results = Vec::new();
        let mut completed_file_names = HashSet::new();

        for record in all_records {
            if record.parser_status == ParserStatus::Completed {
                completed_file_names.insert(record.file_name.clone());
                completed_results.extend(Db::record_to_result(&[record]));
            }
        }

        let to_parse_files_names: Vec<String> = file_names
            .into_iter()
            .filter(|f| !completed_file_names.contains(f))
            .collect();

        if to_parse_files_names.is_empty() {
            info!("所有文件均已成功解析，直接返回缓存结果");
            return Ok(completed_results);
        }

        info!("需要解析 {} 个文件", to_parse_files_names.len());

        match self.request_parse_and_await(&to_parse_files_names).await {
            Ok(new_results) => {
                completed_results.extend(new_results);
            }
            Err(e) => {
                error!("解析文件列表失败 只返回成功部分: {:?}", e);
            }
        }

        Ok(completed_results)
    }

    fn filter_unparsed_files(
        &self,
        all_files: &[String],
        parsed_results: &[ParseResult],
    ) -> Vec<String> {
        let parsed_files: HashSet<_> = parsed_results.iter().map(|r| &r.file_name).collect();
        all_files
            .iter()
            .filter(|f| !parsed_files.contains(*f))
            .cloned()
            .collect()
    }

    async fn request_parse_and_await(&self, file_names: &[String]) -> Result<Vec<ParseResult>> {
        let sender = self
            .sender
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Worker not spawned"))?;

        let (response_sender, response_receiver) = oneshot::channel();
        sender
            .send(WorkerMessage::Parse(file_names.to_vec(), response_sender))
            .await
            .context("发送解析请求失败")?;

        response_receiver.await.context("接收解析结果失败")?
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("开始停止解析器 Worker...");
        if let Some(sender) = &self.sender {
            let (done_tx, done_rx) = oneshot::channel();
            sender
                .send(WorkerMessage::Shutdown(done_tx))
                .await
                .context("发送停机信号失败")?;

            // 等待 worker 确认停止
            done_rx.await.context("等待 worker 停止失败")?;

            // 标记为未启动状态
            self.is_spawned
                .store(false, std::sync::atomic::Ordering::SeqCst);

            info!("解析器 Worker 已停止");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_parser_worker() -> Result<()> {
        dotenv::dotenv()?;
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_target(true)
            .init();

        let db = Db::new_from_env().await?;
        let parser = crate::impls::deepbricks::Client::from_env()?;
        let mut worker = Worker::new(db.clone());
        worker.spawn(Arc::new(parser)).await?;

        let file_names = vec![
            "[LoliHouse] 孤nsingle人的异世界攻略 / Hitoribocchi no Isekai Kouryaku - 12 [WebRip 1080p HEVC-10bit AAC][官方简繁内封字幕]".to_string(),
            "[LoliHouse] 孤nsingle人的异世界攻略 / Hitoribocchi no Isekai Kouryaku - 12 [WebRip 1080p HEVC-10bit AAC][简繁双语]".to_string(),
            "[LoliHouse] 孤nsingle人的异世界攻略 / Hitoribocchi no Isekai Kouryaku - 12 [WebRip 1080p HEVC-10bit AAC][简体中文]".to_string(),
            "[LoliHouse] 孤nsingle人的异世界攻略 / Hitoribocchi no Isekai Kouryaku - 12 [WebRip 1080p HEVC-10bit AAC][简日双语]".to_string(),
        ];

        let results = worker.parse_file_names(file_names).await?;
        println!("解析结果: {:#?}", results);
        Ok(())
    }
}
