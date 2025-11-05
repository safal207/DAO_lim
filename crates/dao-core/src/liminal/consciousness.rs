//! Consciousness Levels — уровни осознанности системы
//!
//! Система спит при покое, просыпается при аномалиях.
//! Адаптивная глубина анализа в зависимости от состояния.

use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{debug, info, warn};

/// Уровень осознанности системы
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConsciousnessLevel {
    /// Dormant — минимальная осознанность (round-robin)
    Dormant = 0,
    /// Aware — базовая осознанность (load balancing)
    Aware = 1,
    /// Vigilant — повышенная осознанность (resonant балансировка)
    Vigilant = 2,
    /// Transcendent — полная осознанность (ML predictions, все метрики)
    Transcendent = 3,
}

impl ConsciousnessLevel {
    /// Стоимость вычислений для уровня
    pub fn computational_cost(&self) -> f64 {
        match self {
            ConsciousnessLevel::Dormant => 1.0,
            ConsciousnessLevel::Aware => 2.0,
            ConsciousnessLevel::Vigilant => 5.0,
            ConsciousnessLevel::Transcendent => 10.0,
        }
    }

    /// Описание уровня
    pub fn description(&self) -> &'static str {
        match self {
            ConsciousnessLevel::Dormant => "Dormant - система в покое",
            ConsciousnessLevel::Aware => "Aware - базовая осознанность",
            ConsciousnessLevel::Vigilant => "Vigilant - повышенная бдительность",
            ConsciousnessLevel::Transcendent => "Transcendent - полное осознание",
        }
    }
}

/// Факторы, влияющие на уровень осознанности
#[derive(Debug, Clone)]
pub struct AwarenessFactors {
    /// Текущий RPS
    pub current_rps: f64,
    /// Средний RPS за период
    pub baseline_rps: f64,
    /// Error rate (0.0 - 1.0)
    pub error_rate: f64,
    /// Средняя латентность p95
    pub p95_latency_ms: f64,
    /// Количество аномалий за последнюю минуту
    pub anomaly_count: u32,
}

/// Оркестратор осознанности — управляет уровнем consciousness
pub struct AwarenessOrchestrator {
    current_level: Arc<RwLock<ConsciousnessLevel>>,
    config: AwarenessConfig,
}

#[derive(Debug, Clone)]
pub struct AwarenessConfig {
    /// Порог RPS для пробуждения
    pub rps_spike_threshold: f64,
    /// Порог error rate для повышения уровня
    pub error_rate_threshold: f64,
    /// Порог латентности для vigilant режима
    pub latency_threshold_ms: f64,
}

impl Default for AwarenessConfig {
    fn default() -> Self {
        Self {
            rps_spike_threshold: 2.0,  // 2x от baseline
            error_rate_threshold: 0.05, // 5% ошибок
            latency_threshold_ms: 500.0,
        }
    }
}

impl AwarenessOrchestrator {
    pub fn new(config: AwarenessConfig) -> Self {
        Self {
            current_level: Arc::new(RwLock::new(ConsciousnessLevel::Aware)),
            config,
        }
    }

    /// Получение текущего уровня осознанности
    pub fn current_level(&self) -> ConsciousnessLevel {
        *self.current_level.read()
    }

    /// Оценка необходимого уровня осознанности
    pub fn evaluate_level(&self, factors: &AwarenessFactors) -> ConsciousnessLevel {
        let mut score = 0;

        // RPS spike
        if factors.current_rps > factors.baseline_rps * self.config.rps_spike_threshold {
            score += 1;
            debug!("RPS spike detected: {} vs baseline {}", factors.current_rps, factors.baseline_rps);
        }

        // High error rate
        if factors.error_rate > self.config.error_rate_threshold {
            score += 2;
            warn!("High error rate: {:.2}%", factors.error_rate * 100.0);
        }

        // High latency
        if factors.p95_latency_ms > self.config.latency_threshold_ms {
            score += 1;
            debug!("High latency detected: {:.2}ms", factors.p95_latency_ms);
        }

        // Anomalies
        if factors.anomaly_count > 0 {
            score += factors.anomaly_count as i32;
            warn!("Anomalies detected: {}", factors.anomaly_count);
        }

        // Определение уровня на основе score
        match score {
            0..=1 => ConsciousnessLevel::Dormant,
            2..=3 => ConsciousnessLevel::Aware,
            4..=5 => ConsciousnessLevel::Vigilant,
            _ => ConsciousnessLevel::Transcendent,
        }
    }

    /// Обновление уровня осознанности
    pub fn update_level(&self, factors: &AwarenessFactors) {
        let new_level = self.evaluate_level(factors);
        let current = self.current_level();

        if new_level != current {
            info!(
                "Consciousness level transition: {:?} → {:?} ({})",
                current,
                new_level,
                new_level.description()
            );
            *self.current_level.write() = new_level;
        }
    }

    /// Ручная установка уровня
    pub fn set_level(&self, level: ConsciousnessLevel) {
        info!("Manually setting consciousness level to: {:?}", level);
        *self.current_level.write() = level;
    }
}

impl Default for AwarenessOrchestrator {
    fn default() -> Self {
        Self::new(AwarenessConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consciousness_levels() {
        let orchestrator = AwarenessOrchestrator::default();

        // Нормальное состояние
        let normal_factors = AwarenessFactors {
            current_rps: 100.0,
            baseline_rps: 100.0,
            error_rate: 0.01,
            p95_latency_ms: 50.0,
            anomaly_count: 0,
        };

        let level = orchestrator.evaluate_level(&normal_factors);
        assert_eq!(level, ConsciousnessLevel::Dormant);

        // Высокая нагрузка
        let high_load = AwarenessFactors {
            current_rps: 500.0,
            baseline_rps: 100.0,
            error_rate: 0.10,
            p95_latency_ms: 800.0,
            anomaly_count: 2,
        };

        let level = orchestrator.evaluate_level(&high_load);
        assert!(level >= ConsciousnessLevel::Vigilant);
    }

    #[test]
    fn test_computational_cost() {
        assert!(ConsciousnessLevel::Transcendent.computational_cost()
                > ConsciousnessLevel::Dormant.computational_cost());
    }
}
