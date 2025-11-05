//! Metrics types

use std::time::Instant;

/// Метрики конкретного запроса
#[derive(Debug, Clone)]
pub struct RequestMetrics {
    pub start_time: Instant,
    pub route_name: String,
    pub upstream_name: Option<String>,
    pub method: String,
    pub path: String,
    pub status_code: Option<u16>,
}

impl RequestMetrics {
    pub fn new(route_name: String, method: String, path: String) -> Self {
        Self {
            start_time: Instant::now(),
            route_name,
            upstream_name: None,
            method,
            path,
            status_code: None,
        }
    }

    pub fn duration(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

/// Системные метрики
#[derive(Debug, Clone, Default)]
pub struct SystemMetrics {
    pub total_requests: u64,
    pub total_errors: u64,
    pub active_connections: u64,
}
