//! DAO Telemetry — Prometheus metrics + OpenTelemetry tracing
//!
//! Инициализация телеметрии в два режима:
//! - Без OTLP: только console/JSON логи + Prometheus метрики
//! - С OTLP: все вышеперечисленное + трассировка через OTLP gRPC export

use anyhow::Result;
use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub mod exporter;
pub mod metrics;

pub use exporter::MetricsExporter;
pub use metrics::{DaoMetrics, MetricsCollector};

/// Инициализация телеметрии.
///
/// `otlp_endpoint` — опциональный OTLP gRPC адрес для OpenTelemetry.
/// Например: `Some("http://localhost:4317")`.
///
/// Если `None` — запускается только console tracing без OTel export.
pub fn init_telemetry_with_otlp(otlp_endpoint: Option<&str>) -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false);

    if let Some(endpoint) = otlp_endpoint {
        use opentelemetry_otlp::WithExportConfig;
        use opentelemetry::trace::TracerProvider as _;
        use opentelemetry_sdk::runtime::Tokio;

        let exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(endpoint);

        let tracer_provider = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(exporter)
            .with_trace_config(
                opentelemetry_sdk::trace::Config::default()
                    .with_resource(opentelemetry_sdk::Resource::new(vec![
                        opentelemetry::KeyValue::new(
                            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                            "dao",
                        ),
                        opentelemetry::KeyValue::new(
                            opentelemetry_semantic_conventions::resource::SERVICE_VERSION,
                            dao_core::DAO_VERSION,
                        ),
                    ])),
            )
            .install_batch(Tokio)
            .map_err(|e| anyhow::anyhow!("Failed to install OTLP exporter: {}", e))?;

        let tracer = tracer_provider.tracer("dao");
        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .with(otel_layer)
            .init();

        tracing::info!("Telemetry initialized with OTLP export to {}", endpoint);
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .init();

        tracing::info!("Telemetry initialized (no OTLP endpoint configured)");
    }

    Ok(())
}

/// Инициализация телеметрии без OTLP (обратная совместимость).
pub fn init_telemetry() -> Result<()> {
    init_telemetry_with_otlp(None)
}

/// Запуск Prometheus exporter
pub async fn start_prometheus_exporter(bind_addr: SocketAddr) -> Result<()> {
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
