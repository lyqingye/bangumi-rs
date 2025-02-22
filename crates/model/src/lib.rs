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

impl From<String> for BgmKind {
    fn from(s: String) -> Self {
        match s.as_str() {
            "anime" => Self::Anime,
            "movie" => Self::Movie,
            _ => unreachable!(),
        }
    }
}
