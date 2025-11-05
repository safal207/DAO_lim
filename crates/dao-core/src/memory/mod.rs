//! Memory — память системы
//!
//! Модуль хранения состояния и профилей:
//! - Горячая конфигурация
//! - Профили сервисов
//! - История состояний
//! - Snapshot'ы

use crate::config::DaoConfig;
use crate::Result;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::SystemTime;

pub mod profile;
pub mod snapshot;

pub use profile::ServiceProfile;
pub use snapshot::Snapshot;

/// Memory — хранилище состояния системы
#[derive(Clone)]
pub struct Memory {
    config: Arc<RwLock<DaoConfig>>,
    profiles: Arc<RwLock<std::collections::HashMap<String, ServiceProfile>>>,
    snapshots: Arc<RwLock<Vec<Snapshot>>>,
}

impl Memory {
    pub fn new(config: DaoConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            profiles: Arc::new(RwLock::new(std::collections::HashMap::new())),
            snapshots: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Получение текущей конфигурации
    pub fn get_config(&self) -> DaoConfig {
        self.config.read().clone()
    }

    /// Обновление конфигурации (hot-reload)
    pub fn update_config(&self, new_config: DaoConfig) -> Result<()> {
        new_config.validate()?;

        let mut config = self.config.write();
        *config = new_config;

        // Создание snapshot
        self.create_snapshot("config_update");

        Ok(())
    }

    /// Получение профиля сервиса
    pub fn get_profile(&self, service_name: &str) -> Option<ServiceProfile> {
        self.profiles.read().get(service_name).cloned()
    }

    /// Обновление профиля сервиса
    pub fn update_profile(&self, service_name: String, profile: ServiceProfile) {
        self.profiles.write().insert(service_name, profile);
    }

    /// Создание snapshot состояния
    pub fn create_snapshot(&self, reason: &str) {
        let snapshot = Snapshot {
            timestamp: SystemTime::now(),
            reason: reason.to_string(),
            config: self.config.read().clone(),
        };

        let mut snapshots = self.snapshots.write();
        snapshots.push(snapshot);

        // Ограничение количества snapshot'ов
        const MAX_SNAPSHOTS: usize = 100;
        if snapshots.len() > MAX_SNAPSHOTS {
            let excess = snapshots.len() - MAX_SNAPSHOTS;
            snapshots.drain(0..excess);
        }
    }

    /// Получение истории snapshot'ов
    pub fn get_snapshots(&self) -> Vec<Snapshot> {
        self.snapshots.read().clone()
    }

    /// Откат к предыдущему snapshot
    pub fn rollback_to_snapshot(&self, index: usize) -> Result<()> {
        let snapshots = self.snapshots.read();
        if let Some(snapshot) = snapshots.get(index) {
            let mut config = self.config.write();
            *config = snapshot.config.clone();
            Ok(())
        } else {
            Err(crate::DaoError::Internal("Snapshot not found".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;

    #[test]
    fn test_memory_snapshot() {
        let config = create_test_config();
        let memory = Memory::new(config);

        memory.create_snapshot("test");
        let snapshots = memory.get_snapshots();

        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].reason, "test");
    }

    fn create_test_config() -> DaoConfig {
        DaoConfig {
            server: ServerConfig {
                bind: "0.0.0.0:8443".to_string(),
                tls_cert: None,
                tls_key: None,
                workers: 1,
            },
            telemetry: None,
            routes: RoutesConfig {
                rule: vec![],
            },
            policies: None,
        }
    }
}
