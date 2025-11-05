//! Sense — чувствование потока
//!
//! Модуль телеметрии и мониторинга:
//! - Сбор метрик запросов
//! - Латентность, throughput, ошибки
//! - Резонанс-метрики для политик

use crate::upstream::UpstreamState;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub mod metrics;
pub use metrics::{RequestMetrics, SystemMetrics};

/// Sense — система телеметрии
#[derive(Clone)]
pub struct Sense {
    upstreams: Arc<Vec<UpstreamState>>,
}

impl Sense {
    pub fn new(upstreams: Arc<Vec<UpstreamState>>) -> Self {
        Self { upstreams }
    }

    /// Запись результата запроса к upstream
    pub fn record_upstream_request(
        &self,
        upstream_name: &str,
        latency: Duration,
        success: bool,
    ) {
        if let Some(upstream) = self.upstreams.iter().find(|u| u.name == upstream_name) {
            upstream.record_request(latency, success);
        }
    }

    /// Получение резонанс-метрик для всех upstream
    pub fn get_resonance_metrics(&self) -> Vec<ResonanceMetrics> {
        self.upstreams
            .iter()
            .map(|u| {
                let stats = u.get_stats();
                ResonanceMetrics {
                    upstream_name: u.name.clone(),
                    load_resonance: calculate_load_resonance(&stats),
                    tempo_spikiness: stats.tempo_spikiness(),
                    p95_latency_ms: stats.p95_latency_ms(),
                    error_rate: stats.error_rate(),
                    current_rps: stats.current_rps(),
                }
            })
            .collect()
    }

    /// Получение состояния конкретного upstream
    pub fn get_upstream_state(&self, name: &str) -> Option<&UpstreamState> {
        self.upstreams.iter().find(|u| u.name == name)
    }
}

/// Метрики резонанса для upstream
#[derive(Debug, Clone)]
pub struct ResonanceMetrics {
    pub upstream_name: String,
    /// Совокупная "нагрузка" (latency + errors + queue)
    pub load_resonance: f64,
    /// Вариативность темпа запросов
    pub tempo_spikiness: f64,
    /// P95 латентность в мс
    pub p95_latency_ms: f64,
    /// Error rate (0.0 - 1.0)
    pub error_rate: f64,
    /// Текущий RPS
    pub current_rps: f64,
}

/// Вычисление load_resonance = сглаженная функция: latency p95 + error_rate + queue_depth
fn calculate_load_resonance(stats: &crate::upstream::UpstreamStats) -> f64 {
    let latency_component = (stats.p95_latency_ms() / 100.0).min(10.0); // Нормализация до ~0-10
    let error_component = stats.error_rate() * 10.0; // 0-10
    let queue_component = stats.queue_depth_norm() * 10.0; // 0-10

    latency_component + error_component + queue_component
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Intent;

    #[test]
    fn test_sense_recording() {
        let upstreams = vec![UpstreamState::new(
            "test".to_string(),
            "http://localhost:8080".to_string(),
            vec![Intent::new("test")],
            1,
        )];

        let sense = Sense::new(Arc::new(upstreams));
        sense.record_upstream_request("test", Duration::from_millis(50), true);

        let metrics = sense.get_resonance_metrics();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].upstream_name, "test");
    }
}
