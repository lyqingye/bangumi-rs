use pan_115::model::OfflineTask;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pan115Context {
    pub file_id: String,
    pub dir_id: String,
    pub file_name: String,
}

impl From<&OfflineTask> for Pan115Context {
    fn from(task: &OfflineTask) -> Self {
        Self {
            file_id: task.file_id.clone(),
            dir_id: task.dir_id.clone(),
            file_name: task.name.clone(),
        }
    }
}

impl TryFrom<String> for Pan115Context {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&s)?)
    }
}

impl TryFrom<Pan115Context> for String {
    type Error = serde_json::Error;

    fn try_from(val: Pan115Context) -> Result<Self, Self::Error> {
        serde_json::to_string(&val)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentFileInfo {
    pub name: String,
    pub size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TorrentContext {
    pub dir: String,
    pub files: Vec<TorrentFileInfo>,
}

impl TryFrom<String> for TorrentContext {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&s)?)
    }
}

impl TryFrom<TorrentContext> for String {
    type Error = serde_json::Error;

    fn try_from(val: TorrentContext) -> Result<Self, Self::Error> {
        serde_json::to_string(&val)
    }
}
