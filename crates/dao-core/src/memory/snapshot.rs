//! Configuration snapshots

use crate::config::DaoConfig;
use std::time::SystemTime;

/// Snapshot конфигурации системы
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub timestamp: SystemTime,
    pub reason: String,
    pub config: DaoConfig,
}

impl Snapshot {
    /// Создание нового snapshot
    pub fn new(reason: String, config: DaoConfig) -> Self {
        Self {
            timestamp: SystemTime::now(),
            reason,
            config,
        }
    }

    /// Возраст snapshot в секундах
    pub fn age_seconds(&self) -> u64 {
        self.timestamp
            .elapsed()
            .unwrap_or_default()
            .as_secs()
    }
}
