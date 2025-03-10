use crate::TorrentProvider;
use anyhow::Result;
use async_trait::async_trait;
use model::{bangumi, torrents};
use tracing::warn;

pub struct MikanProvider {
    pub mikan: mikan::client::Client,
}

#[async_trait]
impl TorrentProvider for MikanProvider {
    async fn search_torrents(&self, bgm: &bangumi::Model) -> Result<Vec<torrents::Model>> {
        match bgm.mikan_id {
            Some(id) => Ok(self
                .mikan
                .collect_by_bangumi_id(id)
                .await?
                .iter()
                .filter_map(|t| {
                    t.pub_date.map(|pub_date| torrents::Model {
                        bangumi_id: bgm.id,
                        title: t.file_name.clone().unwrap_or_default(),
                        size: t.file_size as i64,
                        info_hash: t.info_hash.clone(),
                        magnet: t.magnet_link.clone(),
                        data: None,
                        download_url: t.torrent_download_url.as_ref().map(|url| url.to_string()),
                        pub_date,
                        // TODO source
                    })
                })
                .collect()),
            None => {
                warn!("番剧 {} 缺少 mikan_id", bgm.name);
                Ok(vec![])
            }
        }
    }
}
