//! Liminal Zones — промежуточные ответы
//!
//! Пространство между запросом и ответом.
//! Graceful degradation через промежуточные ответы вместо резкого таймаута.

use hyper::{Response, StatusCode, body::Bytes};
use http_body_util::{BodyExt, Full};
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Конфигурация лиминальной зоны
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiminalZoneConfig {
    /// Время срабатывания промежуточного ответа
    pub at: Duration,
    /// HTTP status code
    pub status: u16,
    /// Тело ответа
    pub body: String,
    /// Заголовки
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
}

/// Liminal Zones — управляет промежуточными ответами
pub struct LiminalZones {
    zones: Vec<LiminalZoneConfig>,
}

impl LiminalZones {
    pub fn new(zones: Vec<LiminalZoneConfig>) -> Self {
        Self { zones }
    }

    /// Получить промежуточный ответ для конкретного таймаута
    pub fn get_response_for_duration(
        &self,
        elapsed: Duration,
    ) -> Option<Response<Full<Bytes>>> {
        // Найти подходящую зону
        let zone = self
            .zones
            .iter()
            .filter(|z| elapsed >= z.at)
            .max_by_key(|z| z.at)?;

        Some(self.build_response(zone))
    }

    /// Построить HTTP response из конфигурации зоны
    fn build_response(&self, zone: &LiminalZoneConfig) -> Response<Full<Bytes>> {
        let mut builder = Response::builder().status(
            StatusCode::from_u16(zone.status).unwrap_or(StatusCode::ACCEPTED),
        );

        // Добавляем заголовки
        for (key, value) in &zone.headers {
            builder = builder.header(key, value);
        }

        // Добавляем маркер лиминальности
        builder = builder.header("X-DAO-Liminal", "true");
        builder = builder.header("X-DAO-Zone-At", format!("{}ms", zone.at.as_millis()));

        builder
            .body(Full::new(Bytes::from(zone.body.clone())))
            .unwrap()
    }

    /// Проверить, есть ли зона для данного elapsed времени
    pub fn has_zone_for(&self, elapsed: Duration) -> bool {
        self.zones.iter().any(|z| elapsed >= z.at)
    }

    /// Получить все зоны
    pub fn zones(&self) -> &[LiminalZoneConfig] {
        &self.zones
    }
}

impl Default for LiminalZones {
    fn default() -> Self {
        Self::new(vec![
            // 100ms - Processing
            LiminalZoneConfig {
                at: Duration::from_millis(100),
                status: 202,
                body: r#"{"status":"processing","message":"Request is being processed"}"#
                    .to_string(),
                headers: [("Content-Type".to_string(), "application/json".to_string())]
                    .into_iter()
                    .collect(),
            },
            // 500ms - Still processing
            LiminalZoneConfig {
                at: Duration::from_millis(500),
                status: 206,
                body: r#"{"status":"partial","message":"Partial results available"}"#.to_string(),
                headers: [("Content-Type".to_string(), "application/json".to_string())]
                    .into_iter()
                    .collect(),
            },
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_liminal_zones_default() {
        let zones = LiminalZones::default();
        assert_eq!(zones.zones().len(), 2);
    }

    #[test]
    fn test_zone_selection() {
        let zones = LiminalZones::default();

        // До первой зоны
        assert!(!zones.has_zone_for(Duration::from_millis(50)));

        // В первой зоне
        assert!(zones.has_zone_for(Duration::from_millis(150)));

        // Во второй зоне
        assert!(zones.has_zone_for(Duration::from_millis(600)));
    }

    #[test]
    fn test_response_building() {
        let zones = LiminalZones::default();

        let response = zones
            .get_response_for_duration(Duration::from_millis(150))
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
        assert!(response.headers().contains_key("X-DAO-Liminal"));
    }
}
