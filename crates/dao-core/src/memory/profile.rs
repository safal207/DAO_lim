//! Service profiles — профили сервисов

use crate::Intent;
use std::time::SystemTime;

/// Профиль сервиса — память о том, какой трафик "полезен"/"вреден"
#[derive(Debug, Clone)]
pub struct ServiceProfile {
    pub service_name: String,
    pub preferred_intents: Vec<Intent>,
    pub forbidden_intents: Vec<Intent>,
    pub optimal_rps_range: Option<(f64, f64)>,
    pub max_acceptable_latency_ms: Option<f64>,
    pub last_updated: SystemTime,
}

impl ServiceProfile {
    pub fn new(service_name: String) -> Self {
        Self {
            service_name,
            preferred_intents: Vec::new(),
            forbidden_intents: Vec::new(),
            optimal_rps_range: None,
            max_acceptable_latency_ms: None,
            last_updated: SystemTime::now(),
        }
    }

    /// Проверка, подходит ли intent для этого сервиса
    pub fn accepts_intent(&self, intent: &Intent) -> bool {
        // Запрещенные intent'ы имеют приоритет
        if self.forbidden_intents.iter().any(|i| i.matches(intent)) {
            return false;
        }

        // Если нет предпочтений, разрешаем все
        if self.preferred_intents.is_empty() {
            return true;
        }

        // Проверка совпадения с предпочтительными
        self.preferred_intents.iter().any(|i| i.matches(intent))
    }

    /// Обновление профиля на основе наблюдений
    pub fn learn_from_observation(
        &mut self,
        intent: &Intent,
        rps: f64,
        latency_ms: f64,
        success: bool,
    ) {
        if !success {
            // Добавление в forbidden, если вызывает ошибки
            if !self.forbidden_intents.contains(intent) {
                self.forbidden_intents.push(intent.clone());
            }
        } else {
            // Обновление preferred, если успешен
            if !self.preferred_intents.contains(intent) {
                self.preferred_intents.push(intent.clone());
            }

            // Обновление оптимального RPS range
            if let Some((min, max)) = self.optimal_rps_range {
                self.optimal_rps_range = Some((min.min(rps), max.max(rps)));
            } else {
                self.optimal_rps_range = Some((rps, rps));
            }

            // Обновление max latency
            if let Some(max_lat) = self.max_acceptable_latency_ms {
                self.max_acceptable_latency_ms = Some(max_lat.max(latency_ms));
            } else {
                self.max_acceptable_latency_ms = Some(latency_ms);
            }
        }

        self.last_updated = SystemTime::now();
    }
}
