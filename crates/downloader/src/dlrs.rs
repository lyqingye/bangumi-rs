use std::sync::Arc;

use crate::{
    DownloaderInfo, ThirdPartyDownloader,
    errors::{Error, Result},
};

pub struct Dlrs<'a> {
    inner: Vec<&'a dyn ThirdPartyDownloader>,
}

impl<'a> Dlrs<'a> {
    pub fn best(&self) -> &'a dyn ThirdPartyDownloader {
        self.inner
            .iter()
            .max_by_key(|d| d.config().priority)
            .copied()
            .unwrap()
    }

    pub fn best_unused(&self, used: &str) -> Option<&'a dyn ThirdPartyDownloader> {
        let used_set: std::collections::HashSet<&str> = used.split(',').collect();
        self.inner
            .iter()
            .filter(|d| !used_set.contains(d.name()))
            .max_by_key(|d| d.config().priority)
            .copied()
    }

    pub fn take(&self, name: &str) -> Option<&'a dyn ThirdPartyDownloader> {
        let latest = name.split(',').last().unwrap_or(name);
        self.inner.iter().find(|d| d.name() == latest).copied()
    }

    pub fn must_take(&self, name: &str) -> Result<&'a dyn ThirdPartyDownloader> {
        self.take(name)
            .ok_or_else(|| Error::DownloaderNotFound(name.to_owned()))
    }

    pub fn info(&self) -> Vec<DownloaderInfo> {
        self.inner
            .iter()
            .map(|d| DownloaderInfo {
                name: d.name().to_string(),
                priority: d.config().priority,
            })
            .collect()
    }
}

impl<'a> From<&'a [Arc<Box<dyn ThirdPartyDownloader>>]> for Dlrs<'a> {
    fn from(downloaders: &'a [Arc<Box<dyn ThirdPartyDownloader>>]) -> Self {
        let downloaders = downloaders.iter().map(|d| &***d).collect::<Vec<_>>();
        Self { inner: downloaders }
    }
}

impl<'a> From<&'a Vec<Arc<Box<dyn ThirdPartyDownloader>>>> for Dlrs<'a> {
    fn from(downloaders: &'a Vec<Arc<Box<dyn ThirdPartyDownloader>>>) -> Self {
        Self::from(downloaders.as_slice())
    }
}
