use anyhow::Result;
use model::sea_orm_active_enums::ResourceType;

#[derive(Debug, Clone)]
pub enum Resource {
    // 磁力链接,InfoHash
    MagnetLink(String, String),
    // InfoHash
    MagnetInfoHash(String),
    // 种子文件字节,InfoHash
    TorrentFileBytes(Vec<u8>, String),
}

impl Resource {
    pub fn from_info_hash<T: Into<String>>(info_hash: T) -> Result<Self> {
        Ok(Resource::MagnetInfoHash(info_hash.into()))
    }

    pub fn from_magnet_link<T: Into<String>>(magnet_link: T) -> Result<Self> {
        let magnet_link = magnet_link.into();
        if let Some(part) = magnet_link.split("btih:").nth(1) {
            // 提取 InfoHash，它可能后跟其他参数（以 & 分隔）
            let info_hash = part.split('&').next().unwrap_or_default();
            if info_hash.len() == 40 {
                return Ok(Resource::MagnetLink(
                    magnet_link.clone(),
                    info_hash.to_string(),
                ));
            }
        }
        Err(anyhow::anyhow!("非法磁力链接，无法获取info_hash"))
    }

    pub fn from_torrent_file_bytes<T: Into<Vec<u8>>>(torrent_file_bytes: T) -> Result<Self> {
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
        }
    }

    pub fn magnet(&self) -> Option<String> {
        match self {
            Resource::MagnetLink(magnet, _) => Some(magnet.clone()),
            Resource::MagnetInfoHash(_) => {
                Some(format!("magnet:?xt=urn:btih:{}", self.info_hash()))
            }
            _ => None,
        }
    }

    pub fn info_hash(&self) -> &str {
        match self {
            Resource::MagnetInfoHash(hash) => hash,
            Resource::TorrentFileBytes(_, hash) => hash,
            Resource::MagnetLink(_, hash) => hash,
        }
    }
}
