//! Upstream management — работа с backend серверами

use crate::Intent;
use hdrhistogram::Histogram;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Состояние upstream сервера
#[derive(Debug, Clone)]
pub struct UpstreamState {
    pub name: String,
    pub url: String,
    pub intents: Vec<Intent>,
    pub weight: u32,
    pub stats: Arc<RwLock<UpstreamStats>>,
}

impl UpstreamState {
    pub fn new(name: String, url: String, intents: Vec<Intent>, weight: u32) -> Self {
        Self {
            name,
            url,
            intents,
            weight,
            stats: Arc::new(RwLock::new(UpstreamStats::new())),
        }
    }

    /// Вычисление intent match score (0.0 = полное совпадение, 1.0 = нет совпадений)
    pub fn intent_gap(&self, request_intent: &Intent) -> f64 {
        if self.intents.is_empty() {
            return 0.0; // No preferences
        }

        for intent in &self.intents {
            if intent.matches(request_intent) {
                return 0.0; // Perfect match
            }
        }

        1.0 // No match
    }

    /// Запись результата запроса
    pub fn record_request(&self, latency: Duration, success: bool) {
        let mut stats = self.stats.write();
        stats.record(latency, success);
    }

    /// Получение текущей статистики
    pub fn get_stats(&self) -> UpstreamStats {
        self.stats.read().clone()
    }
}

/// Статистика upstream сервера
#[derive(Debug, Clone)]
pub struct UpstreamStats {
    /// Гистограмма латентности (в микросекундах)
    latency_hist: Histogram<u64>,

    /// Количество успешных запросов
    pub success_count: u64,

    /// Количество ошибок
    pub error_count: u64,

    /// Время последнего обновления
    pub last_update: Instant,

    /// Скользящий RPS за последнюю минуту
    rps_window: Vec<(Instant, bool)>,
}

impl UpstreamStats {
    pub fn new() -> Self {
        Self {
            latency_hist: Histogram::<u64>::new_with_bounds(1, 60_000_000, 3).unwrap(),
            success_count: 0,
            error_count: 0,
            last_update: Instant::now(),
            rps_window: Vec::with_capacity(10000),
        }
    }

    /// Запись результата запроса
    pub fn record(&mut self, latency: Duration, success: bool) {
        let micros = latency.as_micros() as u64;
        let _ = self.latency_hist.record(micros);

        if success {
            self.success_count += 1;
        } else {
            self.error_count += 1;
        }

        let now = Instant::now();
        self.last_update = now;

        // Обновление RPS window
        self.rps_window.push((now, success));

        // Очистка старых записей (> 60 секунд)
        let cutoff = now - Duration::from_secs(60);
        self.rps_window.retain(|(ts, _)| *ts > cutoff);
    }

    /// P95 латентность в миллисекундах
    pub fn p95_latency_ms(&self) -> f64 {
        if self.latency_hist.len() == 0 {
            return 0.0;
        }
        self.latency_hist.value_at_quantile(0.95) as f64 / 1000.0
    }

    /// P50 (медиана) латентность в миллисекундах
    pub fn p50_latency_ms(&self) -> f64 {
        if self.latency_hist.len() == 0 {
            return 0.0;
        }
        self.latency_hist.value_at_quantile(0.50) as f64 / 1000.0
    }

    /// Error rate (0.0 - 1.0)
    pub fn error_rate(&self) -> f64 {
        let total = self.success_count + self.error_count;
        if total == 0 {
            return 0.0;
        }
        self.error_count as f64 / total as f64
    }

    /// Текущий RPS за последние 60 секунд
    pub fn current_rps(&self) -> f64 {
        if self.rps_window.is_empty() {
            return 0.0;
        }

        let now = Instant::now();
        let window_start = now - Duration::from_secs(60);
        let count = self.rps_window.iter()
            .filter(|(ts, _)| *ts > window_start)
            .count();

        count as f64 / 60.0
    }

    /// Нормализованная глубина очереди (для будущей реализации)
    pub fn queue_depth_norm(&self) -> f64 {
        // TODO: реальная имплементация с tracking активных запросов
        0.0
    }

    /// Spikiness — вариативность RPS
    pub fn tempo_spikiness(&self) -> f64 {
        if self.rps_window.len() < 10 {
            return 0.0;
        }

        // Простая метрика: стандартное отклонение RPS по 10-секундным бинам
        let now = Instant::now();
        let mut bins = vec![0u32; 6]; // 6 бинов по 10 секунд

        for (ts, _) in &self.rps_window {
            let age = now.duration_since(*ts).as_secs();
            if age < 60 {
                let bin = (age / 10) as usize;
                bins[bin] += 1;
            }
        }

        let mean = bins.iter().sum::<u32>() as f64 / 6.0;
        if mean < 0.1 {
            return 0.0;
        }

        let variance = bins.iter()
            .map(|&x| {
                let diff = x as f64 - mean;
                diff * diff
            })
            .sum::<f64>() / 6.0;

        let std_dev = variance.sqrt();
        std_dev / mean.max(1.0) // Coefficient of variation
    }
}

impl Default for UpstreamStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upstream_stats_recording() {
        let mut stats = UpstreamStats::new();

        stats.record(Duration::from_millis(10), true);
        stats.record(Duration::from_millis(20), true);
        stats.record(Duration::from_millis(30), false);

        assert_eq!(stats.success_count, 2);
        assert_eq!(stats.error_count, 1);
        assert!(stats.p50_latency_ms() > 0.0);
        assert!(stats.error_rate() > 0.0 && stats.error_rate() < 1.0);
    }

    #[test]
    fn test_intent_gap() {
        let upstream = UpstreamState::new(
            "test".to_string(),
            "http://localhost:8080".to_string(),
            vec![Intent::new("realtime"), Intent::new("low-latency")],
            1,
        );

        let realtime_intent = Intent::new("realtime");
        let batch_intent = Intent::new("batch");

        assert_eq!(upstream.intent_gap(&realtime_intent), 0.0);
        assert_eq!(upstream.intent_gap(&batch_intent), 1.0);
    }
}
