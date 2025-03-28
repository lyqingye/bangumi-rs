use bytes::Bytes;
use lazy_static::lazy_static;
use model::sea_orm_active_enums::ResourceType;

use crate::errors::{Error, Result};

#[derive(Debug, Clone)]
pub enum Resource {
    // 磁力链接,InfoHash
    MagnetLink(String, String),
    // InfoHash
    MagnetInfoHash(String),
    // 种子文件字节,InfoHash
    TorrentFileBytes(Bytes, String),
    // 种子下载URL,InfoHash
    TorrentURL(String, String),
}

impl Resource {
    pub fn from_info_hash<T: Into<String>>(info_hash: T) -> Result<Self> {
        let info_hash = info_hash.into();
        if info_hash.len() != 40 || !info_hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(Error::InfoHashFormat(info_hash));
        }
        Ok(Resource::MagnetInfoHash(info_hash))
    }

    pub fn from_magnet_link<T: Into<String>>(magnet_link: T) -> Result<Self> {
        let magnet_link = magnet_link.into();
        // 使用正则表达式提取 InfoHash
        lazy_static! {
            static ref RE: regex::Regex =
                regex::Regex::new(r"magnet:\?xt=urn:btih:([0-9a-fA-F]{40})(&|$)").unwrap();
        }

        if let Some(caps) = RE.captures(&magnet_link) {
            if let Some(info_hash) = caps.get(1) {
                let info_hash = info_hash.as_str();
                return Ok(Resource::MagnetLink(
                    magnet_link.clone(),
                    info_hash.to_string(),
                ));
            }
        }
        Err(Error::MagnetFormat(magnet_link))
    }

    pub fn from_torrent_url<T: Into<String>>(torrent_url: T, info_hash: T) -> Result<Self> {
        let torrent_url = torrent_url.into();
        let info_hash = info_hash.into();
        Ok(Resource::TorrentURL(torrent_url, info_hash))
    }

    pub fn from_torrent_file_bytes<T: Into<Bytes>>(torrent_file_bytes: T) -> Result<Self> {
        let torrent_file_bytes = torrent_file_bytes.into();
        let torrent = torrent::Torrent::from_bytes(&torrent_file_bytes)?;
        Ok(Resource::TorrentFileBytes(
            torrent_file_bytes,
            torrent.info_hash_str()?,
        ))
    }

    pub fn get_type(&self) -> ResourceType {
        match self {
            Resource::MagnetLink(_, _) => ResourceType::Magnet,
            Resource::MagnetInfoHash(_) => ResourceType::InfoHash,
            Resource::TorrentFileBytes(_, _) => ResourceType::Torrent,
            Resource::TorrentURL(_, _) => ResourceType::TorrentURL,
        }
    }

    pub fn magnet(&self) -> Option<String> {
        match self {
            Resource::MagnetLink(magnet, _) => Some(magnet.clone()),
            Resource::MagnetInfoHash(_)
            | Resource::TorrentFileBytes(_, _)
            | Resource::TorrentURL(_, _) => {
                Some(format!("magnet:?xt=urn:btih:{}", self.info_hash()))
            }
        }
    }

    pub fn info_hash(&self) -> &str {
        match self {
            Resource::MagnetInfoHash(hash) => hash,
            Resource::TorrentFileBytes(_, hash) => hash,
            Resource::MagnetLink(_, hash) => hash,
            Resource::TorrentURL(_, hash) => hash,
        }
    }

    pub fn torrent_url(&self) -> Option<String> {
        match self {
            Resource::TorrentURL(url, _) => Some(url.clone()),
            _ => None,
        }
    }
}
