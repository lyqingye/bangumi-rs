use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Metrics {
    pub num_of_tasks: usize,
}
