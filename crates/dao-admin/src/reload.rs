//! Config reloader

use dao_core::config::DaoConfig;
use dao_core::memory::Memory;
use std::path::Path;
use std::sync::Arc;

/// Перезагрузчик конфигурации
pub struct ConfigReloader {
    memory: Arc<Memory>,
}

impl ConfigReloader {
    pub fn new(memory: Arc<Memory>) -> Self {
        Self { memory }
    }

    /// Перезагрузка конфигурации из файла
    pub async fn reload_from_file(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        tracing::info!("Reloading config from: {:?}", path.as_ref());

        let new_config = DaoConfig::from_file(path)?;
        new_config.validate()?;

        self.memory.update_config(new_config)?;

        tracing::info!("Config reloaded successfully");
        Ok(())
    }

    /// Валидация конфигурации без применения
    pub fn validate_config(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let config = DaoConfig::from_file(path)?;
        config.validate()?;
        Ok(())
    }
}
