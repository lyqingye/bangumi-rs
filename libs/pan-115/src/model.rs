use std::collections::HashMap;

use anyhow::Result;
use reqwest::header::HeaderMap;
use serde::Deserialize;

use super::errors::{map_115_error_code, Pan115Error};

#[derive(Deserialize, Default)]
pub struct LoginResp {
    #[serde(default)]
    pub code: i32,
    #[serde(default)]
    pub check_ssd: bool,
    #[serde(default)]
    pub data: Option<LoginData>,
    #[serde(default)]
    pub errno: i32,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub state: i32,
    #[serde(default)]
    pub expire: i64,
}

#[derive(Deserialize, Default)]
pub struct LoginData {
    #[serde(default)]
    pub expire: i64,
    #[serde(default)]
    pub link: String,
    #[serde(default)]
    pub user_id: i64,
}

impl LoginResp {
    pub fn is_ok(&self) -> Result<(), Pan115Error> {
        if self.state == 0 {
            Ok(())
        } else if self.code != 0 {
            map_115_error_code(self.code)
        } else {
            Err(Pan115Error::Unknown115Error)
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum StringInt {
    Int(i32),
    String(String),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum StringInt64 {
    Int(i64),
    String(String),
}

impl Default for StringInt64 {
    fn default() -> Self {
        StringInt64::Int(0)
    }
}

impl Default for StringInt {
    fn default() -> Self {
        StringInt::Int(0)
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct BasicResp {
    #[serde(rename = "errno", default)]
    pub errno: StringInt,
    #[serde(rename = "errNo", default)]
    pub err_no: i32,
    #[serde(rename = "errcode", default)]
    pub err_code: i32,
    #[serde(default)]
    pub error_msg: String,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub state: bool,
    #[serde(default)]
    pub errtype: String,
    #[serde(default)]
    pub msg: String,
}

impl BasicResp {
    pub fn is_ok(&self) -> Result<(), Pan115Error> {
        if self.state {
            Ok(())
        } else {
            if self.err_no != 0 {
                return map_115_error_code(self.err_no);
            }

            if let StringInt::Int(i) = &self.errno {
                if *i != 0 {
                    return map_115_error_code(*i);
                }
            }
            if self.err_code != 0 {
                return map_115_error_code(self.err_code);
            }

            if let StringInt::String(s) = &self.errno {
                if let Ok(i) = s.parse::<i32>() {
                    if i != 0 {
                        return map_115_error_code(i);
                    }
                }
            }

            if !self.error_msg.is_empty() {
                Err(Pan115Error::Wrap115Error(self.error_msg.clone()))
            } else if !self.error.is_empty() {
                Err(Pan115Error::Wrap115Error(self.error.clone()))
            } else {
                Err(Pan115Error::Unknown115Error)
            }
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct FileListResp {
    #[serde(flatten, default)]
    pub basic_resp: BasicResp,

    #[serde(rename = "aid", default)]
    pub area_id: String,

    #[serde(rename = "cid", default)]
    pub category_id: StringInt,

    #[serde(default)]
    pub count: i32,

    #[serde(default)]
    pub order: String,

    #[serde(default)]
    pub is_asc: i32,

    #[serde(default)]
    pub offset: i32,
    #[serde(default)]
    pub limit: i32,
    #[serde(default)]
    pub page_size: i32,

    #[serde(rename = "data", default)]
    pub files: Vec<FileInfo>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct FileInfo {
    #[serde(rename = "aid", default)]
    pub area_id: StringInt,

    #[serde(rename = "cid", default)]
    pub category_id: StringInt,

    #[serde(rename = "fid", default)]
    pub file_id: String,

    #[serde(rename = "pid", default)]
    pub parent_id: String,

    #[serde(rename = "n", default)]
    pub name: String,

    #[serde(rename = "ico", default)]
    pub file_type: String,

    #[serde(rename = "s", default)]
    pub size: StringInt64,

    #[serde(rename = "sha", default)]
    pub sha1: String,

    #[serde(rename = "pc", default)]
    pub pick_code: String,

    #[serde(rename = "m", default)]
    pub is_star: StringInt,

    #[serde(rename = "fl", default)]
    pub labels: Vec<LabelInfo>,

    #[serde(rename = "tp", default)]
    pub create_time: StringInt64,

    #[serde(rename = "t", default)]
    pub update_time: String,
}

impl FileInfo {
    pub fn is_dir(&self) -> bool {
        self.file_id.is_empty()
    }

    pub fn file_id(&self) -> String {
        if self.is_dir() {
            self.category_id()
        } else {
            self.file_id.clone()
        }
    }

    pub fn pid(&self) -> String {
        if self.is_dir() {
            self.category_id()
        } else {
            self.parent_id.clone()
        }
    }

    pub fn category_id(&self) -> String {
        match &self.category_id {
            StringInt::Int(i) => i.to_string(),
            StringInt::String(s) => s.to_owned(),
        }
    }

    pub fn file_size(&self) -> usize {
        match &self.size {
            StringInt64::Int(i) => *i as usize,
            StringInt64::String(s) => s.parse::<usize>().unwrap_or(0),
        }
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct LabelInfo {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub sort: StringInt,
    #[serde(rename = "create_time", default)]
    pub create_time: i64,

    #[serde(rename = "update_time", default)]
    pub update_time: i64,
}

#[derive(Debug, Deserialize)]
pub struct OfflineTaskResp {
    #[serde(flatten, default)]
    pub basic_resp: BasicResp,
    #[serde(rename = "total", default)]
    pub total: i64,
    #[serde(rename = "count", default)]
    pub count: i64,
    #[serde(rename = "page_row", default)]
    pub page_row: i64,
    #[serde(rename = "page_count", default)]
    pub page_count: i64,
    #[serde(rename = "page", default)]
    pub page: i64,
    #[serde(rename = "quota", default)]
    pub quota: i64,
    #[serde(rename = "tasks", default)]
    pub tasks: Vec<OfflineTask>,
}

#[derive(Debug, Deserialize)]
pub struct OfflineTask {
    #[serde(rename = "info_hash", default)]
    pub info_hash: String,
    #[serde(rename = "name", default)]
    pub name: String,
    #[serde(rename = "size", default)]
    pub size: i64,
    #[serde(rename = "url", default)]
    pub url: String,
    #[serde(rename = "add_time", default)]
    pub add_time: i64,
    #[serde(rename = "peers", default)]
    pub peers: i64,
    #[serde(rename = "rateDownload", default)]
    pub rate_download: f64,
    #[serde(rename = "status", default)]
    pub status: i32,
    #[serde(rename = "percentDone", default)]
    pub percent: f64,
    #[serde(rename = "last_update", default)]
    pub update_time: i64,
    #[serde(rename = "left_time", default)]
    pub left_time: i64,
    #[serde(rename = "file_id", default)]
    pub file_id: String,
    #[serde(rename = "delete_file_id", default)]
    pub del_file_id: String,
    #[serde(rename = "wp_path_id", default)]
    pub dir_id: String,
    #[serde(rename = "move", default)]
    pub move_field: i32, // 改名为 move_field
}

#[derive(Debug, Clone, Copy)]
pub enum OfflineTaskStatus {
    Pending = 0,
    Downloading = 1,
    Completed = 2,
    Failed = 3,
    Unknow = 4,
}
impl OfflineTask {
    pub fn status(&self) -> OfflineTaskStatus {
        match self.status {
            0 => OfflineTaskStatus::Pending,
            1 => OfflineTaskStatus::Downloading,
            2 => OfflineTaskStatus::Completed,
            3 => OfflineTaskStatus::Failed,
            _ => OfflineTaskStatus::Unknow,
        }
    }

    pub fn is_completed(&self) -> bool {
        self.status == 2
    }

    pub fn is_failed(&self) -> bool {
        self.status == 3
    }

    pub fn is_pending(&self) -> bool {
        self.status == 0
    }

    pub fn is_downloading(&self) -> bool {
        self.status == 1
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct DownloadResp {
    #[serde(flatten, default)]
    pub basic_resp: BasicResp,
    #[serde(rename = "data", default)]
    pub data: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct OfflineAddUrlResponse {
    #[serde(flatten, default)]
    pub basic_resp: BasicResp,
    #[serde(default)]
    pub result: Vec<OfflineTaskResponse>,
}

#[derive(Debug, Deserialize, Default)]
pub struct OfflineTaskResponse {
    #[serde(default)]
    pub info_hash: String,
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct GetFileInfoResponse {
    #[serde(flatten, default)]
    pub basic_resp: BasicResp,
    #[serde(rename = "data", default)]
    pub files: Vec<FileInfo>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct FileDownloadUrl {
    #[serde(default)]
    pub client: f64,
    #[serde(default)]
    pub oss_id: String,
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct DownloadInfo {
    #[serde(default)]
    pub file_name: String,
    #[serde(default)]
    pub file_size: StringInt64,
    #[serde(default)]
    pub pick_code: String,
    #[serde(default)]
    pub url: FileDownloadUrl,
    #[serde(skip, default)]
    pub header: HeaderMap,
}

impl DownloadInfo {
    pub fn file_size(&self) -> i64 {
        match &self.file_size {
            StringInt64::Int(i) => *i,
            StringInt64::String(s) => s.parse::<i64>().unwrap_or(0),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct DownloadData(pub HashMap<String, DownloadInfo>);

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct MkdirResp {
    #[serde(flatten)]
    pub basic_resp: BasicResp,

    #[serde(rename = "aid")]
    pub area_id: StringInt,

    #[serde(rename = "cid")]
    pub category_id: StringInt,

    #[serde(rename = "cname")]
    pub category_name: String,

    #[serde(rename = "file_id")]
    pub file_id: String,

    #[serde(rename = "file_name")]
    pub file_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_resp() {
        let json1 = r#"
    {
        "errno": 0,
        "errNo": 404,
        "error": "Not Found",
        "state": false,
        "errtype": "client_error",
        "msg": "Resource not found"
    }
    "#;

        let json2 = r#"
    {
        "errno": "500",
        "errNo": 500,
        "error": "Internal Server Error",
        "state": true,
        "errtype": "server_error",
        "msg": "Something went wrong"
    }
    "#;

        let _: BasicResp = serde_json::from_str(json1).unwrap();
        let _: BasicResp = serde_json::from_str(json2).unwrap();
    }

    #[test]
    fn test_file_info() {
        let json = r#"
        {
            "errno": 0,
            "errNo": 200,
            "error": "",
            "state": true,
            "errtype": "",
            "msg": "Success",
            "aid": "123",
            "cid": 1,
            "count": 10,
            "order": "name",
            "is_asc": 1,
            "offset": 0,
            "limit": 10,
            "page_size": 10,
            "data": [
                {
                    "aid": 123,
                    "cid": "1",
                    "fid": "file123",
                    "pid": "parent123",
                    "n": "example.txt",
                    "ico": "file",
                    "s": 1024,
                    "sha": "abc123",
                    "pc": "pickcode123",
                    "m": 0,
                    "fl": [
                        {
                            "id": "label1",
                            "name": "Important",
                            "color": "red",
                            "sort": 1,
                            "create_time": 1633024800,
                            "update_time": 1633024800
                        }
                    ],
                    "tp": 1633024800,
                    "t": "2021-10-01T12:00:00Z"
                }
            ]
        }
        "#;

        let resp: FileListResp = serde_json::from_str(json).unwrap();
        resp.basic_resp.is_ok().unwrap();
    }
}
