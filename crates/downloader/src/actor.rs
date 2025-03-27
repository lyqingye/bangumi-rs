use std::sync::Arc;

use anyhow::{Context as _, Result};
use statig::awaitable::InitializedStateMachine;
use tokio::sync::mpsc;
use tracing::error;

use crate::{
    Store, ThirdPartyDownloader, Tid,
    stm::{Context, Event, TaskStm},
};

pub type Tx = (String, Event);

#[derive(Clone)]
pub struct Actor {
    ch: mpsc::UnboundedSender<Tx>,
    store: Arc<Box<dyn Store>>,
    downloaders: Vec<Arc<Box<dyn ThirdPartyDownloader>>>,
}

impl Actor {
    pub async fn spawn(&mut self) -> Result<()> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.ch = tx;

        {
            let actor = self.clone();
            tokio::spawn(async move {
                actor.run_loop(rx).await;
            });
        }

        Ok(())
    }

    pub async fn run_loop(&self, mut rx: mpsc::UnboundedReceiver<Tx>) {
        let mut stm = TaskStm::new(
            self.store.clone(),
            self.downloaders.clone(),
            &mut Context::uninitialized(),
        )
        .await;

        while let Some(tx) = rx.recv().await {
            match self.execute(&mut stm, tx).await {
                Ok(_) => {}
                Err(e) => {
                    error!("处理事件失败: {}", e);
                }
            }
        }
    }

    async fn execute(&self, stm: &mut InitializedStateMachine<TaskStm>, tx: Tx) -> Result<()> {
        let (info_hash, event) = tx;
        let task = self
            .store
            .list_by_hashes(&[info_hash])
            .await?
            .first()
            .cloned()
            .ok_or(anyhow::anyhow!("任务不存在"))?;

        let tdl = self.take_downloader(&task.downloader)?;
        let mut ctx = Context {
            tid: Tid::from(task.tid()),
            info_hash: &task.info_hash,
            task: &task,
            tdl,
            next_event: None,
        };

        stm.handle_with_context(&event, &mut ctx).await;

        Ok(())
    }
}

impl Actor {
    fn take_downloader(&self, assigned_downloader: &str) -> Result<&dyn ThirdPartyDownloader> {
        let latest = assigned_downloader.split(',').last().unwrap();
        let downloader = &***self
            .downloaders
            .iter()
            .find(|d| d.name() == latest)
            .context(format!("指定的下载器不存在: {}", latest))?;
        Ok(downloader)
    }
}
