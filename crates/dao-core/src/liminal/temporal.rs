//! Temporal Resonance — временной резонанс
//!
//! Память о ритмах времени. Система знает, "как было вчера в это время".
//! Предсказание будущего на основе циклов прошлого.

use chrono::{DateTime, Datelike, Local, Timelike, Weekday};
use std::collections::HashMap;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::debug;

/// Временной профиль нагрузки
#[derive(Debug, Clone, PartialEq)]
pub enum TemporalProfile {
    /// Низкая нагрузка
    Low,
    /// Средняя нагрузка
    Medium,
    /// Высокая нагрузка
    High,
    /// Пиковая нагрузка
    Peak,
}

impl TemporalProfile {
    /// Ожидаемый множитель RPS для профиля
    pub fn expected_multiplier(&self) -> f64 {
        match self {
            TemporalProfile::Low => 0.3,
            TemporalProfile::Medium => 1.0,
            TemporalProfile::High => 2.0,
            TemporalProfile::Peak => 5.0,
        }
    }
}

/// Временной ключ (день недели + час)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TemporalKey {
    pub weekday: Weekday,
    pub hour: u32,
}

impl TemporalKey {
    pub fn from_datetime(dt: &DateTime<Local>) -> Self {
        Self {
            weekday: dt.weekday(),
            hour: dt.hour(),
        }
    }

    pub fn now() -> Self {
        Self::from_datetime(&Local::now())
    }
}

/// Temporal Resonance — хранит память о временных паттернах
pub struct TemporalResonance {
    /// Профили по времени (день недели + час)
    profiles: Arc<RwLock<HashMap<TemporalKey, TemporalProfile>>>,
    /// История RPS для обучения
    history: Arc<RwLock<Vec<TemporalObservation>>>,
}

#[derive(Debug, Clone)]
pub struct TemporalObservation {
    pub timestamp: DateTime<Local>,
    pub rps: f64,
    pub error_rate: f64,
    pub p95_latency: f64,
}

impl TemporalResonance {
    pub fn new() -> Self {
        Self {
            profiles: Arc::new(RwLock::new(Self::default_profiles())),
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Дефолтные профили (эвристика)
    fn default_profiles() -> HashMap<TemporalKey, TemporalProfile> {
        let mut profiles = HashMap::new();

        // Понедельник утро - пик
        for hour in 9..12 {
            profiles.insert(
                TemporalKey {
                    weekday: Weekday::Mon,
                    hour,
                },
                TemporalProfile::Peak,
            );
        }

        // Выходные ночь - низкая нагрузка
        for &weekday in &[Weekday::Sat, Weekday::Sun] {
            for hour in 0..6 {
                profiles.insert(
                    TemporalKey { weekday, hour },
                    TemporalProfile::Low,
                );
            }
        }

        profiles
    }

    /// Получить профиль для текущего времени
    pub fn current_profile(&self) -> TemporalProfile {
        self.profile_for_time(&Local::now())
    }

    /// Получить профиль для конкретного времени
    pub fn profile_for_time(&self, dt: &DateTime<Local>) -> TemporalProfile {
        let key = TemporalKey::from_datetime(dt);
        self.profiles
            .read()
            .get(&key)
            .cloned()
            .unwrap_or(TemporalProfile::Medium)
    }

    /// Предсказать профиль на N часов вперёд
    pub fn predict_profile(&self, hours_ahead: u32) -> TemporalProfile {
        let future = Local::now() + chrono::Duration::hours(hours_ahead as i64);
        self.profile_for_time(&future)
    }

    /// Записать наблюдение для обучения
    pub fn record_observation(&self, obs: TemporalObservation) {
        let mut history = self.history.write();
        history.push(obs.clone());

        // Ограничиваем размер истории
        const MAX_HISTORY: usize = 10080; // Неделя с записью раз в минуту
        if history.len() > MAX_HISTORY {
            let excess = history.len() - MAX_HISTORY;
            history.drain(0..excess);
        }

        // Обновляем профили на основе истории
        self.update_profiles_from_history();
    }

    /// Обновление профилей на основе истории
    fn update_profiles_from_history(&self) {
        let history = self.history.read();
        if history.len() < 100 {
            return; // Недостаточно данных
        }

        // Группируем по TemporalKey и вычисляем средний RPS
        let mut grouped: HashMap<TemporalKey, Vec<f64>> = HashMap::new();

        for obs in history.iter() {
            let key = TemporalKey::from_datetime(&obs.timestamp);
            grouped.entry(key).or_insert_with(Vec::new).push(obs.rps);
        }

        // Обновляем профили
        let mut profiles = self.profiles.write();
        for (key, rps_values) in grouped {
            let avg_rps: f64 = rps_values.iter().sum::<f64>() / rps_values.len() as f64;

            let profile = if avg_rps < 50.0 {
                TemporalProfile::Low
            } else if avg_rps < 200.0 {
                TemporalProfile::Medium
            } else if avg_rps < 500.0 {
                TemporalProfile::High
            } else {
                TemporalProfile::Peak
            };

            profiles.insert(key, profile);
        }

        debug!("Updated temporal profiles from {} observations", history.len());
    }

    /// Резонанс: насколько текущая нагрузка соответствует ожиданиям?
    pub fn resonance_score(&self, current_rps: f64) -> f64 {
        let expected_profile = self.current_profile();
        let expected_multiplier = expected_profile.expected_multiplier();

        // Базовый RPS (можно настроить)
        let baseline_rps = 100.0;
        let expected_rps = baseline_rps * expected_multiplier;

        // Отклонение от ожидаемого
        let deviation = (current_rps - expected_rps).abs() / expected_rps.max(1.0);

        // Резонанс: 1.0 = идеальное совпадение, 0.0 = большое отклонение
        (1.0 - deviation).max(0.0)
    }
}

impl Default for TemporalResonance {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporal_key() {
        let dt = Local::now();
        let key = TemporalKey::from_datetime(&dt);

        assert_eq!(key.weekday, dt.weekday());
        assert_eq!(key.hour, dt.hour());
    }

    #[test]
    fn test_profile_prediction() {
        let resonance = TemporalResonance::new();

        // Текущий профиль
        let current = resonance.current_profile();
        assert!(matches!(
            current,
            TemporalProfile::Low
                | TemporalProfile::Medium
                | TemporalProfile::High
                | TemporalProfile::Peak
        ));

        // Предсказание на час вперёд
        let future = resonance.predict_profile(1);
        assert!(matches!(
            future,
            TemporalProfile::Low
                | TemporalProfile::Medium
                | TemporalProfile::High
                | TemporalProfile::Peak
        ));
    }

    #[test]
    fn test_resonance_score() {
        let resonance = TemporalResonance::new();

        // Идеальный резонанс при ожидаемой нагрузке
        let score = resonance.resonance_score(100.0);
        assert!(score >= 0.0 && score <= 1.0);
    }
}
