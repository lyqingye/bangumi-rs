mod entity;
pub mod migrator;
use std::fmt::{self, Display};

use entity::sea_orm_active_enums::{BgmKind, Kind, ResourceType, SubscribeStatus};
pub use entity::*;

impl From<String> for Kind {
    fn from(s: String) -> Self {
        match s.as_str() {
            "OVA" => Self::Ep,
            "TV" => Self::Op,
            "MOVIE" => Self::Ed,
            "MAD" => Self::Mad,
            "SP" => Self::Sp,
            _ => Self::Other,
        }
    }
}

impl From<String> for SubscribeStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Subscribed" => Self::Subscribed,
            "Downloaded" => Self::Downloaded,
            _ => Self::None,
        }
    }
}

impl TryFrom<String> for BgmKind {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "anime" => Ok(Self::Anime),
            "movie" => Ok(Self::Movie),
            _ => Err(format!("invalid bgm kind: {}", s)),
        }
    }
}

impl torrent_download_tasks::Model {
    pub fn tid(&self) -> &str {
        self.tid.as_deref().unwrap_or(self.info_hash.as_str())
    }
}

impl Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceType::Torrent => write!(f, "Torrent"),
            ResourceType::Magnet => write!(f, "Magnet"),
            ResourceType::TorrentURL => write!(f, "TorrentURL"),
            ResourceType::InfoHash => write!(f, "InfoHash"),
        }
    }
}
