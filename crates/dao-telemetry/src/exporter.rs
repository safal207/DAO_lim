//! Metrics exporter

use parking_lot::RwLock;
use std::sync::Arc;

/// Экспортер метрик
pub struct MetricsExporter {
    metrics: Arc<RwLock<MetricsSnapshot>>,
}

impl MetricsExporter {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(MetricsSnapshot::default())),
        }
    }

    pub fn record_request(&self, duration_ms: f64, success: bool) {
        let mut metrics = self.metrics.write();
        metrics.total_requests += 1;
        if success {
            metrics.successful_requests += 1;
        } else {
            metrics.failed_requests += 1;
        }
        metrics.total_duration_ms += duration_ms;
    }

    pub fn get_snapshot(&self) -> MetricsSnapshot {
        self.metrics.read().clone()
    }
}

impl Default for MetricsExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_duration_ms: f64,
}

impl MetricsSnapshot {
    pub fn average_duration_ms(&self) -> f64 {
        if self.total_requests > 0 {
            self.total_duration_ms / self.total_requests as f64
        } else {
            0.0
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_requests > 0 {
            self.successful_requests as f64 / self.total_requests as f64
        } else {
            0.0
        }
    }
}
