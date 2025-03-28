use std::sync::Arc;

use crate::{
    DownloaderInfo, ThirdPartyDownloader,
    errors::{Error, Result},
};

pub fn assigned_dlr(dlr: &str) -> &str {
    dlr.split(',').last().unwrap_or(dlr)
}

pub fn assign_dlr(dlr: &str, new_dlr: &str) -> String {
    format!("{},{}", dlr, new_dlr)
}

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
        let latest = assigned_dlr(name);
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
    fn from(dlrs: &'a [Arc<Box<dyn ThirdPartyDownloader>>]) -> Self {
        let dlrs = dlrs.iter().map(|d| &***d).collect::<Vec<_>>();
        Self { inner: dlrs }
    }
}

impl<'a> From<&'a Vec<Arc<Box<dyn ThirdPartyDownloader>>>> for Dlrs<'a> {
    fn from(dlrs: &'a Vec<Arc<Box<dyn ThirdPartyDownloader>>>) -> Self {
        Self::from(dlrs.as_slice())
    }
}
