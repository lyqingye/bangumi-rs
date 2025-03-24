use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct Response<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

/// 任务类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TaskType {
    #[default]
    Upload,
    Copy,
    OfflineDownload,
    OfflineDownloadTransfer,
    Decompress,
    DecompressUpload,
}

impl TaskType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskType::Upload => "upload",
            TaskType::Copy => "copy",
            TaskType::OfflineDownload => "offline_download",
            TaskType::OfflineDownloadTransfer => "offline_download_transfer",
            TaskType::Decompress => "decompress",
            TaskType::DecompressUpload => "decompress_upload",
        }
    }
}

/// 任务状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr, Default)]
#[repr(u8)]
pub enum TaskState {
    #[default]
    Pending = 0,
    Running = 1,
    Succeeded = 2,
    Canceling = 3,
    Canceled = 4,
    Errored = 5,
    Failing = 6,
    Failed = 7,
    WaitingRetry = 8,
    BeforeRetry = 9,
}

/// 用户角色枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr, Default)]
#[repr(u8)]
pub enum UserRole {
    #[default]
    Normal = 0,
    Guest = 1,
    Admin = 2,
}

/// 任务信息
#[derive(Debug, Deserialize, Serialize)]
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub creator: String,
    pub creator_role: UserRole,
    pub state: TaskState,
    #[serde(with = "none_if_empty")]
    pub status: Option<String>,
    pub progress: f32,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub total_bytes: i64,
    #[serde(with = "none_if_empty")]
    pub error: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct AddOfflineDownloadTaskResult {
    pub tasks: Vec<TaskInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum Tools {
    #[default]
    #[serde(rename = "qBittorrent")]
    Qbittorrent,
    #[serde(rename = "Transmission")]
    Transmission,
    #[serde(rename = "115 Cloud")]
    Pan115,
    #[serde(rename = "PikPak")]
    PikPak,
}

impl TryFrom<String> for Tools {
    type Error = Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "qBittorrent" => Ok(Tools::Qbittorrent),
            "Transmission" => Ok(Tools::Transmission),
            "115 Cloud" => Ok(Tools::Pan115),
            "PikPak" => Ok(Tools::PikPak),
            _ => Err(Error::UnsupportedTool(s)),
        }
    }
}

impl From<Tools> for String {
    fn from(tool: Tools) -> Self {
        match tool {
            Tools::Qbittorrent => "qBittorrent".to_string(),
            Tools::Transmission => "Transmission".to_string(),
            Tools::Pan115 => "115 Cloud".to_string(),
            Tools::PikPak => "PikPak".to_string(),
        }
    }
}

impl Display for Tools {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

#[derive(Debug, Serialize)]
pub struct AddOfflineDownloadTaskRequest {
    pub urls: Vec<String>,
    pub path: String,
    pub tool: Tools,
    pub delete_policy: String,
}

/// 登录请求参数
#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub otp_code: Option<String>,
}

/// 登录响应数据
#[derive(Debug, Deserialize, Default)]
pub struct LoginResponse {
    pub token: String,
}

/// 用户信息
#[derive(Debug, Deserialize, Default)]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub base_path: String,
    pub role: i32,
    pub disabled: bool,
    pub permission: i32,
    pub sso_id: String,
    pub otp: bool,
}

/// 当字符串为空时解析为 None 的包装类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoneIfEmpty(pub Option<String>);

// 帮助模块，实现serde的各种序列化与反序列化方法
mod none_if_empty {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() { Ok(None) } else { Ok(Some(s)) }
    }

    pub fn serialize<S>(value: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(s) => serializer.serialize_str(s),
            None => serializer.serialize_str(""),
        }
    }
}

/// 列表中的文件项目
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FsListFileItem {
    pub name: String,
    pub size: i64,
    pub is_dir: bool,
    pub modified: String,
    #[serde(default)]
    pub created: String,
    pub sign: Option<String>,
    pub thumb: Option<String>,
    #[serde(rename = "type")]
    pub file_type: i32,
    #[serde(default)]
    pub hashinfo: String,
    pub hash_info: Option<serde_json::Value>,
}

/// 文件目录列表响应
#[derive(Debug, Deserialize, Default)]
pub struct FsListResponse {
    pub content: Vec<FsListFileItem>,
    pub total: i32,
    #[serde(with = "none_if_empty")]
    pub readme: Option<String>,
    #[serde(with = "none_if_empty")]
    pub header: Option<String>,
    pub write: bool,
    pub provider: String,
}

/// 文件详情响应
#[derive(Debug, Deserialize, Default)]
pub struct FsGetResponse {
    pub name: String,
    pub size: i64,
    pub is_dir: bool,
    pub modified: String,
    #[serde(default)]
    pub created: String,
    pub sign: Option<String>,
    pub thumb: Option<String>,
    #[serde(rename = "type")]
    pub file_type: i32,
    #[serde(default)]
    pub hashinfo: String,
    pub hash_info: Option<serde_json::Value>,
    pub raw_url: String,
    #[serde(with = "none_if_empty")]
    pub readme: Option<String>,
    #[serde(with = "none_if_empty")]
    pub header: Option<String>,
    pub provider: String,
    pub related: Option<serde_json::Value>,
}

/// 文件列表请求
#[derive(Debug, Serialize)]
pub struct FsListRequest {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    pub page: u32,
    pub per_page: u32,
    pub refresh: bool,
}

/// 文件获取请求
#[derive(Debug, Serialize)]
pub struct FsGetRequest {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// 完整文件列表响应（无分页）
#[derive(Debug, Default)]
pub struct AllFilesList {
    pub files: Vec<FsListFileItem>,
    pub total_count: usize,
    pub total_size: i64,
    pub provider: String,
}

/// 递归文件列表中的文件项（带完整路径）
#[derive(Debug, Clone)]
pub struct RecursiveFileItem {
    pub file: FsListFileItem,
    pub full_path: String,
}

/// 文件递归展开结果
#[derive(Debug, Default)]
pub struct RecursiveFilesList {
    pub files: Vec<RecursiveFileItem>,
    pub directories: Vec<String>,
    pub total_count: usize,
    pub total_size: i64,
    pub total_dirs: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    struct ExampleWithNoneIfEmpty {
        pub id: String,
        #[serde(with = "none_if_empty")]
        pub optional_field: Option<String>,
    }

    #[test]
    fn test_deserialize_add_offline_download_task_result() {
        let json = r#"{"code":200,"message":"success","data":{"tasks":[{"id":"yZLkyXOWOSX3-Hsn_-fBc","name":"download https://mikanani.me/Download/20250317/7cbf45178ebe4ee0607fa51b6d630638ebdd7c63.torrent to (/downloads)","creator":"admin","creator_role":2,"state":0,"status":"","progress":0,"start_time":null,"end_time":null,"total_bytes":0,"error":""}]}}"#;
        let result: Response<AddOfflineDownloadTaskResult> = serde_json::from_str(json).unwrap();
        println!("{:?}", result);
    }

    #[test]
    fn test_empty_string_as_none() {
        let json = r#"{
            "id": "test_id",
            "name": "test task",
            "creator": "admin",
            "creator_role": 2,
            "state": 0,
            "status": "",
            "progress": 0,
            "start_time": null,
            "end_time": null,
            "total_bytes": 0,
            "error": ""
        }"#;

        let task_info: TaskInfo = serde_json::from_str(json).unwrap();
        assert!(
            task_info.status.is_none(),
            "空字符串的status应该被反序列化为None"
        );
        assert!(
            task_info.error.is_none(),
            "空字符串的error应该被反序列化为None"
        );
    }

    #[test]
    fn test_non_empty_string() {
        let json = r#"{
            "id": "test_id",
            "name": "test task",
            "creator": "admin",
            "creator_role": 2,
            "state": 0,
            "status": "processing",
            "progress": 0,
            "start_time": null,
            "end_time": null,
            "total_bytes": 0,
            "error": "some error"
        }"#;

        let task_info: TaskInfo = serde_json::from_str(json).unwrap();
        assert_eq!(
            task_info.status,
            Some("processing".to_string()),
            "非空字符串的status应该正确保留"
        );
        assert_eq!(
            task_info.error,
            Some("some error".to_string()),
            "非空字符串的error应该正确保留"
        );
    }

    #[test]
    fn test_none_if_empty_module() {
        // 测试反序列化
        #[derive(Debug, Deserialize, PartialEq)]
        struct TestStruct {
            #[serde(with = "none_if_empty")]
            field: Option<String>,
        }

        let empty_json = r#"{"field":""}"#;
        let test_struct: TestStruct = serde_json::from_str(empty_json).unwrap();
        assert_eq!(test_struct.field, None);

        let non_empty_json = r#"{"field":"value"}"#;
        let test_struct: TestStruct = serde_json::from_str(non_empty_json).unwrap();
        assert_eq!(test_struct.field, Some("value".to_string()));

        // 测试序列化
        #[derive(Debug, Serialize, PartialEq)]
        struct TestStructSer {
            #[serde(with = "none_if_empty")]
            field: Option<String>,
        }

        let test_struct = TestStructSer { field: None };
        let json = serde_json::to_string(&test_struct).unwrap();
        assert_eq!(json, r#"{"field":""}"#);

        let test_struct = TestStructSer {
            field: Some("value".to_string()),
        };
        let json = serde_json::to_string(&test_struct).unwrap();
        assert_eq!(json, r#"{"field":"value"}"#);
    }

    #[test]
    fn test_serialize_tools() {
        let tools = Tools::Qbittorrent;
        let json = serde_json::to_string(&tools).unwrap();
        assert_eq!(json, r#""qBittorrent""#);

        let tools = Tools::Transmission;
        let json = serde_json::to_string(&tools).unwrap();
        assert_eq!(json, r#""Transmission""#);

        let tools = Tools::Pan115;
        let json = serde_json::to_string(&tools).unwrap();
        assert_eq!(json, r#""115 Cloud""#);
    }
}
