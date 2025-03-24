mod entity;
pub mod migrator;
use entity::sea_orm_active_enums::{BgmKind, Kind, SubscribeStatus};
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
