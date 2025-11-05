//! Echo Analysis — анализ отголосков
//!
//! Каждый запрос оставляет эхо в системе.
//! Детекция аномалий через паттерны и отклонения от эха прошлого.

use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;
use tracing::warn;

/// Эхо запроса — сжатый отпечаток
#[derive(Debug, Clone)]
pub struct RequestEcho {
    /// URI path (хешированный для экономии памяти)
    pub path_hash: u64,
    /// HTTP method
    pub method: String,
    /// Response status
    pub status: u16,
    /// Latency в миллисекундах
    pub latency_ms: f64,
    /// Размер ответа
    pub response_size: u64,
    /// Timestamp
    pub timestamp: std::time::Instant,
}

impl RequestEcho {
    /// Вычислить "расстояние" между двумя эхо
    pub fn distance(&self, other: &RequestEcho) -> f64 {
        let mut dist = 0.0;

        // Path similarity (0 если совпадают, 1 если разные)
        if self.path_hash != other.path_hash {
            dist += 1.0;
        }

        // Method similarity
        if self.method != other.method {
            dist += 0.5;
        }

        // Status similarity (группы: 2xx, 3xx, 4xx, 5xx)
        let self_status_group = self.status / 100;
        let other_status_group = other.status / 100;
        if self_status_group != other_status_group {
            dist += 2.0;
        }

        // Latency difference (нормализованная)
        let latency_diff = (self.latency_ms - other.latency_ms).abs() / self.latency_ms.max(1.0);
        dist += latency_diff;

        // Size difference (нормализованная)
        let size_diff = (self.response_size as f64 - other.response_size as f64).abs()
            / self.response_size.max(1) as f64;
        dist += size_diff * 0.5;

        dist
    }
}

/// Echo Analyzer — детектор аномалий через эхо
pub struct EchoAnalyzer {
    /// Кольцевой буфер эхо
    echo_buffer: Arc<RwLock<VecDeque<RequestEcho>>>,
    /// Размер буфера
    buffer_size: usize,
    /// Порог аномальности (в сигмах)
    anomaly_threshold: f64,
}

impl EchoAnalyzer {
    pub fn new(buffer_size: usize, anomaly_threshold: f64) -> Self {
        Self {
            echo_buffer: Arc::new(RwLock::new(VecDeque::with_capacity(buffer_size))),
            buffer_size,
            anomaly_threshold,
        }
    }

    /// Записать новое эхо
    pub fn record_echo(&self, echo: RequestEcho) {
        let mut buffer = self.echo_buffer.write();

        if buffer.len() >= self.buffer_size {
            buffer.pop_front();
        }

        buffer.push_back(echo);
    }

    /// Проверить, является ли запрос аномальным
    pub fn is_anomaly(&self, current: &RequestEcho) -> bool {
        let buffer = self.echo_buffer.read();

        if buffer.len() < 10 {
            return false; // Недостаточно данных
        }

        // Вычисляем среднее расстояние до всех эхо в буфере
        let distances: Vec<f64> = buffer.iter().map(|echo| current.distance(echo)).collect();

        let mean = distances.iter().sum::<f64>() / distances.len() as f64;

        // Стандартное отклонение
        let variance = distances
            .iter()
            .map(|d| {
                let diff = d - mean;
                diff * diff
            })
            .sum::<f64>()
            / distances.len() as f64;

        let std_dev = variance.sqrt();

        // Проверка на аномальность (за пределами N сигм)
        let z_score = if std_dev > 0.001 {
            (mean / std_dev).abs()
        } else {
            0.0
        };

        let is_anomaly = z_score > self.anomaly_threshold;

        if is_anomaly {
            warn!(
                "Anomaly detected: z-score={:.2}, mean_distance={:.2}",
                z_score, mean
            );
        }

        is_anomaly
    }

    /// Получить статистику эхо
    pub fn statistics(&self) -> EchoStatistics {
        let buffer = self.echo_buffer.read();

        let total_count = buffer.len();
        let avg_latency = if total_count > 0 {
            buffer.iter().map(|e| e.latency_ms).sum::<f64>() / total_count as f64
        } else {
            0.0
        };

        let status_distribution = Self::calculate_status_distribution(&buffer);

        EchoStatistics {
            total_count,
            avg_latency,
            status_distribution,
        }
    }

    fn calculate_status_distribution(buffer: &VecDeque<RequestEcho>) -> std::collections::HashMap<u16, usize> {
        let mut dist = std::collections::HashMap::new();
        for echo in buffer {
            *dist.entry(echo.status / 100 * 100).or_insert(0) += 1;
        }
        dist
    }

    /// Очистить буфер
    pub fn clear(&self) {
        self.echo_buffer.write().clear();
    }
}

impl Default for EchoAnalyzer {
    fn default() -> Self {
        Self::new(1000, 3.0) // 1000 эхо, 3 сигмы
    }
}

/// Статистика эхо
#[derive(Debug, Clone)]
pub struct EchoStatistics {
    pub total_count: usize,
    pub avg_latency: f64,
    pub status_distribution: std::collections::HashMap<u16, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_echo_distance() {
        let echo1 = RequestEcho {
            path_hash: 123,
            method: "GET".to_string(),
            status: 200,
            latency_ms: 50.0,
            response_size: 1000,
            timestamp: std::time::Instant::now(),
        };

        let echo2 = RequestEcho {
            path_hash: 123,
            method: "GET".to_string(),
            status: 200,
            latency_ms: 55.0,
            response_size: 1100,
            timestamp: std::time::Instant::now(),
        };

        let distance = echo1.distance(&echo2);
        assert!(distance < 1.0); // Похожие запросы

        let echo3 = RequestEcho {
            path_hash: 456,
            method: "POST".to_string(),
            status: 500,
            latency_ms: 500.0,
            response_size: 100,
            timestamp: std::time::Instant::now(),
        };

        let distance2 = echo1.distance(&echo3);
        assert!(distance2 > 2.0); // Очень разные запросы
    }

    #[test]
    fn test_echo_analyzer() {
        let analyzer = EchoAnalyzer::new(10, 3.0);

        // Записываем несколько похожих эхо
        for i in 0..10 {
            analyzer.record_echo(RequestEcho {
                path_hash: 123,
                method: "GET".to_string(),
                status: 200,
                latency_ms: 50.0 + i as f64,
                response_size: 1000,
                timestamp: std::time::Instant::now(),
            });
        }

        // Аномальный запрос
        let anomaly = RequestEcho {
            path_hash: 999,
            method: "POST".to_string(),
            status: 500,
            latency_ms: 5000.0,
            response_size: 10,
            timestamp: std::time::Instant::now(),
        };

        assert!(analyzer.is_anomaly(&anomaly));
    }
}
