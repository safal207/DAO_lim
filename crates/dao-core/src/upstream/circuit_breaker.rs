//! Circuit Breaker — защита от каскадных отказов
//!
//! Три состояния:
//!
//! ```text
//!                  failure_threshold
//!   Closed ──────────────────────────▶ Open
//!     ▲                                  │
//!     │ success_threshold                │ timeout
//!     │                                  ▼
//!     └──────────────────────────── HalfOpen
//!                  probe succeeds
//! ```
//!
//! - **Closed** — нормальная работа
//! - **Open** — запросы блокируются, upstream недоступен
//! - **HalfOpen** — один пробный запрос; если успешен → Closed, иначе → Open

use parking_lot::Mutex;
use std::time::{Duration, Instant};

/// Конфигурация circuit breaker для одного upstream
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Сколько ошибок подряд открывают цепь
    pub failure_threshold: u32,
    /// Сколько успешных запросов в HalfOpen закрывают цепь
    pub success_threshold: u32,
    /// Как долго цепь остаётся Open перед переходом в HalfOpen
    pub timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: Duration::from_secs(30),
        }
    }
}

/// Внутреннее состояние circuit breaker
#[derive(Debug)]
enum State {
    /// Нормальная работа — счётчик последовательных ошибок
    Closed { consecutive_failures: u32 },
    /// Цепь разомкнута — запросы блокируются
    Open { opened_at: Instant },
    /// Пробный режим — один запрос прошёл, ждём результата
    HalfOpen { consecutive_successes: u32 },
}

/// Circuit Breaker
#[derive(Debug)]
pub struct CircuitBreaker {
    state: Mutex<State>,
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Mutex::new(State::Closed { consecutive_failures: 0 }),
            config,
        }
    }

    /// Можно ли отправлять запрос к upstream?
    /// При переходе из Open→HalfOpen сразу разрешает один пробный запрос.
    pub fn is_available(&self) -> bool {
        let mut state = self.state.lock();
        match *state {
            State::Closed { .. } => true,
            State::Open { opened_at } => {
                if opened_at.elapsed() >= self.config.timeout {
                    // Переход в HalfOpen — разрешаем пробный запрос
                    *state = State::HalfOpen { consecutive_successes: 0 };
                    true
                } else {
                    false
                }
            }
            State::HalfOpen { .. } => true,
        }
    }

    /// Зафиксировать успешный запрос
    pub fn record_success(&self) {
        let mut state = self.state.lock();
        match *state {
            State::HalfOpen { consecutive_successes } => {
                let next = consecutive_successes + 1;
                if next >= self.config.success_threshold {
                    *state = State::Closed { consecutive_failures: 0 };
                } else {
                    *state = State::HalfOpen { consecutive_successes: next };
                }
            }
            State::Closed { .. } => {
                // Сбрасываем счётчик ошибок при успехе
                *state = State::Closed { consecutive_failures: 0 };
            }
            State::Open { .. } => {
                // Не должно происходить — запросы блокируются в Open
            }
        }
    }

    /// Зафиксировать ошибку запроса
    pub fn record_failure(&self) {
        let mut state = self.state.lock();
        match *state {
            State::Closed { consecutive_failures } => {
                let next = consecutive_failures + 1;
                if next >= self.config.failure_threshold {
                    *state = State::Open { opened_at: Instant::now() };
                } else {
                    *state = State::Closed { consecutive_failures: next };
                }
            }
            State::HalfOpen { .. } => {
                // Пробный запрос провалился — возвращаемся в Open
                *state = State::Open { opened_at: Instant::now() };
            }
            State::Open { .. } => {
                // Уже открыт — НЕ сбрасываем opened_at, иначе цепь
                // никогда не перейдёт в HalfOpen под продолжающейся нагрузкой
            }
        }
    }

    /// Текущее состояние для отображения
    pub fn status(&self) -> CircuitStatus {
        let state = self.state.lock();
        match *state {
            State::Closed { consecutive_failures } => CircuitStatus::Closed { consecutive_failures },
            State::Open { opened_at } => CircuitStatus::Open {
                opened_secs_ago: opened_at.elapsed().as_secs(),
                recovers_in_secs: self.config.timeout
                    .checked_sub(opened_at.elapsed())
                    .unwrap_or(Duration::ZERO)
                    .as_secs(),
            },
            State::HalfOpen { consecutive_successes } => CircuitStatus::HalfOpen { consecutive_successes },
        }
    }

    /// Является ли цепь открытой прямо сейчас (без перехода в HalfOpen)
    pub fn is_open(&self) -> bool {
        matches!(*self.state.lock(), State::Open { .. })
    }
}

/// Статус цепи для отображения в API/CLI
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum CircuitStatus {
    Closed { consecutive_failures: u32 },
    Open { opened_secs_ago: u64, recovers_in_secs: u64 },
    HalfOpen { consecutive_successes: u32 },
}

impl CircuitStatus {
    pub fn is_available(&self) -> bool {
        !matches!(self, CircuitStatus::Open { .. })
    }

    pub fn label(&self) -> &'static str {
        match self {
            CircuitStatus::Closed { .. }   => "closed",
            CircuitStatus::Open { .. }     => "open",
            CircuitStatus::HalfOpen { .. } => "half_open",
        }
    }
}
