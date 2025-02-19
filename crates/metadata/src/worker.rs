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

use crate::{db::Db, fetcher::Fetcher};
use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use tracing::{error, info, warn};

const REFRESH_COOLDOWN: i64 = 1; // minutes
const CHANNEL_CAPACITY: usize = 100;
const POLL_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum RefreshKind {
    Metadata,
    Torrents,
    Calendar,
}

#[derive(Debug, Clone)]
pub struct RefreshRequest {
    pub bangumi_id: Option<i32>,
    pub kind: RefreshKind,
    pub timestamp: NaiveDateTime,
}

impl RefreshRequest {
    fn new(bangumi_id: Option<i32>, kind: RefreshKind) -> Self {
        Self {
            bangumi_id,
            kind,
            timestamp: chrono::Local::now().naive_utc(),
        }
    }

    fn key(&self) -> RefreshKey {
        RefreshKey {
            bangumi_id: self.bangumi_id,
            kind: self.kind.clone(),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
struct RefreshKey {
    bangumi_id: Option<i32>,
    kind: RefreshKind,
}

#[derive(Debug)]
enum WorkerMessage {
    Refresh(RefreshRequest),
    Shutdown(oneshot::Sender<()>),
}

#[derive(Clone)]
pub struct Worker {
    db: Db,
    mikan: mikan::client::Client,
    client: reqwest::Client,
    fetcher: Fetcher,
    sender: Option<mpsc::Sender<WorkerMessage>>,
    is_spawned: Arc<AtomicBool>,
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
            is_spawned: Arc::new(AtomicBool::new(false)),
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
        if !self
            .is_spawned
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            return Err(anyhow::anyhow!("Worker 已经启动"));
        }

        let (sender, mut receiver) = mpsc::channel(CHANNEL_CAPACITY);
        self.sender = Some(sender);

        let worker = self.clone();
        let is_spawned = self.is_spawned.clone();

        tokio::spawn(async move {
            let refresh_times: Arc<Mutex<HashMap<RefreshKey, NaiveDateTime>>> =
                Arc::new(Mutex::new(HashMap::new()));

            while let Some(msg) = receiver.recv().await {
                match msg {
                    WorkerMessage::Refresh(request) => {
                        let key = request.key();
                        if worker.should_process_request(&key, &refresh_times).await {
                            worker.handle_refresh_request(request).await;
                        }
                    }
                    WorkerMessage::Shutdown(done_tx) => {
                        info!("元数据 Worker 收到停机信号");
                        let _ = done_tx.send(());
                        break;
                    }
                }
            }
            is_spawned.store(false, Ordering::SeqCst);
            info!("元数据 Worker 已停止");
        });

        Ok(())
    }

    pub async fn request_refresh(&self, bangumi_id: Option<i32>, kind: RefreshKind) -> Result<()> {
        let sender = self.sender.as_ref().context("Worker 未启动")?;

        sender
            .send(WorkerMessage::Refresh(RefreshRequest::new(
                bangumi_id, kind,
            )))
            .await
            .context("发送刷新请求失败")?;

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("开始停止元数据 Worker...");

        if let Some(sender) = &self.sender {
            let (done_tx, done_rx) = oneshot::channel();
            sender
                .send(WorkerMessage::Shutdown(done_tx))
                .await
                .context("发送停机信号失败")?;

            // 等待 worker 确认停止
            done_rx.await.context("等待 worker 停止失败")?;

            info!("元数据 Worker 已停止");
        }
        Ok(())
    }

    async fn should_process_request(
        &self,
        key: &RefreshKey,
        refresh_times: &Arc<Mutex<HashMap<RefreshKey, NaiveDateTime>>>,
    ) -> bool {
        let now = chrono::Local::now().naive_utc();
        let mut times = refresh_times.lock().await;
        if let Some(last_time) = times.get(key) {
            if now.signed_duration_since(*last_time).num_minutes() < REFRESH_COOLDOWN {
                return false;
            }
        }
        times.insert(key.clone(), now);
        true
    }

    async fn handle_refresh_request(&self, request: RefreshRequest) {
        if let Err(e) = self.process_refresh_request(request).await {
            error!("处理刷新请求失败: {}", e);
        }
    }

    async fn process_refresh_request(&self, request: RefreshRequest) -> Result<()> {
        match (request.bangumi_id, request.kind) {
            (Some(id), RefreshKind::Torrents) => {
                let bgm = self.db.get_bangumi_by_id(id).await?.context("番剧未找到")?;
                info!("正在刷新番剧种子信息: {}", bgm.name);
                self.collect_torrents(&bgm).await?;
            }
            (Some(id), RefreshKind::Metadata) => {
                let mut bgm = self.db.get_bangumi_by_id(id).await?.context("番剧未找到")?;
                info!("正在刷新番剧元数据: {}", bgm.name);
                self.fetcher.fill_bangumi_metadata(&mut bgm).await?;
                let episodes = self.fetcher.collect_episodes(&bgm).await?;
                self.db.save_bangumi_tv_episodes(&bgm, episodes).await?;
                self.db.update_bangumi(bgm).await?;
            }
            (None, RefreshKind::Calendar) => {
                self.refresh_calendar().await?;
            }
            _ => warn!("无效的刷新请求: {:?}", request),
        }
        Ok(())
    }

    pub async fn refresh_calendar(&self) -> Result<()> {
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
            self.request_refresh(Some(bgm.id), RefreshKind::Metadata)
                .await?;
        }
        Ok(())
    }

    pub async fn collect_torrents(&self, bgm: &bangumi::Model) -> Result<()> {
        let torrents = self.fetcher.collect_torrents(bgm).await?;

        if torrents.is_empty() {
            info!("未找到番剧 {} 的种子信息", bgm.name);
            return Ok(());
        }

        info!("已收集 {} 个番剧 {} 的种子信息", torrents.len(), bgm.name);

        self.db.save_mikan_torrents(bgm.id, torrents).await?;
        Ok(())
    }

    pub async fn update_bangumi_mdb(
        &self,
        bgm_id: i32,
        tmdb_id: Option<u64>,
        mikan_id: Option<i32>,
        banugmi_tv_id: Option<i32>,
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
        self.db.update_bangumi(bgm).await?;

        self.request_refresh(Some(bgm_id), RefreshKind::Metadata)
            .await?;
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
        worker
            .request_refresh(Some(91), RefreshKind::Torrents)
            .await?;
        tokio::time::sleep(Duration::from_secs(30)).await;
        worker.shutdown().await?;
        Ok(())
    }
}
