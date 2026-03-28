//! Active health checks — фоновые HTTP-пробы upstream'ов
//!
//! Каждый upstream с настроенным `health_check` получает отдельную
//! фоновую задачу, которая периодически делает HTTP GET к указанному
//! пути и передаёт результат в circuit breaker через `record_request()`.
//!
//! Конфигурация в dao.toml:
//! ```toml
//! [[routes.rule.upstreams]]
//! name = "api-backend-1"
//! url  = "http://127.0.0.1:8081"
//!
//! [routes.rule.upstreams.health_check]
//! path         = "/health"
//! interval_secs = 10
//! timeout_secs  = 5
//! ```

use super::state::UpstreamState;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Конфигурация активной проверки upstream'а
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthCheckConfig {
    /// URL-путь для проверки (по умолчанию: "/health")
    #[serde(default = "default_path")]
    pub path: String,

    /// Интервал между проверками в секундах (по умолчанию: 10)
    #[serde(default = "default_interval_secs")]
    pub interval_secs: u64,

    /// Таймаут одной проверки в секундах (по умолчанию: 5)
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_path() -> String { "/health".to_string() }
fn default_interval_secs() -> u64 { 10 }
fn default_timeout_secs() -> u64 { 5 }

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            path: default_path(),
            interval_secs: default_interval_secs(),
            timeout_secs: default_timeout_secs(),
        }
    }
}

/// Запуск фоновой задачи проверки для одного upstream.
///
/// Клонирует `UpstreamState` — внутренние Arc<> для stats и circuit breaker
/// разделяются, поэтому результаты проб попадают в общее состояние.
pub fn spawn_health_checker(upstream: UpstreamState, config: HealthCheckConfig) {
    tokio::spawn(async move {
        let interval = Duration::from_secs(config.interval_secs);
        let timeout  = Duration::from_secs(config.timeout_secs);

        // Небольшая задержка перед первой пробой, чтобы сервер успел стартовать
        tokio::time::sleep(Duration::from_secs(2)).await;

        loop {
            let t0 = Instant::now();
            let ok = probe_upstream(&upstream.url, &config.path, timeout).await
                .unwrap_or_else(|e| {
                    warn!("Health probe error [{}]: {}", upstream.name, e);
                    false
                });

            let latency_ms = t0.elapsed().as_secs_f64() * 1000.0;

            debug!(
                upstream = %upstream.name,
                healthy  = ok,
                latency_ms = %format!("{:.1}", latency_ms),
                "health probe",
            );

            // Сохраняем результат для daoctl upstreams
            upstream.update_health(ok, latency_ms);

            // Prometheus метрика: 1.0 = healthy, 0.0 = unhealthy
            metrics::gauge!("dao_health_probe_status", "upstream" => upstream.name.clone())
                .set(if ok { 1.0 } else { 0.0 });
            metrics::histogram!("dao_health_probe_latency_seconds", "upstream" => upstream.name.clone())
                .record(latency_ms / 1000.0);

            // Передаём результат в circuit breaker
            // При ошибке пишем нулевую задержку — важен только флаг success
            let record_latency = if ok {
                Duration::from_millis(latency_ms as u64)
            } else {
                Duration::ZERO
            };
            upstream.record_request(record_latency, ok);

            tokio::time::sleep(interval).await;
        }
    });
}

/// Выполняет одну HTTP-пробу к `base_url + path`.
/// Возвращает `true` если upstream ответил кодом < 500.
async fn probe_upstream(base_url: &str, path: &str, timeout: Duration) -> anyhow::Result<bool> {
    use http_body_util::BodyExt;
    use hyper::Request;
    use hyper_util::rt::TokioIo;
    use tokio::net::TcpStream;

    let url = format!("{}{}", base_url.trim_end_matches('/'), path);
    let uri: http::Uri = url.parse()
        .map_err(|e| anyhow::anyhow!("Invalid URL {}: {}", url, e))?;

    let host = uri.host()
        .ok_or_else(|| anyhow::anyhow!("No host in URL: {}", url))?
        .to_string();
    let port = uri.port_u16().unwrap_or(80);
    let addr = format!("{}:{}", host, port);

    let stream = tokio::time::timeout(timeout, TcpStream::connect(&addr))
        .await
        .map_err(|_| anyhow::anyhow!("Connect timeout to {}", addr))?
        .map_err(|e| anyhow::anyhow!("Connect error {}: {}", addr, e))?;

    let io = TokioIo::new(stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
        .await
        .map_err(|e| anyhow::anyhow!("HTTP handshake: {}", e))?;
    tokio::spawn(async move { let _ = conn.await; });

    let req_path = uri.path_and_query()
        .map(|p| p.as_str())
        .unwrap_or("/");

    let req = Request::builder()
        .method("GET")
        .uri(req_path)
        .header("Host", &host)
        .header("User-Agent", "dao-health-check/1.0")
        .body(http_body_util::Empty::<hyper::body::Bytes>::new())
        .map_err(|e| anyhow::anyhow!("Request build: {}", e))?;

    let resp = tokio::time::timeout(timeout, sender.send_request(req))
        .await
        .map_err(|_| anyhow::anyhow!("Request timeout"))?
        .map_err(|e| anyhow::anyhow!("Request error: {}", e))?;

    let status = resp.status().as_u16();
    // Drain body to free the connection slot
    let _ = resp.into_body().collect().await;

    Ok(status < 500)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = HealthCheckConfig::default();
        assert_eq!(cfg.path, "/health");
        assert_eq!(cfg.interval_secs, 10);
        assert_eq!(cfg.timeout_secs, 5);
    }
}
