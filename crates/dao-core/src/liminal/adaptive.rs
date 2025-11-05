//! Adaptive Thresholds — самообучающиеся пороги
//!
//! Пороги сами учатся, где проходит граница между нормой и аномалией.

use parking_lot::RwLock;
use std::collections::VecDeque;

/// Адаптивный порог
pub struct AdaptiveThreshold {
    /// История значений
    history: RwLock<VecDeque<f64>>,
    /// Размер окна истории
    window_size: usize,
    /// Множитель для порога (в сигмах)
    sigma_multiplier: f64,
}

impl AdaptiveThreshold {
    pub fn new(window_size: usize, sigma_multiplier: f64) -> Self {
        Self {
            history: RwLock::new(VecDeque::with_capacity(window_size)),
            window_size,
            sigma_multiplier,
        }
    }

    /// Записать новое значение
    pub fn record(&self, value: f64) {
        let mut history = self.history.write();
        if history.len() >= self.window_size {
            history.pop_front();
        }
        history.push_back(value);
    }

    /// Текущий порог
    pub fn current_threshold(&self) -> f64 {
        let history = self.history.read();
        if history.len() < 2 {
            return 100.0; // Дефолтный порог
        }

        let mean = history.iter().sum::<f64>() / history.len() as f64;
        let variance = history
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>()
            / history.len() as f64;
        let std_dev = variance.sqrt();

        mean + self.sigma_multiplier * std_dev
    }

    /// Превышает ли значение порог?
    pub fn exceeds(&self, value: f64) -> bool {
        value > self.current_threshold()
    }
}

/// Адаптивные пороги для различных метрик
pub struct AdaptiveThresholds {
    pub rate_limit: AdaptiveThreshold,
    pub error_rate: AdaptiveThreshold,
    pub latency: AdaptiveThreshold,
}

impl AdaptiveThresholds {
    pub fn new() -> Self {
        Self {
            rate_limit: AdaptiveThreshold::new(1000, 2.0),
            error_rate: AdaptiveThreshold::new(500, 3.0),
            latency: AdaptiveThreshold::new(1000, 2.5),
        }
    }

    /// Обновить все пороги
    pub fn update(&self, rps: f64, error_rate: f64, latency_ms: f64) {
        self.rate_limit.record(rps);
        self.error_rate.record(error_rate);
        self.latency.record(latency_ms);
    }
}

impl Default for AdaptiveThresholds {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_threshold() {
        let threshold = AdaptiveThreshold::new(10, 2.0);

        // Записываем нормальные значения
        for i in 0..10 {
            threshold.record(100.0 + i as f64);
        }

        let current = threshold.current_threshold();
        assert!(current > 100.0);

        // Нормальное значение не должно превышать
        assert!(!threshold.exceeds(105.0));

        // Аномальное значение должно превышать
        assert!(threshold.exceeds(200.0));
    }
}
