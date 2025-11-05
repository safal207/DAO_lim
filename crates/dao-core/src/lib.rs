//! DAO Core — Dynamic Awareness Orchestrator
//!
//! Ядро системы осознанной маршрутизации трафика.
//! Пять модулей лиминального шлюза:
//!
//! - **Gate**: Прием соединений (TCP, TLS, ALPN, SNI)
//! - **Sense**: Телеметрия и чувствование потока
//! - **Align**: Принятие решений и политики маршрутизации
//! - **Flow**: Конвейер фильтров и трансформаций
//! - **Memory**: Профили сервисов и горячая конфигурация

pub mod gate;
pub mod sense;
pub mod align;
pub mod flow;
pub mod memory;

pub mod config;
pub mod upstream;
pub mod error;

pub use error::{DaoError, Result};

/// Версия протокола DAO
pub const DAO_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Маркер Intent — тег намерения трафика
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Intent(pub String);

impl Intent {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn matches(&self, other: &Intent) -> bool {
        self.0 == other.0
    }
}

impl From<&str> for Intent {
    fn from(s: &str) -> Self {
        Intent(s.to_string())
    }
}

impl AsRef<str> for Intent {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
