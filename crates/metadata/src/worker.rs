use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use sea_orm::DatabaseConnection;
use tokio::sync::{Mutex, RwLock, mpsc, oneshot};
use tracing::{error, info};

use dict::DictCode;
use model::sea_orm_active_enums::BgmKind;

use crate::{
    MetadataAttr, MetadataAttrSet, MetadataDb, TorrentProvider, db::Db, fetcher::Fetcher,
    matcher::Matcher, mdb_bgmtv::MdbBgmTV, mdb_mikan::MdbMikan, mdb_tmdb::MdbTmdb, metrics,
    providers::mikan::MikanProvider,
};

const REFRESH_COOLDOWN: i64 = 1; // minutes

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Inner {
    Metadata(i32, bool),
    Torrents(i32),
    /// 刷新指定季节的放送列表, 如果为None, 则刷新当前季节
    Calendar(Option<String>, bool),
}

#[derive(Debug)]
enum Cmd {
    Refresh(Inner),
    AddBangumi(String, i32, Option<i32>, Option<u64>),
    Shutdown(),
}

struct Metadatabases {
    bgmtv: MdbBgmTV,
    mikan: MdbMikan,
    tmdb: MdbTmdb,
}

#[derive(Clone)]
pub struct Worker {
    db: Db,
    mikan: mikan::client::Client,
    client: reqwest::Client,
    fetcher: Fetcher,
    sender: Option<mpsc::UnboundedSender<(Cmd, Option<oneshot::Sender<()>>)>>,
    dict: dict::Dict,
    matcher: Matcher,
    assets_path: String,
    metrics: Arc<RwLock<metrics::Metrics>>,
    providers: Arc<Vec<Box<dyn TorrentProvider>>>,
}

impl Worker {
    pub fn new(
        db: Db,
        client: reqwest::Client,
        mikan: mikan::client::Client,
        fetcher: Fetcher,
        dict: dict::Dict,
        assets_path: String,
        providers: Arc<Vec<Box<dyn TorrentProvider>>>,
    ) -> Self {
        let matcher = Matcher::new(fetcher.tmdb.clone(), fetcher.bgm_tv.clone(), mikan.clone());
        Self {
            db,
            mikan,
            client,
            fetcher,
            sender: None,
            dict,
            matcher,
            assets_path,
            metrics: Arc::new(RwLock::new(metrics::Metrics::default())),
            providers,
        }
    }

    pub fn new_with_conn(
        conn: Arc<DatabaseConnection>,
        client: reqwest::Client,
        mikan: mikan::client::Client,
        fetcher: Fetcher,
        dict: dict::Dict,
        assets_path: String,
        providers: Arc<Vec<Box<dyn TorrentProvider>>>,
    ) -> Result<Self> {
        let db = Db::new(conn);
        Ok(Self::new(
            db,
            client,
            mikan,
            fetcher,
            dict,
            assets_path,
            providers,
        ))
    }

    pub async fn new_from_env() -> Result<Self> {
        let db = Db::new_from_env().await?;
        let assets_path = std::env::var("ASSETS_PATH")?;
        let mikan = mikan::client::Client::from_env()?;
        let mikan_provider = MikanProvider::new(mikan.clone());
        Ok(Self::new(
            db,
            reqwest::Client::new(),
            mikan,
            Fetcher::new_from_env()?,
            dict::Dict::new_from_env().await?,
            assets_path,
            Arc::new(vec![Box::new(mikan_provider)]),
        ))
    }

    fn new_mdbs(&self) -> Arc<Metadatabases> {
        Arc::new(Metadatabases {
            bgmtv: MdbBgmTV {
                bgm_tv: self.fetcher.bgm_tv.clone(),
                assets_path: self.assets_path.clone(),
            },
            mikan: MdbMikan {
                mikan: self.mikan.clone(),
                client: self.client.clone(),
                assets_path: self.assets_path.clone(),
            },
            tmdb: MdbTmdb {
                tmdb: self.fetcher.tmdb.clone(),
                assets_path: self.assets_path.clone(),
            },
        })
    }

    pub fn spawn(&mut self) -> Result<()> {
        if self.sender.is_some() {
            return Err(anyhow::anyhow!("Worker 已经启动"));
        }

        let (sender, mut receiver) = mpsc::unbounded_channel();
        self.sender = Some(sender);

        let worker = self.clone();

        tokio::spawn(async move {
            let refresh_times: Arc<Mutex<HashMap<Inner, NaiveDateTime>>> =
                Arc::new(Mutex::new(HashMap::new()));

            let mdbs = worker.new_mdbs();

            while let Some((msg, done_tx)) = receiver.recv().await {
                let exit = match msg {
                    Cmd::Refresh(inner) => {
                        if worker.should_process(&inner, &refresh_times).await {
                            match worker.process(inner, &mdbs).await {
                                Ok(_) => {}
                                Err(e) => error!("处理刷新请求失败: {}", e),
                            }
                        }
                        false
                    }
                    Cmd::AddBangumi(title, mikan_id, bgm_tv_id, tmdb_id) => {
                        match worker
                            .handle_add_bangumi(title, mikan_id, bgm_tv_id, tmdb_id)
                            .await
                        {
                            Ok(_) => {}
                            Err(e) => error!("处理添加番剧请求失败: {}", e),
                        };
                        false
                    }
                    Cmd::Shutdown() => {
                        info!("元数据 Worker 收到停机信号");
                        true
                    }
                };
                if let Some(done_tx) = done_tx {
                    let _ = done_tx.send(());
                }
                if exit {
                    break;
                }
            }
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
    pub fn request_refresh_metadata(&self, bangumi_id: i32, force: bool) -> Result<()> {
        self.send_cmd(Cmd::Refresh(Inner::Metadata(bangumi_id, force)), None)
    }

    pub fn request_refresh_torrents(&self, bangumi_id: i32) -> Result<()> {
        self.send_cmd(Cmd::Refresh(Inner::Torrents(bangumi_id)), None)
    }

    pub async fn request_refresh_torrents_and_wait(&self, bangumi_id: i32) -> Result<()> {
        let (done_tx, done_rx) = oneshot::channel();
        self.send_cmd(Cmd::Refresh(Inner::Torrents(bangumi_id)), Some(done_tx))?;
        tokio::time::timeout(Duration::from_secs(60), done_rx)
            .await
            .context("等待种子刷新超时")??;
        Ok(())
    }

    pub fn request_refresh_calendar(&self, season: Option<String>, force: bool) -> Result<()> {
        self.send_cmd(Cmd::Refresh(Inner::Calendar(season, force)), None)
    }

    pub async fn request_add_bangumi(
        &self,
        title: String,
        mikan_id: i32,
        bgm_tv_id: Option<i32>,
        tmdb_id: Option<u64>,
    ) -> Result<()> {
        let (done_tx, done_rx) = oneshot::channel();
        self.send_cmd(
            Cmd::AddBangumi(title, mikan_id, bgm_tv_id, tmdb_id),
            Some(done_tx),
        )?;
        tokio::time::timeout(Duration::from_secs(60), done_rx)
            .await
            .context("等待添加番剧超时")??;
        Ok(())
    }

    fn send_cmd(&self, cmd: Cmd, done_tx: Option<oneshot::Sender<()>>) -> Result<()> {
        let sender = self.sender.as_ref().context("Worker 未启动")?;

        sender.send((cmd, done_tx)).context("发送刷新请求失败")?;

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("开始停止元数据 Worker...");

        if let Some(sender) = &self.sender {
            let (done_tx, done_rx) = oneshot::channel();
            sender.send((Cmd::Shutdown(), Some(done_tx)))?;
            done_rx.await?;
        }
        info!("元数据 Worker 已停止");
        Ok(())
    }

    async fn process(&self, request: Inner, mdbs: &Arc<Metadatabases>) -> Result<()> {
        match request {
            Inner::Torrents(id) => {
                self.handle_collect_torrents(id).await?;
            }
            Inner::Metadata(id, force) => {
                self.handle_refresh_metadata(id, force, mdbs).await?;
            }
            Inner::Calendar(season, force) => {
                self.handle_refresh_calendar(season, force, mdbs).await?;
            }
        }
        Ok(())
    }
}

/// Handlers
impl Worker {
    /// 处理元数据刷新请求
    async fn handle_refresh_metadata(
        &self,
        bangumi_id: i32,
        force: bool,
        mdbs: &Arc<Metadatabases>,
    ) -> Result<()> {
        let mut bgm = self
            .db
            .get_bangumi_by_id(bangumi_id)
            .await?
            .context("番剧未找到")?;
        let name = bgm.name.clone();
        info!("正在刷新番剧元数据: {}", name);

        if force {
            self.try_match_bangumi(&mut bgm).await?;
        }

        // NOTE: 这里需要考虑外部服务被重复访问

        // 2. 使用Tmdb填充绝大部分信息
        match mdbs
            .tmdb
            .update_bangumi_metadata(&mut bgm, mdbs.tmdb.supports(), force)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                error!("使用tmdb填充元数据失败: {}", e);
            }
        }

        // 3. 使用bgm.tv作为Fallback, 如果TMDB 无法提供封面，那么则使用bgm.tv提供
        let mut attrs = mdbs.bgmtv.supports();
        if bgm.poster_image_url.is_some() {
            attrs.remove(MetadataAttr::Poster);
        }
        match mdbs
            .bgmtv
            .update_bangumi_metadata(&mut bgm, attrs, force)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                error!("使用bgm.tv填充元数据失败: {}", e);
            }
        }

        // 4. 如果前两者都无法提供封面，那么则使用Mikan提供
        if bgm.poster_image_url.is_none() {
            if let Err(e) = mdbs
                .mikan
                .update_bangumi_metadata(
                    &mut bgm,
                    MetadataAttrSet(vec![MetadataAttr::Poster]),
                    false,
                )
                .await
            {
                error!("使用mikan填充封面失败: {}", e);
            }
        }

        // 收集剧集列表
        let episodes = self.fetcher.collect_episodes(&bgm).await?;

        // 剧集开始集数
        bgm.ep_start_number = episodes
            .data
            .iter()
            .filter(|e| e.get_ep().is_some())
            .min_by_key(|e| e.get_ep().unwrap())
            .map(|e| e.get_ep().unwrap())
            .unwrap_or(1);

        self.db.save_bangumi_tv_episodes(&bgm, episodes).await?;
        self.db.update_bangumi(bgm).await?;

        info!("番剧 {} 元数据刷新完成", name);
        Ok(())
    }

    /// 处理放送列表刷新请求
    async fn handle_refresh_calendar(
        &self,
        season: Option<String>,
        force: bool,
        mdbs: &Arc<Metadatabases>,
    ) -> Result<()> {
        info!("正在刷新放送列表: {:?}", season);

        let calendar = if let Some(season) = season.as_ref() {
            self.mikan.get_calendar_by_season(season).await?
        } else {
            self.mikan.get_calendar().await?
        };
        info!(
            "已收集 {} 个番剧信息, 放送季: {:?}",
            calendar.bangumis.len(),
            calendar.season
        );

        // 更新当前放送季
        if season.is_none() {
            self.dict
                .set_value(
                    DictCode::CurrentSeasonSchedule,
                    calendar.season.clone().unwrap_or_default(),
                )
                .await?;
        }

        let mikan_ids: Vec<_> = calendar.bangumis.iter().map(|bgm| bgm.id).collect();
        self.db.save_mikan_calendar(calendar).await?;

        info!("正在匹配番剧: {}", mikan_ids.len());
        let bangumis = self.db.list_bangumi_by_mikan_ids(mikan_ids).await?;
        for mut bgm in bangumis {
            let bgm_id = bgm.id;

            self.try_match_bangumi(&mut bgm)
                .await
                .inspect_err(|e| error!("匹配番剧失败: {}", e))
                .ok();

            self.handle_refresh_metadata(bgm_id, force, mdbs)
                .await
                .inspect_err(|e| error!("刷新番剧元数据失败: {}", e))
                .ok();
        }
        info!("放送列表刷新完成");
        Ok(())
    }

    async fn try_match_bangumi(&self, bgm: &mut model::bangumi::Model) -> Result<()> {
        if bgm.bangumi_tv_id.is_none() || bgm.air_date.is_none() {
            self.matcher.match_bgm_tv(bgm).await?;
            self.db.update_bangumi(bgm.clone()).await?;
        }
        if bgm.tmdb_id.is_none() || bgm.bgm_kind.is_none() || bgm.season_number.is_none() {
            self.matcher.match_tmdb(bgm).await?;
            self.db.update_bangumi(bgm.clone()).await?;
        }
        if bgm.tmdb_id.is_none() || bgm.bgm_kind.is_none() {
            self.matcher.match_tmdb_movie(bgm).await?;
            self.db.update_bangumi(bgm.clone()).await?;
        }
        Ok(())
    }

    /// 处理番剧种子信息收集请求
    async fn handle_collect_torrents(&self, bangumi_id: i32) -> Result<()> {
        let bgm = self
            .db
            .get_bangumi_by_id(bangumi_id)
            .await?
            .context("番剧未找到")?;

        info!("正在收集番剧 {} 的种子信息", bgm.name);

        let mut torrents = Vec::new();

        for provider in self.providers.iter() {
            match provider.search_torrents(&bgm).await {
                Ok(result) => torrents.extend(result),
                Err(e) => {
                    error!("[{}] 收集种子信息失败: {}", provider.name(), e);
                }
            }
        }

        torrents.dedup_by_key(|t| t.info_hash.clone());

        if torrents.is_empty() {
            info!("未找到番剧 {} 的种子信息", bgm.name);
            return Ok(());
        }

        info!("已收集 {} 个番剧 {} 的种子信息", torrents.len(), bgm.name);

        // 获取torrents中最新的种子对应的发布时间
        let latest_torrent = torrents.iter().max_by_key(|t| t.pub_date);
        if let Some(torrent) = latest_torrent {
            self.db
                .update_bangumi_update_time(bgm.id, torrent.pub_date)
                .await?;
        }

        self.db.batch_upsert_torrent(torrents).await?;

        Ok(())
    }

    async fn handle_add_bangumi(
        &self,
        title: String,
        mikan_id: i32,
        bgm_tv_id: Option<i32>,
        tmdb_id: Option<u64>,
    ) -> Result<()> {
        if self.db.get_bangumi_by_mikan_id(mikan_id).await?.is_some() {
            return Ok(());
        }

        self.db
            .add_bangumi(title, mikan_id, bgm_tv_id, tmdb_id)
            .await?;
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
        kind: BgmKind,
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
        bgm.bgm_kind = Some(kind);
        self.db.update_bangumi(bgm).await?;

        self.request_refresh_metadata(bgm_id, true)?;
        Ok(())
    }

    pub fn fetcher(&self) -> &Fetcher {
        &self.fetcher
    }

    pub async fn metrics(&self) -> metrics::Metrics {
        let now = chrono::Local::now().timestamp();

        // 使用读锁检查是否需要刷新
        {
            let guard = self.metrics.read().await;
            if now <= guard.last_refresh_time {
                return guard.clone();
            }
        }

        // 需要刷新时才获取写锁
        let mut guard = self.metrics.write().await;

        // 双重检查，避免多个线程同时刷新
        if now <= guard.last_refresh_time {
            return guard.clone();
        }

        // 清空旧的服务状态
        guard.services.clear();

        // 并行检查所有服务
        let bgm_tv_fut = self.fetcher.bgm_tv.get_subject(1);
        let mikan_fut = self.mikan.get_bangumi_info(3333);
        let tmdb_fut = self.fetcher.tmdb.get_movie(822119);

        let (bgm_tv_result, mikan_result, tmdb_result) =
            tokio::join!(bgm_tv_fut, mikan_fut, tmdb_fut);

        // 添加服务状态
        guard.services.push(metrics::Service {
            name: "bgm.tv".to_string(),
            status: match bgm_tv_result {
                Ok(_) => metrics::ServiceStatus {
                    success: true,
                    error: None,
                },
                Err(e) => metrics::ServiceStatus {
                    success: false,
                    error: Some(e.to_string()),
                },
            },
        });

        guard.services.push(metrics::Service {
            name: "mikan".to_string(),
            status: match mikan_result {
                Ok(_) => metrics::ServiceStatus {
                    success: true,
                    error: None,
                },
                Err(e) => metrics::ServiceStatus {
                    success: false,
                    error: Some(e.to_string()),
                },
            },
        });

        guard.services.push(metrics::Service {
            name: "tmdb".to_string(),
            status: match tmdb_result {
                Ok(_) => metrics::ServiceStatus {
                    success: true,
                    error: None,
                },
                Err(e) => metrics::ServiceStatus {
                    success: false,
                    error: Some(e.to_string()),
                },
            },
        });

        // 更新刷新时间
        guard.last_refresh_time = now + Duration::from_secs(60).as_secs() as i64;

        guard.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_refresh_calendar() -> Result<()> {
        dotenv::dotenv()?;
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_target(true)
            .init();

        let mut worker = Worker::new_from_env().await?;
        worker.spawn()?;
        worker.request_refresh_torrents_and_wait(20).await?;
        // tokio::time::sleep(Duration::from_secs(120)).await;
        worker.shutdown().await?;
        Ok(())
    }
}
