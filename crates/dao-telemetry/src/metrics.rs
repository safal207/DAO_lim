//! DAO metrics collection

use std::sync::Arc;
use parking_lot::RwLock;

/// Коллектор метрик DAO
#[derive(Clone)]
pub struct MetricsCollector {
    metrics: Arc<RwLock<DaoMetrics>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(DaoMetrics::default())),
        }
    }

    /// Запись метрик запроса
    pub fn record_request(&self, route: &str, upstream: &str, duration_secs: f64, status: u16) {
        let mut m = self.metrics.write();
        m.total_requests += 1;

        // Prometheus metrics
        metrics::counter!("dao_requests_total", "route" => route.to_string()).increment(1);
        metrics::histogram!("dao_request_duration_seconds", "route" => route.to_string())
            .record(duration_secs);

        if status >= 500 {
            m.total_errors += 1;
        }

        metrics::counter!(
            "dao_upstream_requests_total",
            "upstream" => upstream.to_string(),
            "status" => status.to_string()
        )
        .increment(1);
    }

    /// Обновление счетчика активных соединений
    pub fn set_active_connections(&self, count: u64) {
        let mut m = self.metrics.write();
        m.active_connections = count;
        metrics::gauge!("dao_upstream_connections").set(count as f64);
    }

    /// Получение текущих метрик
    pub fn get_metrics(&self) -> DaoMetrics {
        self.metrics.read().clone()
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Метрики DAO
#[derive(Debug, Clone, Default)]
pub struct DaoMetrics {
    pub total_requests: u64,
    pub total_errors: u64,
    pub active_connections: u64,
}
