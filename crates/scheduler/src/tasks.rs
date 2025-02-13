use anyhow::Result;
use model::episode_download_tasks;
use model::sea_orm_active_enums::State;
use sea_orm::Set;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::db::Db;

/// 任务缓存管理器
#[derive(Clone)]
pub struct TaskManager {
    db: Db,
    // 使用 RwLock 实现读写锁，优化读操作性能
    // 外层 HashMap 的 key 是 bangumi_id
    // 内层 HashMap 的 key 是 episode_number
    tasks: Arc<RwLock<HashMap<i32, HashMap<i32, episode_download_tasks::Model>>>>,
}

impl TaskManager {
    pub fn new(db: Db) -> Self {
        Self {
            db,
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 初始化指定番剧的任务缓存
    pub async fn init_bangumi_tasks(&self, bangumi_id: i32) -> Result<()> {
        // 从数据库加载任务
        let tasks = self.db.get_unfinished_tasks_by_bangumi(bangumi_id).await?;

        // 更新缓存
        let mut cache = self.tasks.write().await;
        let bangumi_tasks = cache.entry(bangumi_id).or_default();

        for task in tasks {
            bangumi_tasks.insert(task.episode_number, task);
        }

        Ok(())
    }

    /// 清除指定番剧的任务缓存
    pub async fn clear_bangumi_tasks(&self, bangumi_id: i32) {
        let mut cache = self.tasks.write().await;
        cache.remove(&bangumi_id);
    }

    /// 获取指定番剧的所有未完成任务
    pub async fn get_unfinished_tasks(
        &self,
        bangumi_id: i32,
    ) -> Result<Vec<episode_download_tasks::Model>> {
        // 先尝试从缓存读取
        let cache = self.tasks.read().await;
        if let Some(bangumi_tasks) = cache.get(&bangumi_id) {
            return Ok(bangumi_tasks.values().cloned().collect());
        }
        drop(cache);

        // 缓存未命中，从数据库加载并更新缓存
        self.init_bangumi_tasks(bangumi_id).await?;

        // 再次从缓存读取
        let cache = self.tasks.read().await;
        Ok(cache
            .get(&bangumi_id)
            .map(|tasks| tasks.values().cloned().collect())
            .unwrap_or_default())
    }

    /// 更新任务状态
    pub async fn update_task_state(
        &self,
        bangumi_id: i32,
        episode_number: i32,
        state: State,
    ) -> Result<()> {
        // 先更新数据库
        self.db
            .update_task_state(bangumi_id, episode_number, state.clone())
            .await?;

        if state == State::Downloaded {
            // 删除缓存
            let mut cache = self.tasks.write().await;
            if let Some(bangumi_tasks) = cache.get_mut(&bangumi_id) {
                bangumi_tasks.remove(&episode_number);
            }
        } else {
            // 更新缓存
            let mut cache = self.tasks.write().await;
            if let Some(bangumi_tasks) = cache.get_mut(&bangumi_id) {
                if let Some(task) = bangumi_tasks.get_mut(&episode_number) {
                    task.state = state;
                }
            }
        }

        Ok(())
    }

    /// 更新任务状态为就绪，并设置选中的种子
    pub async fn update_task_ready(
        &self,
        bangumi_id: i32,
        episode_number: i32,
        info_hash: &str,
    ) -> Result<()> {
        // 先更新数据库
        self.db
            .update_task_ready(bangumi_id, episode_number, info_hash)
            .await?;

        // 更新缓存
        let mut cache = self.tasks.write().await;
        if let Some(bangumi_tasks) = cache.get_mut(&bangumi_id) {
            if let Some(task) = bangumi_tasks.get_mut(&episode_number) {
                task.state = State::Ready;
                task.ref_torrent_info_hash = Some(info_hash.to_string());
            }
        }

        Ok(())
    }

    /// 批量创建下载任务
    pub async fn batch_create_tasks(
        &self,
        bangumi_id: i32,
        episode_numbers: Vec<i32>,
    ) -> Result<()> {
        use model::episode_download_tasks::ActiveModel;

        // 构造批量任务
        let tasks: Vec<ActiveModel> = episode_numbers
            .into_iter()
            .map(|episode_number| ActiveModel {
                bangumi_id: Set(bangumi_id),
                episode_number: Set(episode_number),
                state: Set(State::Missing),
                ..Default::default()
            })
            .collect();

        // 批量插入数据库
        self.db.batch_create_tasks(tasks).await?;

        // 获取新创建的任务
        let new_tasks = self.db.get_unfinished_tasks_by_bangumi(bangumi_id).await?;

        // 更新缓存
        let mut cache = self.tasks.write().await;
        let bangumi_tasks = cache.entry(bangumi_id).or_default();

        for task in new_tasks {
            bangumi_tasks.insert(task.episode_number, task);
        }

        Ok(())
    }
}
