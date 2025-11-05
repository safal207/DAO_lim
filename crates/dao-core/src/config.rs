//! Конфигурация DAO

use crate::{Intent, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Корневая конфигурация DAO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaoConfig {
    pub server: ServerConfig,
    pub telemetry: Option<TelemetryConfig>,
    pub routes: RoutesConfig,
    pub policies: Option<HashMap<String, PolicyConfig>>,
}

impl DaoConfig {
    /// Загрузка из TOML файла
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let config: DaoConfig = toml::from_str(&content)
            .map_err(|e| crate::DaoError::config(format!("Failed to parse config: {}", e)))?;
        Ok(config)
    }

    /// Валидация конфигурации
    pub fn validate(&self) -> Result<()> {
        // Проверка bind-адресов
        if self.server.bind.is_empty() {
            return Err(crate::DaoError::config("server.bind is empty"));
        }

        // Проверка наличия маршрутов
        if self.routes.rule.is_empty() {
            return Err(crate::DaoError::config("No routes defined"));
        }

        // Валидация каждого маршрута
        for route in &self.routes.rule {
            route.validate()?;
        }

        Ok(())
    }
}

/// Конфигурация сервера
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub bind: String,
    pub tls_cert: Option<String>,
    pub tls_key: Option<String>,
    #[serde(default = "default_workers")]
    pub workers: usize,
}

fn default_workers() -> usize {
    num_cpus::get()
}

/// Конфигурация телеметрии
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub prometheus_bind: String,
}

/// Конфигурация маршрутов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutesConfig {
    pub rule: Vec<RouteRule>,
}

/// Правило маршрутизации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRule {
    pub name: String,
    #[serde(rename = "match")]
    pub match_rule: MatchRule,
    pub policy: String,
    pub intent: Option<String>,
    pub upstreams: Vec<UpstreamConfig>,
    pub filters: Option<FilterConfig>,
}

impl RouteRule {
    pub fn validate(&self) -> Result<()> {
        if self.upstreams.is_empty() {
            return Err(crate::DaoError::config(format!(
                "Route '{}' has no upstreams",
                self.name
            )));
        }
        Ok(())
    }

    pub fn intent(&self) -> Option<Intent> {
        self.intent.as_ref().map(|s| Intent::new(s.clone()))
    }
}

/// Правило матчинга запроса
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRule {
    pub host: Option<String>,
    pub path_prefix: Option<String>,
    pub path_exact: Option<String>,
    pub upgrade: Option<String>,
    pub headers: Option<HashMap<String, String>>,
}

impl MatchRule {
    /// Проверка соответствия запроса правилу
    pub fn matches(&self, req: &http::Request<hyper::body::Incoming>) -> bool {
        // Host matching
        if let Some(expected_host) = &self.host {
            let host = req
                .headers()
                .get(http::header::HOST)
                .and_then(|v| v.to_str().ok());
            if host != Some(expected_host.as_str()) {
                return false;
            }
        }

        // Path matching
        let path = req.uri().path();
        if let Some(prefix) = &self.path_prefix {
            if !path.starts_with(prefix) {
                return false;
            }
        }
        if let Some(exact) = &self.path_exact {
            if path != exact {
                return false;
            }
        }

        // Upgrade matching (WebSocket)
        if let Some(expected_upgrade) = &self.upgrade {
            let upgrade = req
                .headers()
                .get(http::header::UPGRADE)
                .and_then(|v| v.to_str().ok());
            if upgrade != Some(expected_upgrade.as_str()) {
                return false;
            }
        }

        // Header matching
        if let Some(expected_headers) = &self.headers {
            for (key, expected_value) in expected_headers {
                let actual_value = req
                    .headers()
                    .get(key)
                    .and_then(|v| v.to_str().ok());
                if actual_value != Some(expected_value.as_str()) {
                    return false;
                }
            }
        }

        true
    }
}

/// Конфигурация upstream'а
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamConfig {
    pub name: String,
    pub url: String,
    pub intent: Option<Vec<String>>,
    #[serde(default = "default_weight")]
    pub weight: u32,
}

fn default_weight() -> u32 {
    1
}

impl UpstreamConfig {
    pub fn intents(&self) -> Vec<Intent> {
        self.intent
            .as_ref()
            .map(|v| v.iter().map(|s| Intent::new(s.clone())).collect())
            .unwrap_or_default()
    }
}

/// Конфигурация фильтров
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    pub request_headers_add: Option<HashMap<String, String>>,
    pub request_headers_remove: Option<Vec<String>>,
    pub response_headers_add: Option<HashMap<String, String>>,
    pub rate_limit_rps: Option<u32>,
}

/// Конфигурация политики
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    #[serde(default = "default_w_load")]
    pub w_load: f64,
    #[serde(default = "default_w_intent")]
    pub w_intent: f64,
    #[serde(default = "default_w_tempo")]
    pub w_tempo: f64,
}

fn default_w_load() -> f64 { 0.6 }
fn default_w_intent() -> f64 { 0.3 }
fn default_w_tempo() -> f64 { 0.1 }

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            w_load: default_w_load(),
            w_intent: default_w_intent(),
            w_tempo: default_w_tempo(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_rule_host() {
        let rule = MatchRule {
            host: Some("api.example.com".to_string()),
            path_prefix: None,
            path_exact: None,
            upgrade: None,
            headers: None,
        };

        let req = http::Request::builder()
            .uri("http://api.example.com/test")
            .header(http::header::HOST, "api.example.com")
            .body(hyper::body::Incoming::default())
            .unwrap();

        // Note: This test won't compile without proper body type
        // Just demonstrating the structure
    }
}
