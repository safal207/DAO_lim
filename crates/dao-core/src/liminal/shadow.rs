//! Shadow Traffic — теневое дублирование запросов
//!
//! Запросы-призраки, существующие между реальностью и тестом.
//! Идеально для безопасного тестирования новых версий.

use crate::{Result, upstream::UpstreamState};
use hyper::{Request, body::Incoming};
use std::sync::Arc;
use tracing::{debug, warn};

/// Режим теневого трафика
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShadowMode {
    /// Асинхронное дублирование (fire and forget)
    Async,
    /// Синхронное дублирование (ждём ответа, но игнорируем)
    Sync,
    /// Сравнение ответов (для валидации)
    Compare,
}

/// Конфигурация теневого трафика
#[derive(Debug, Clone)]
pub struct ShadowConfig {
    /// К какому upstream дублировать
    pub shadow_upstream: String,
    /// Процент трафика для дублирования (0.0 - 1.0)
    pub shadow_rate: f64,
    /// Режим дублирования
    pub mode: ShadowMode,
}

/// Shadow Traffic orchestrator
pub struct ShadowTraffic {
    config: Arc<ShadowConfig>,
}

impl ShadowTraffic {
    pub fn new(config: ShadowConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Должен ли этот запрос быть продублирован?
    pub fn should_shadow(&self) -> bool {
        if self.config.shadow_rate >= 1.0 {
            return true;
        }
        if self.config.shadow_rate <= 0.0 {
            return false;
        }

        // Вероятностная выборка
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen::<f64>() < self.config.shadow_rate
    }

    /// Дублирование запроса к теневому upstream
    pub async fn shadow_request(
        &self,
        _original_req: &Request<Incoming>,
        _shadow_upstream: Arc<UpstreamState>,
    ) -> Result<()> {
        debug!(
            "Shadowing request to {} (mode: {:?})",
            self.config.shadow_upstream, self.config.mode
        );

        match self.config.mode {
            ShadowMode::Async => {
                // Fire and forget - запускаем в фоне
                // TODO: реальное клонирование и отправка запроса
                tokio::spawn(async move {
                    debug!("Shadow request sent (async)");
                });
            }
            ShadowMode::Sync => {
                // Ждём ответа, но игнорируем результат
                // TODO: отправка и ожидание
                debug!("Shadow request sent (sync)");
            }
            ShadowMode::Compare => {
                // Сравниваем ответы
                // TODO: сравнение результатов
                debug!("Shadow request sent (compare mode)");
            }
        }

        Ok(())
    }

    /// Конфигурация
    pub fn config(&self) -> &ShadowConfig {
        &self.config
    }
}

impl Default for ShadowTraffic {
    fn default() -> Self {
        Self::new(ShadowConfig {
            shadow_upstream: String::new(),
            shadow_rate: 0.0,
            mode: ShadowMode::Async,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shadow_probability() {
        let config = ShadowConfig {
            shadow_upstream: "test".to_string(),
            shadow_rate: 0.5,
            mode: ShadowMode::Async,
        };

        let shadow = ShadowTraffic::new(config);

        // Проверяем, что вероятность примерно 50%
        let mut count = 0;
        for _ in 0..1000 {
            if shadow.should_shadow() {
                count += 1;
            }
        }

        // Должно быть примерно 500 ± 100
        assert!(count > 400 && count < 600, "Shadow rate check failed: {}", count);
    }

    #[test]
    fn test_shadow_always() {
        let config = ShadowConfig {
            shadow_upstream: "test".to_string(),
            shadow_rate: 1.0,
            mode: ShadowMode::Async,
        };

        let shadow = ShadowTraffic::new(config);
        assert!(shadow.should_shadow());
    }

    #[test]
    fn test_shadow_never() {
        let config = ShadowConfig {
            shadow_upstream: "test".to_string(),
            shadow_rate: 0.0,
            mode: ShadowMode::Async,
        };

        let shadow = ShadowTraffic::new(config);
        assert!(!shadow.should_shadow());
    }
}
