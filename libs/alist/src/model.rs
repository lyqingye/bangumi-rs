use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;

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
    Failed = 3,
    Canceling = 4,
    Canceled = 5,
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
#[derive(Debug, Deserialize, Serialize, Default)]
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

/// 批量操作响应
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct BatchOperationResult {
    pub code: i32,
    pub message: String,
    pub data: HashMap<String, String>,
}

/// 批量操作请求体
#[derive(Debug, Serialize)]
pub struct BatchOperationRequest {
    pub task_ids: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct AddOfflineDownloadTaskResult {
    pub tasks: Vec<TaskInfo>,
}

#[derive(Debug, Serialize)]
pub struct AddOfflineDownloadTaskRequest {
    pub urls: Vec<String>,
    pub path: String,
    pub tool: String,
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
        if s.is_empty() {
            Ok(None)
        } else {
            Ok(Some(s))
        }
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

/// 展示如何使用NoneIfEmpty模块的示例结构体
#[derive(Debug, Deserialize, Serialize)]
struct ExampleWithNoneIfEmpty {
    pub id: String,
    #[serde(with = "none_if_empty")]
    pub optional_field: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
