use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum WorkerState {
    Collecting,
    Idle,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkerMetrics {
    pub name: String,
    pub state: WorkerState,
    pub last_collection_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl WorkerMetrics {
    pub fn new(name: String, state: WorkerState) -> WorkerMetrics {
        Self {
            name,
            state,
            last_collection_time: None,
        }
    }

    pub fn set_state(&mut self, state: WorkerState) {
        self.state = state;
    }

    pub fn set_last_collection_time(&mut self, time: chrono::DateTime<chrono::Utc>) {
        self.last_collection_time = Some(time);
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Metrics {
    pub workers: Vec<WorkerMetrics>,
}
