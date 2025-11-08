//! Presence Detection — детекция присутствия/отсутствия
//!
//! Понимание того, что upstream "не отвечает" vs "отсутствует".
//! Различие между "медленным ответом" и "отсутствием".

use std::time::{Duration, Instant};
use parking_lot::RwLock;
use tracing::info;

/// Состояние присутствия upstream
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresenceState {
    /// Присутствует и отвечает стабильно
    Present,
    /// Лиминальное состояние - иногда отвечает, иногда нет (flickering)
    Liminal,
    /// Явно недоступен
    Absent,
    /// Неопределённое состояние (недостаточно данных)
    Unknown,
}

impl PresenceState {
    pub fn description(&self) -> &'static str {
        match self {
            PresenceState::Present => "Upstream is present and stable",
            PresenceState::Liminal => "Upstream is flickering (liminal state)",
            PresenceState::Absent => "Upstream is absent",
            PresenceState::Unknown => "Upstream state is unknown",
        }
    }

    /// Можно ли отправлять трафик?
    pub fn can_send_traffic(&self) -> bool {
        matches!(self, PresenceState::Present | PresenceState::Liminal)
    }
}

/// История проверок присутствия
#[derive(Debug, Clone)]
struct PresenceHistory {
    /// Последние N проверок (true = success, false = failure)
    checks: Vec<bool>,
    /// Последняя успешная проверка
    last_success: Option<Instant>,
    /// Последняя неудачная проверка
    last_failure: Option<Instant>,
}

/// Presence Detector — детектор состояния upstream
#[derive(Debug)]
pub struct PresenceDetector {
    state: RwLock<PresenceState>,
    history: RwLock<PresenceHistory>,
    config: PresenceConfig,
}

#[derive(Debug, Clone)]
pub struct PresenceConfig {
    /// Размер окна истории
    pub history_size: usize,
    /// Порог для Present (процент успешных проверок)
    pub present_threshold: f64,
    /// Порог для Liminal
    pub liminal_threshold: f64,
    /// Таймаут для перехода в Absent
    pub absent_timeout: Duration,
}

impl Default for PresenceConfig {
    fn default() -> Self {
        Self {
            history_size: 10,
            present_threshold: 0.8,     // 80% success
            liminal_threshold: 0.3,     // 30% success
            absent_timeout: Duration::from_secs(30),
        }
    }
}

impl PresenceDetector {
    pub fn new(config: PresenceConfig) -> Self {
        Self {
            state: RwLock::new(PresenceState::Unknown),
            history: RwLock::new(PresenceHistory {
                checks: Vec::with_capacity(config.history_size),
                last_success: None,
                last_failure: None,
            }),
            config,
        }
    }

    /// Записать результат проверки
    pub fn record_check(&self, success: bool) {
        let mut history = self.history.write();

        // Обновляем историю
        if history.checks.len() >= self.config.history_size {
            history.checks.remove(0);
        }
        history.checks.push(success);

        // Обновляем timestamps
        let now = Instant::now();
        if success {
            history.last_success = Some(now);
        } else {
            history.last_failure = Some(now);
        }

        drop(history);

        // Обновляем состояние
        self.update_state();
    }

    /// Обновить состояние на основе истории
    fn update_state(&self) {
        let history = self.history.read();

        if history.checks.is_empty() {
            return;
        }

        let success_rate = history.checks.iter().filter(|&&s| s).count() as f64
            / history.checks.len() as f64;

        let new_state = if success_rate >= self.config.present_threshold {
            PresenceState::Present
        } else if success_rate >= self.config.liminal_threshold {
            PresenceState::Liminal
        } else {
            // Проверяем таймаут
            if let Some(last_success) = history.last_success {
                if last_success.elapsed() > self.config.absent_timeout {
                    PresenceState::Absent
                } else {
                    PresenceState::Liminal
                }
            } else {
                PresenceState::Absent
            }
        };

        let old_state = *self.state.read();
        if new_state != old_state {
            info!(
                "Presence state transition: {:?} → {:?}",
                old_state, new_state
            );
            *self.state.write() = new_state;
        }
    }

    /// Текущее состояние
    pub fn current_state(&self) -> PresenceState {
        *self.state.read()
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        let history = self.history.read();
        if history.checks.is_empty() {
            return 0.0;
        }

        history.checks.iter().filter(|&&s| s).count() as f64 / history.checks.len() as f64
    }
}

impl Default for PresenceDetector {
    fn default() -> Self {
        Self::new(PresenceConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_detection() {
        let detector = PresenceDetector::default();

        // Несколько успешных проверок
        for _ in 0..10 {
            detector.record_check(true);
        }

        assert_eq!(detector.current_state(), PresenceState::Present);

        // Начинают появляться ошибки
        for _ in 0..5 {
            detector.record_check(false);
        }

        assert_eq!(detector.current_state(), PresenceState::Liminal);

        // Все проверки неудачны
        for _ in 0..10 {
            detector.record_check(false);
        }

        assert_eq!(detector.current_state(), PresenceState::Absent);
    }

    #[test]
    fn test_success_rate() {
        let detector = PresenceDetector::default();

        detector.record_check(true);
        detector.record_check(true);
        detector.record_check(false);

        assert!((detector.success_rate() - 0.666).abs() < 0.01);
    }
}
