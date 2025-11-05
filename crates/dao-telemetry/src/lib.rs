//! DAO Telemetry — экспорт метрик и трейсинг
//!
//! Prometheus metrics exporter и tracing для DAO

use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub mod exporter;
pub mod metrics;

pub use exporter::MetricsExporter;
pub use metrics::{DaoMetrics, MetricsCollector};

/// Инициализация телеметрии
pub fn init_telemetry() -> anyhow::Result<()> {
    // Tracing subscriber с ENV filter
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Telemetry initialized");
    Ok(())
}

/// Запуск Prometheus exporter
pub async fn start_prometheus_exporter(bind_addr: SocketAddr) -> anyhow::Result<()> {
    let builder = PrometheusBuilder::new();
    builder
        .with_http_listener(bind_addr)
        .install()
        .map_err(|e| anyhow::anyhow!("Failed to install Prometheus exporter: {}", e))?;

    tracing::info!("Prometheus exporter started on {}", bind_addr);
    Ok(())
}

/// Регистрация метрик DAO
pub fn register_dao_metrics() {
    // Metrics будут автоматически регистрироваться при первом использовании
    tracing::debug!("DAO metrics registered");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_registration() {
        register_dao_metrics();
    }
}
