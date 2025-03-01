use crate::Scheduler;
use anyhow::{Context, Result};
use tracing::{error, info};

impl Scheduler {
    /// 订阅番剧
    #[allow(clippy::too_many_arguments)]
    pub async fn subscribe(
        &self,
        bangumi_id: i32,
        start_episode_number: Option<i32>,
        resolution_filter: Option<Vec<parser::VideoResolution>>,
        language_filter: Option<Vec<parser::Language>>,
        release_group_filter: Option<String>,
        collector_interval: Option<i32>,
        metadata_interval: Option<i32>,
        enforce_torrent_release_after_broadcast: bool,
    ) -> Result<()> {
        // 将分辨率列表转换为逗号分隔的字符串
        let resolution_filter_str = resolution_filter.map(|resolutions| {
            resolutions
                .into_iter()
                .filter(|res| *res != parser::VideoResolution::Unknown)
                .map(|res| res.to_string())
                .collect::<Vec<_>>()
                .join(",")
        });

        // 将语言列表转换为逗号分隔的字符串
        let language_filter_str = language_filter.map(|langs| {
            langs
                .into_iter()
                .filter(|lang| *lang != parser::Language::Unknown)
                .map(|lang| lang.to_string())
                .collect::<Vec<_>>()
                .join(",")
        });

        // 刷新元数据
        self.metadata
            .request_refresh_metadata(bangumi_id, false)
            .await?;

        // 4. 获取所有剧集信息
        let episodes = self.db.get_bangumi_episodes(bangumi_id).await?;
        if episodes.is_empty() {
            return Err(anyhow::anyhow!("未找到剧集信息, 无法订阅"));
        }

        let default_start_episode_number =
            episodes.iter().map(|episode| episode.number).min().unwrap();

        let start_episode = start_episode_number.unwrap_or(default_start_episode_number);

        // 5. 收集需要下载的剧集编号
        let episode_numbers: Vec<i32> = episodes
            .iter()
            .filter(|episode| episode.number >= start_episode)
            .map(|episode| episode.number)
            .collect();

        // 6. 批量创建下载任务
        if !episode_numbers.is_empty() {
            info!(
                "为番剧 {} 批量创建第 {} 到 {} 集的下载任务",
                bangumi_id,
                episode_numbers.first().unwrap(),
                episode_numbers.last().unwrap()
            );
            self.task_manager
                .batch_create_tasks(bangumi_id, episode_numbers)
                .await?;
        }

        self.db
            .upsert_subscription(
                bangumi_id,
                Some(start_episode),
                resolution_filter_str,
                language_filter_str,
                release_group_filter,
                collector_interval,
                metadata_interval,
                enforce_torrent_release_after_broadcast,
            )
            .await
            .context("更新订阅状态失败")?;

        let subscription = self
            .db
            .get_subscription(bangumi_id)
            .await?
            .expect("未找到订阅记录");

        // 7. 创建并启动新的 worker
        self.spawn_new_or_restart_worker(subscription).await?;

        Ok(())
    }

    /// 取消订阅番剧
    pub async fn unsubscribe(&self, bangumi_id: i32) -> Result<()> {
        // 更新订阅状态为未订阅
        if let Err(e) = self.db.unsubscribe(bangumi_id).await {
            error!("更新订阅状态失败: {}", e);
            return Err(e);
        }

        // 停止并移除对应的 worker
        let mut workers = self.workers.lock().await;
        if let Some(worker) = workers.remove(&bangumi_id) {
            if let Err(e) = worker.shutdown().await {
                error!(
                    "停止番剧 {} 的下载任务处理器失败: {}",
                    worker.bangumi.name, e
                );
            } else {
                info!("已停止番剧 {} 的下载任务处理器", worker.bangumi.name);
            }
        }

        Ok(())
    }
}
