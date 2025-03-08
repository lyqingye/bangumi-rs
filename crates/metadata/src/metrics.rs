use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ServiceStatus {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Service {
    pub name: String,
    pub status: ServiceStatus,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Metrics {
    pub services: Vec<Service>,
    pub last_refresh_time: i64,
}
