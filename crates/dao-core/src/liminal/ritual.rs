//! Ritual Protocols — протоколы ритуалов
//!
//! Переходные церемонии для важных событий. Система не "включается" резко.

use std::time::{Duration, Instant};
use tracing::info;

/// Фаза ритуала
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RitualPhase {
    /// Подготовка
    Preparation,
    /// Загрузка конфигурации
    LoadConfig,
    /// Прогрев соединений
    WarmConnections,
    /// Теневое тестирование
    ShadowTesting,
    /// Полная активация
    FullProduction,
    /// Завершение
    Complete,
}

impl RitualPhase {
    pub fn duration(&self) -> Duration {
        match self {
            RitualPhase::Preparation => Duration::from_secs(5),
            RitualPhase::LoadConfig => Duration::from_secs(10),
            RitualPhase::WarmConnections => Duration::from_secs(20),
            RitualPhase::ShadowTesting => Duration::from_secs(30),
            RitualPhase::FullProduction => Duration::from_secs(0),
            RitualPhase::Complete => Duration::from_secs(0),
        }
    }

    pub fn next(&self) -> Option<RitualPhase> {
        match self {
            RitualPhase::Preparation => Some(RitualPhase::LoadConfig),
            RitualPhase::LoadConfig => Some(RitualPhase::WarmConnections),
            RitualPhase::WarmConnections => Some(RitualPhase::ShadowTesting),
            RitualPhase::ShadowTesting => Some(RitualPhase::FullProduction),
            RitualPhase::FullProduction => Some(RitualPhase::Complete),
            RitualPhase::Complete => None,
        }
    }
}

/// Ritual Protocol — управляет церемониями переходов
pub struct RitualProtocol {
    current_phase: parking_lot::RwLock<RitualPhase>,
    phase_started_at: parking_lot::RwLock<Instant>,
}

impl RitualProtocol {
    pub fn new() -> Self {
        Self {
            current_phase: parking_lot::RwLock::new(RitualPhase::Preparation),
            phase_started_at: parking_lot::RwLock::new(Instant::now()),
        }
    }

    /// Текущая фаза
    pub fn current_phase(&self) -> RitualPhase {
        *self.current_phase.read()
    }

    /// Прогресс текущей фазы (0.0 - 1.0)
    pub fn phase_progress(&self) -> f64 {
        let phase = self.current_phase();
        let duration = phase.duration();

        if duration.is_zero() {
            return 1.0;
        }

        let elapsed = self.phase_started_at.read().elapsed();
        (elapsed.as_secs_f64() / duration.as_secs_f64()).min(1.0)
    }

    /// Обновление: переход к следующей фазе если готово
    pub fn update(&self) {
        let progress = self.phase_progress();
        if progress >= 1.0 {
            let current = self.current_phase();
            if let Some(next) = current.next() {
                info!("Ritual phase transition: {:?} → {:?}", current, next);
                *self.current_phase.write() = next;
                *self.phase_started_at.write() = Instant::now();
            }
        }
    }

    /// Готов ли к продакшену?
    pub fn is_production_ready(&self) -> bool {
        matches!(
            self.current_phase(),
            RitualPhase::FullProduction | RitualPhase::Complete
        )
    }
}

impl Default for RitualProtocol {
    fn default() -> Self {
        Self::new()
    }
}
