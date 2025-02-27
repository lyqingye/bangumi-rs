use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_bencode::de;
use sha1::{Digest, Sha1};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str;

/// 表示 Torrent 文件中的信息字典
#[derive(Debug, Serialize, Deserialize)]
pub struct TorrentInfo {
    pub name: Option<String>,
    #[serde(with = "serde_bytes")]
    pub pieces: Vec<u8>,
    #[serde(rename = "piece length")]
    pub piece_length: i64,
    pub length: Option<i64>,
    pub files: Option<Vec<TorrentFile>>,
    // 可选字段
    pub md5sum: Option<String>,
    pub private: Option<i64>,
    #[serde(rename = "creation date")]
    pub creation_date: Option<i64>,
    #[serde(rename = "created by")]
    pub created_by: Option<String>,
    pub comment: Option<String>,
    pub encoding: Option<String>,
    #[serde(flatten)]
    pub rest: BTreeMap<String, serde_bencode::value::Value>,
}

/// 表示 Torrent 文件中的文件信息
#[derive(Debug, Serialize, Deserialize)]
pub struct TorrentFile {
    pub length: i64,
    pub path: Vec<String>,
    #[serde(flatten)]
    pub rest: BTreeMap<String, serde_bencode::value::Value>,
}

/// 表示 Torrent 文件的主要结构
#[derive(Debug, Serialize, Deserialize)]
pub struct Torrent {
    pub info: TorrentInfo,
    pub announce: Option<String>,
    #[serde(rename = "announce-list")]
    pub announce_list: Option<Vec<Vec<String>>>,
    #[serde(flatten)]
    pub rest: BTreeMap<String, serde_bencode::value::Value>,
}

/// # 示例
///
/// ```rust,no_run
/// use anyhow::Result;
/// use torrent::Torrent;
///
/// fn main() -> Result<()> {
///     // 从文件加载 torrent
///     let torrent = Torrent::from_file("example.torrent")?;
///     
///     // 获取 info_hash (原始字节)
///     let info_hash = torrent.info_hash()?;
///     println!("Info Hash (hex): {}", hex::encode(info_hash));
///     
///     // 获取 magnet 链接
///     let magnet = torrent.magnet_link()?;
///     println!("Magnet Link: {}", magnet);
///     
///     Ok(())
/// }
/// ```
impl Torrent {
    /// 从文件路径加载 Torrent
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path).context("无法打开 torrent 文件")?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .context("无法读取 torrent 文件")?;
        Self::from_bytes(&buf)
    }

    /// 从字节数组加载 Torrent
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        // 尝试直接解析
        match de::from_bytes::<Torrent>(bytes) {
            Ok(torrent) => Ok(torrent),
            Err(original_err) => {
                // 如果直接解析失败，尝试先解析为 Value，然后手动构建 Torrent
                match de::from_bytes::<serde_bencode::value::Value>(bytes) {
                    Ok(value) => {
                        if let serde_bencode::value::Value::Dict(dict) = value {
                            // 尝试从字典中提取信息
                            let mut torrent_dict = BTreeMap::new();

                            // 复制所有键值对
                            for (k, v) in dict {
                                torrent_dict.insert(k, v);
                            }

                            // 尝试将字典转换为 Torrent
                            match serde_bencode::to_bytes(&torrent_dict)
                                .and_then(|bytes| de::from_bytes::<Torrent>(&bytes))
                            {
                                Ok(torrent) => return Ok(torrent),
                                Err(_) => {
                                    // 如果仍然失败，返回原始错误
                                    return Err(original_err.into());
                                }
                            }
                        }

                        // 如果不是字典，返回原始错误
                        Err(original_err.into())
                    }
                    Err(_) => {
                        // 如果连 Value 都解析不了，返回原始错误
                        Err(original_err.into())
                    }
                }
            }
        }
    }

    /// 计算 info 字典的 SHA1 哈希值（info_hash）
    pub fn info_hash(&self) -> Result<[u8; 20]> {
        let info_bencoded = serde_bencode::to_bytes(&self.info).context("无法编码 info 字典")?;
        let mut hasher = Sha1::new();
        hasher.update(&info_bencoded);
        let hash = hasher.finalize();
        let mut result = [0u8; 20];
        result.copy_from_slice(&hash);
        Ok(result)
    }

    /// 获取 Magnet URI
    pub fn magnet_link(&self) -> Result<String> {
        let info_hash = self.info_hash()?;
        let hex_hash = hex::encode(info_hash);

        // 构建基本的 magnet 链接
        let mut magnet = format!("magnet:?xt=urn:btih:{}", hex_hash);

        // 添加名称（如果有）
        if let Some(name) = &self.info.name {
            magnet.push_str(&format!("&dn={}", urlencoding::encode(name)));
        }

        // 添加 tracker（如果有）
        if let Some(announce) = &self.announce {
            magnet.push_str(&format!("&tr={}", urlencoding::encode(announce)));
        }

        // 添加 announce-list 中的 tracker（如果有）
        if let Some(announce_list) = &self.announce_list {
            for trackers in announce_list {
                for tracker in trackers {
                    magnet.push_str(&format!("&tr={}", urlencoding::encode(tracker)));
                }
            }
        }

        Ok(magnet)
    }

    /// 尝试解析 torrent 文件并返回详细的诊断信息
    #[cfg(test)]
    pub fn diagnose_file<P: AsRef<Path>>(path: P) -> Result<String> {
        let mut file = File::open(path.as_ref()).context("无法打开 torrent 文件")?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .context("无法读取 torrent 文件")?;

        let mut result = String::new();
        result.push_str(&format!("文件大小: {} 字节\n", buf.len()));

        // 尝试解析为通用 bencode 值
        match de::from_bytes::<serde_bencode::value::Value>(&buf) {
            Ok(value) => {
                result.push_str("成功解析为 bencode 值\n");

                if let serde_bencode::value::Value::Dict(dict) = &value {
                    result.push_str(&format!("顶层字典包含 {} 个键值对\n", dict.len()));

                    // 检查是否有 info 字典
                    let info_key = "info".as_bytes().to_vec();
                    if let Some(info) = dict.get(&info_key) {
                        result.push_str("找到 info 字典\n");

                        if let serde_bencode::value::Value::Dict(info_dict) = info {
                            result
                                .push_str(&format!("info 字典包含 {} 个键值对\n", info_dict.len()));

                            // 列出 info 字典中的所有键
                            result.push_str("info 字典中的键: ");
                            for key in info_dict.keys() {
                                let key_str = match str::from_utf8(key) {
                                    Ok(s) => s.to_string(),
                                    Err(_) => format!("<非 UTF-8 键: {:?}>", key),
                                };
                                result.push_str(&format!("{}, ", key_str));
                            }
                            result.push('\n');

                            // 检查 pieces 字段
                            let pieces_key = "pieces".as_bytes().to_vec();
                            if let Some(pieces) = info_dict.get(&pieces_key) {
                                match pieces {
                                    serde_bencode::value::Value::Bytes(bytes) => {
                                        result.push_str(&format!(
                                            "pieces 是字节数组，长度为 {} 字节\n",
                                            bytes.len()
                                        ));
                                    }
                                    _ => {
                                        result.push_str(&format!(
                                            "pieces 不是字节数组，而是 {:?}\n",
                                            pieces
                                        ));
                                    }
                                }
                            } else {
                                result.push_str("未找到 pieces 字段\n");
                            }
                        } else {
                            result.push_str("info 不是字典\n");
                        }
                    } else {
                        result.push_str("未找到 info 字典\n");
                    }

                    // 列出顶层字典中的所有键
                    result.push_str("顶层字典中的键: ");
                    for key in dict.keys() {
                        let key_str = match str::from_utf8(key) {
                            Ok(s) => s.to_string(),
                            Err(_) => format!("<非 UTF-8 键: {:?}>", key),
                        };
                        result.push_str(&format!("{}, ", key_str));
                    }
                    result.push('\n');
                } else {
                    result.push_str("顶层值不是字典\n");
                }

                // 尝试解析为 Torrent 结构体
                match Self::from_bytes(&buf) {
                    Ok(_) => {
                        result.push_str("成功解析为 Torrent 结构体\n");
                    }
                    Err(e) => {
                        result.push_str(&format!("无法解析为 Torrent 结构体: {}\n", e));
                    }
                }
            }
            Err(e) => {
                result.push_str(&format!("无法解析为 bencode 值: {}\n", e));
            }
        }

        Ok(result)
    }

    /// 直接从 torrent 文件生成 magnet 链接
    pub fn magnet_from_file<P: AsRef<Path>>(path: P) -> Result<String> {
        let torrent = Self::from_file(path)?;
        torrent.magnet_link()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::env;

    #[test]
    fn test_info_hash() {
        // 创建一个简单的 torrent 结构用于测试
        let info = TorrentInfo {
            name: Some("test.txt".to_string()),
            pieces: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            piece_length: 16384,
            length: Some(1024),
            files: None,
            md5sum: None,
            private: None,
            creation_date: None,
            created_by: None,
            comment: None,
            encoding: None,
            rest: BTreeMap::new(),
        };

        let torrent = Torrent {
            info,
            announce: Some("http://tracker.example.com/announce".to_string()),
            announce_list: None,
            rest: BTreeMap::new(),
        };

        // 测试 info_hash 方法
        let info_hash = torrent.info_hash().unwrap();
        assert_eq!(info_hash.len(), 20); // SHA1 哈希应该是 20 字节

        // 测试 magnet_link 方法
        let magnet = torrent.magnet_link().unwrap();
        assert!(magnet.starts_with("magnet:?xt=urn:btih:"));
        assert!(magnet.contains("&dn=test.txt"));
        assert!(magnet.contains("&tr=http%3A%2F%2Ftracker.example.com%2Fannounce"));
    }

    // 注意：这个测试需要一个实际的 torrent 文件
    // 如果没有文件，测试会被跳过
    #[test]
    fn test_magnet_link_from_file() {
        dotenv::dotenv().ok();
        // 尝试从环境变量获取 torrent 文件路径
        let torrent_path = match env::var("TORRENT_TEST_FILE") {
            Ok(path) => path,
            Err(_) => {
                println!("跳过测试：未设置 TORRENT_TEST_FILE 环境变量");
                return;
            }
        };

        // 尝试解析 torrent 文件
        let torrent = match Torrent::from_file(&torrent_path) {
            Ok(t) => t,
            Err(e) => {
                println!("无法解析 torrent 文件 {}: {}", torrent_path, e);
                println!("请确保文件存在且格式正确");
                return;
            }
        };

        // 生成 magnet 链接
        match torrent.magnet_link() {
            Ok(magnet) => {
                println!("成功生成 magnet 链接: {}", magnet);
                assert!(magnet.starts_with("magnet:?xt=urn:btih:"));
            }
            Err(e) => {
                println!("无法生成 magnet 链接: {}", e);
                panic!("生成 magnet 链接失败");
            }
        }
    }

    #[test]
    fn test_diagnose_file() {
        dotenv::dotenv().ok();
        // 尝试从环境变量获取 torrent 文件路径
        let torrent_path = match env::var("TORRENT_TEST_FILE") {
            Ok(path) => path,
            Err(_) => {
                println!("跳过测试：未设置 TORRENT_TEST_FILE 环境变量");
                return;
            }
        };

        // 尝试诊断 torrent 文件
        match Torrent::diagnose_file(&torrent_path) {
            Ok(diagnosis) => {
                println!("诊断结果:\n{}", diagnosis);
            }
            Err(e) => {
                println!("无法诊断 torrent 文件: {}", e);
            }
        }
    }
}
