use pan_115::model::OfflineTask;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pan115Context {
    pub file_id: String,
    pub dir_id: String,
    pub file_name: String
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

impl Into<String> for Pan115Context {
    fn into(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

