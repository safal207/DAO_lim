//! Metamorphic Config — метаморфная конфигурация
//!
//! Конфигурация как живой организм. Плавные переходы без резких границ.

use crate::config::DaoConfig;
use std::time::{Duration, Instant};
use tracing::info;

/// Состояние метаморфозы
#[derive(Debug, Clone)]
pub enum MetamorphState {
    /// Стабильное состояние (одна конфигурация)
    Stable,
    /// В процессе трансформации
    Transforming {
        from_config: Box<DaoConfig>,
        to_config: Box<DaoConfig>,
        progress: f64, // 0.0 - 1.0
        started_at: Instant,
        duration: Duration,
    },
}

/// Metamorphic Config — управляет плавными переходами
pub struct MetamorphicConfig {
    state: parking_lot::RwLock<MetamorphState>,
}

impl MetamorphicConfig {
    pub fn new() -> Self {
        Self {
            state: parking_lot::RwLock::new(MetamorphState::Stable),
        }
    }

    /// Начать метаморфозу к новой конфигурации
    pub fn begin_transformation(&self, from: DaoConfig, to: DaoConfig, duration: Duration) {
        info!(
            "Beginning config metamorphosis over {:?}",
            duration
        );

        *self.state.write() = MetamorphState::Transforming {
            from_config: Box::new(from),
            to_config: Box::new(to),
            progress: 0.0,
            started_at: Instant::now(),
            duration,
        };
    }

    /// Обновить прогресс трансформации
    pub fn update_progress(&self) {
        let mut state = self.state.write();

        if let MetamorphState::Transforming {
            started_at,
            duration,
            progress,
            ..
        } = &mut *state
        {
            let elapsed = started_at.elapsed();
            *progress = (elapsed.as_secs_f64() / duration.as_secs_f64()).min(1.0);

            if *progress >= 1.0 {
                info!("Metamorphosis complete");
                *state = MetamorphState::Stable;
            }
        }
    }

    /// Получить текущий прогресс (0.0 - 1.0)
    pub fn progress(&self) -> f64 {
        match *self.state.read() {
            MetamorphState::Stable => 1.0,
            MetamorphState::Transforming { progress, .. } => progress,
        }
    }

    /// Находится ли конфиг в процессе трансформации?
    pub fn is_transforming(&self) -> bool {
        matches!(*self.state.read(), MetamorphState::Transforming { .. })
    }
}

impl Default for MetamorphicConfig {
    fn default() -> Self {
        Self::new()
    }
}
