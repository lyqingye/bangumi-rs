use std::sync::Arc;

use crate::{
    DownloaderInfo, ThirdPartyDownloader,
    errors::{Error, Result},
};

pub struct Dlrs<'a> {
    downloaders: Vec<&'a dyn ThirdPartyDownloader>,
}

impl<'a> Dlrs<'a> {
    pub fn best(&self) -> &'a dyn ThirdPartyDownloader {
        self.downloaders
            .iter()
            .max_by_key(|d| d.config().priority)
            .map(|d| *d)
            .unwrap()
    }

    pub fn best_except(&self, except: &str) -> Option<&'a dyn ThirdPartyDownloader> {
        self.downloaders
            .iter()
            .filter(|d| !except.contains(d.name()))
            .max_by_key(|d| d.config().priority)
            .map(|d| *d)
    }

    pub fn take(&self, name: &str) -> Option<&'a dyn ThirdPartyDownloader> {
        self.downloaders
            .iter()
            .find(|d| d.name() == name)
            .map(|d| *d)
    }

    pub fn must_take(&self, name: &str) -> Result<&'a dyn ThirdPartyDownloader> {
        self.take(name)
            .ok_or_else(|| Error::DownloaderNotFound(name.to_owned()))
    }

    pub fn info(&self) -> Vec<DownloaderInfo> {
        self.downloaders
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
        Self { downloaders }
    }
}

// 为 Vec 类型实现 From trait，便于使用
impl<'a> From<&'a Vec<Arc<Box<dyn ThirdPartyDownloader>>>> for Dlrs<'a> {
    fn from(downloaders: &'a Vec<Arc<Box<dyn ThirdPartyDownloader>>>) -> Self {
        Self::from(downloaders.as_slice())
    }
}
