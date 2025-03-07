use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use sea_orm::DatabaseConnection;
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{error, info};

use dict::DictCode;
use model::sea_orm_active_enums::BgmKind;

use crate::{
    db::Db, fetcher::Fetcher, matcher::Matcher, mdb_bgmtv::MdbBgmTV, mdb_mikan::MdbMikan,
    mdb_tmdb::MdbTmdb, MetadataAttr, MetadataAttrSet, MetadataDb,
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
}

impl Worker {
    pub fn new(
        db: Db,
        client: reqwest::Client,
        mikan: mikan::client::Client,
        fetcher: Fetcher,
        dict: dict::Dict,
        assets_path: String,
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
        }
    }

    pub fn new_with_conn(
        conn: Arc<DatabaseConnection>,
        client: reqwest::Client,
        mikan: mikan::client::Client,
        fetcher: Fetcher,
        dict: dict::Dict,
        assets_path: String,
    ) -> Result<Self> {
        let db = Db::new(conn);
        Ok(Self::new(db, client, mikan, fetcher, dict, assets_path))
    }

    pub async fn new_from_env() -> Result<Self> {
        let db = Db::new_from_env().await?;
        let assets_path = std::env::var("ASSETS_PATH")?;
        Ok(Self::new(
            db,
            reqwest::Client::new(),
            mikan::client::Client::from_env()?,
            Fetcher::new_from_env()?,
            dict::Dict::new_from_env().await?,
            assets_path,
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

    pub async fn spawn(&mut self) -> Result<()> {
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
    pub async fn request_refresh_metadata(&self, bangumi_id: i32, force: bool) -> Result<()> {
        self.send_cmd(Cmd::Refresh(Inner::Metadata(bangumi_id, force)), None)
            .await
    }

    pub async fn request_refresh_torrents(&self, bangumi_id: i32) -> Result<()> {
        self.send_cmd(Cmd::Refresh(Inner::Torrents(bangumi_id)), None)
            .await
    }

    pub async fn request_refresh_torrents_and_wait(&self, bangumi_id: i32) -> Result<()> {
        let (done_tx, done_rx) = oneshot::channel();
        self.send_cmd(Cmd::Refresh(Inner::Torrents(bangumi_id)), Some(done_tx))
            .await?;
        tokio::time::timeout(Duration::from_secs(60), done_rx)
            .await
            .context("等待种子刷新超时")??;
        Ok(())
    }

    pub async fn request_refresh_calendar(
        &self,
        season: Option<String>,
        force: bool,
    ) -> Result<()> {
        self.send_cmd(Cmd::Refresh(Inner::Calendar(season, force)), None)
            .await
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
        )
        .await?;
        tokio::time::timeout(Duration::from_secs(60), done_rx)
            .await
            .context("等待添加番剧超时")??;
        Ok(())
    }

    async fn send_cmd(&self, cmd: Cmd, done_tx: Option<oneshot::Sender<()>>) -> Result<()> {
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
                self.handle_refresh_calendar(season, force).await?;
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
            match mdbs
                .mikan
                .update_bangumi_metadata(
                    &mut bgm,
                    MetadataAttrSet(vec![MetadataAttr::Poster]),
                    false,
                )
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    error!("使用mikan填充封面失败: {}", e);
                }
            }
        }

        // 收集剧集列表
        let episodes = self.fetcher.collect_episodes(&bgm).await?;
        self.db.save_bangumi_tv_episodes(&bgm, episodes).await?;
        self.db.update_bangumi(bgm).await?;

        info!("番剧 {} 元数据刷新完成", name);
        Ok(())
    }

    /// 处理放送列表刷新请求
    async fn handle_refresh_calendar(&self, season: Option<String>, force: bool) -> Result<()> {
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
        for bgm in bangumis {
            let bgm_id = bgm.id;

            match self.try_match_bangumi(bgm).await {
                Ok(_) => {}
                Err(e) => {
                    error!("匹配番剧失败: {}", e);
                }
            };

            self.request_refresh_metadata(bgm_id, force).await?;
        }
        info!("放送列表刷新完成");
        Ok(())
    }

    async fn try_match_bangumi(&self, bgm: model::bangumi::Model) -> Result<()> {
        let mut bgm = bgm;
        if bgm.bangumi_tv_id.is_none() {
            self.matcher.match_bgm_tv(&mut bgm, false).await?;
            self.db.update_bangumi(bgm.clone()).await?;
        } else if bgm.tmdb_id.is_none() || bgm.bgm_kind.is_none() || bgm.season_number.is_none() {
            self.matcher.match_tmdb(&mut bgm).await?;
            self.db.update_bangumi(bgm.clone()).await?;
        }
        if bgm.tmdb_id.is_none() || bgm.bgm_kind.is_none() {
            self.matcher.match_tmdb_movie(&mut bgm).await?;
            self.db.update_bangumi(bgm).await?;
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

        let torrents = self.fetcher.collect_torrents(&bgm).await?;

        if torrents.is_empty() {
            info!("未找到番剧 {} 的种子信息", bgm.name);
            return Ok(());
        }

        info!("已收集 {} 个番剧 {} 的种子信息", torrents.len(), bgm.name);

        self.db.save_mikan_torrents(bgm.id, &torrents).await?;

        // 获取torrents中最新的种子对应的发布时间
        let latest_torrent = torrents.into_iter().max_by_key(|t| t.pub_date);
        if let Some(torrent) = latest_torrent {
            if let Some(pub_date) = torrent.pub_date {
                self.db.update_bangumi_update_time(bgm.id, pub_date).await?;
            }
        }
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
    #[ignore]
    async fn test_refresh_calendar() -> Result<()> {
        dotenv::dotenv()?;
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_target(true)
            .init();

        let mut worker = Worker::new_from_env().await?;
        worker.spawn().await?;
        worker.request_refresh_torrents_and_wait(20).await?;
        // tokio::time::sleep(Duration::from_secs(120)).await;
        worker.shutdown().await?;
        Ok(())
    }
}
