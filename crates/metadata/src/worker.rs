use dict::DictCode;
use model::{bangumi, sea_orm_active_enums::SubscribeStatus};
use sea_orm::DatabaseConnection;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::sync::{mpsc, oneshot, Mutex};

use crate::{db::Db, fetcher::Fetcher, MetadataDb};
use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use tracing::{error, info, warn};

const REFRESH_COOLDOWN: i64 = 1; // minutes
const CHANNEL_CAPACITY: usize = 100;
const POLL_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Inner {
    Metadata(i32, bool),
    Torrents(i32),
    Calendar,
}

#[derive(Debug)]
enum Cmd {
    Refresh(Inner),
    Shutdown(oneshot::Sender<()>),
}

#[derive(Clone)]
pub struct Worker {
    db: Db,
    mikan: mikan::client::Client,
    client: reqwest::Client,
    fetcher: Fetcher,
    sender: Option<mpsc::Sender<Cmd>>,
    dict: dict::Dict,
}

impl Worker {
    pub fn new(
        db: Db,
        client: reqwest::Client,
        mikan: mikan::client::Client,
        fetcher: Fetcher,
        dict: dict::Dict,
    ) -> Self {
        Self {
            db,
            mikan,
            client,
            fetcher,
            sender: None,
            dict,
        }
    }

    pub fn new_with_conn(
        conn: Arc<DatabaseConnection>,
        client: reqwest::Client,
        mikan: mikan::client::Client,
        fetcher: Fetcher,
        dict: dict::Dict,
    ) -> Result<Self> {
        let db = Db::new(conn);
        Ok(Self::new(db, client, mikan, fetcher, dict))
    }

    pub async fn new_from_env() -> Result<Self> {
        let db = Db::new_from_env().await?;

        Ok(Self::new(
            db,
            reqwest::Client::new(),
            mikan::client::Client::from_env()?,
            Fetcher::new_from_env()?,
            dict::Dict::new_from_env().await?,
        ))
    }

    pub async fn spawn(&mut self) -> Result<()> {
        if self.sender.is_some() {
            return Err(anyhow::anyhow!("Worker 已经启动"));
        }

        let (sender, mut receiver) = mpsc::channel(CHANNEL_CAPACITY);
        self.sender = Some(sender);

        let worker = self.clone();

        tokio::spawn(async move {
            let refresh_times: Arc<Mutex<HashMap<Inner, NaiveDateTime>>> =
                Arc::new(Mutex::new(HashMap::new()));

            while let Some(msg) = receiver.recv().await {
                match msg {
                    Cmd::Refresh(inner) => {
                        if !worker.should_process(&inner, &refresh_times).await {
                            continue;
                        }
                        match worker.process(inner).await {
                            Ok(_) => {}
                            Err(e) => error!("处理刷新请求失败: {}", e),
                        }
                    }
                    Cmd::Shutdown(done_tx) => {
                        info!("元数据 Worker 收到停机信号");
                        let _ = done_tx.send(());
                        break;
                    }
                }
            }
            info!("元数据 Worker 已停止");
        });

        Ok(())
    }

    /// 检查是否应该处理请求, 规则为: 一定时间段内只处理一次
    async fn should_process(
        &self,
        req: &Inner,
        refresh_times: &Arc<Mutex<HashMap<Inner, NaiveDateTime>>>,
    ) -> bool {
        let now = chrono::Local::now().naive_utc();
        let mut times = refresh_times.lock().await;
        if let Some(last_time) = times.get(req) {
            if now.signed_duration_since(*last_time).num_minutes() < REFRESH_COOLDOWN {
                return false;
            }
        }
        times.insert(req.clone(), now);
        true
    }

    /// 快捷命令, 外部使用
    pub async fn request_refresh_metadata(&self, bangumi_id: i32, force: bool) -> Result<()> {
        self.send_cmd(Cmd::Refresh(Inner::Metadata(bangumi_id, force)))
            .await
    }

    pub async fn request_refresh_torrents(&self, bangumi_id: i32) -> Result<()> {
        self.send_cmd(Cmd::Refresh(Inner::Torrents(bangumi_id)))
            .await
    }

    pub async fn request_refresh_calendar(&self) -> Result<()> {
        self.send_cmd(Cmd::Refresh(Inner::Calendar)).await
    }

    async fn send_cmd(&self, cmd: Cmd) -> Result<()> {
        let sender = self.sender.as_ref().context("Worker 未启动")?;

        sender.send(cmd).await.context("发送刷新请求失败")?;

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("开始停止元数据 Worker...");

        if let Some(sender) = &self.sender {
            let (done_tx, done_rx) = oneshot::channel();
            sender.send(Cmd::Shutdown(done_tx)).await?;
            done_rx.await?;
        }
        info!("元数据 Worker 已停止");
        Ok(())
    }

    async fn process(&self, request: Inner) -> Result<()> {
        match request {
            Inner::Torrents(id) => {
                self.handle_collect_torrents(id).await?;
            }
            Inner::Metadata(id, force) => {
                self.handle_refresh_metadata(id, force).await?;
            }
            Inner::Calendar => {
                self.handle_refresh_calendar().await?;
            }
            _ => warn!("无效的刷新请求: {:?}", request),
        }
        Ok(())
    }
}

/// Handlers
impl Worker {
    /// 处理元数据刷新请求
    pub async fn handle_refresh_metadata(&self, bangmui_id: i32, force: bool) -> Result<()> {
        let mut bgm = self
            .db
            .get_bangumi_by_id(bangmui_id)
            .await?
            .context("番剧未找到")?;
        info!("正在刷新番剧元数据: {}", bgm.name);

        // 填充番剧元数据 TODO

        // 收集剧集列表
        let episodes = self.fetcher.collect_episodes(&bgm).await?;
        self.db.save_bangumi_tv_episodes(&bgm, episodes).await?;
        self.db.update_bangumi(bgm).await?;
        Ok(())
    }

    /// 处理放送列表刷新请求
    pub async fn handle_refresh_calendar(&self) -> Result<()> {
        info!("正在刷新放送列表");
        let calendar = self.mikan.get_calendar().await?;
        info!(
            "已收集 {} 个番剧信息, 放送季: {:?}",
            calendar.bangumis.len(),
            calendar.season
        );

        self.dict
            .set_value(
                DictCode::CurrentSeasonSchedule,
                calendar.season.clone().unwrap_or_default(),
            )
            .await?;

        let mikan_ids: Vec<_> = calendar.bangumis.iter().map(|bgm| bgm.id).collect();
        self.db.save_mikan_calendar(calendar).await?;

        let bangumis = self.db.list_bangumi_by_mikan_ids(mikan_ids).await?;
        for bgm in bangumis {
            self.request_refresh_metadata(bgm.id, false).await?;
        }
        Ok(())
    }

    /// 处理番剧种子信息收集请求
    pub async fn handle_collect_torrents(&self, bangumi_id: i32) -> Result<()> {
        let bgm = self
            .db
            .get_bangumi_by_id(bangumi_id)
            .await?
            .context("番剧未找到")?;

        info!("正在收集番剧 {} 的种子信息", bgm.name);

        let torrents = self.fetcher.collect_torrents(&bgm).await?;

        if torrents.is_empty() {
            info!("未找到番剧 {} 的种子信息", bgm.name);
            return Ok(());
        }

        info!("已收集 {} 个番剧 {} 的种子信息", torrents.len(), bgm.name);

        self.db.save_mikan_torrents(bgm.id, torrents).await?;
        Ok(())
    }
}

impl Worker {
    /// 更新番剧MDB信息
    pub async fn update_bangumi_mdb(
        &self,
        bgm_id: i32,
        tmdb_id: Option<u64>,
        mikan_id: Option<i32>,
        banugmi_tv_id: Option<i32>,
        season_number: Option<u64>,
    ) -> Result<()> {
        let mut bgm = self
            .db
            .get_bangumi_by_id(bgm_id)
            .await?
            .context("番剧未找到")?;
        info!(
            "正在更新番剧MDB {} TMDB ID: {:?}, mikan ID: {:?}, bgm.tv ID: {:?}",
            bgm.name, tmdb_id, mikan_id, banugmi_tv_id
        );
        if mikan_id.is_some() {
            bgm.mikan_id = mikan_id;
        }
        if banugmi_tv_id.is_some() {
            bgm.bangumi_tv_id = banugmi_tv_id;
        }
        if tmdb_id.is_some() {
            bgm.tmdb_id = tmdb_id;
        }
        if season_number.is_some() {
            bgm.season_number = season_number;
        }
        self.db.update_bangumi(bgm).await?;

        self.request_refresh_metadata(bgm_id, true).await?;
        Ok(())
    }

    pub fn fetcher(&self) -> &Fetcher {
        &self.fetcher
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_refresh_calendar() -> Result<()> {
        dotenv::dotenv()?;
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_target(true)
            .init();

        let mut worker = Worker::new_from_env().await?;
        worker.spawn().await?;
        worker.request_refresh_torrents(91).await?;
        tokio::time::sleep(Duration::from_secs(30)).await;
        worker.shutdown().await?;
        Ok(())
    }
}
