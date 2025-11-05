//! DAO Admin — hot-reload и управление
//!
//! Модуль для:
//! - Горячей перезагрузки конфигурации
//! - Мониторинга изменений файла конфигурации
//! - API управления (будущее)

use dao_core::config::DaoConfig;
use dao_core::memory::Memory;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

pub mod reload;

pub use reload::ConfigReloader;

/// Admin — система управления
pub struct Admin {
    config_path: PathBuf,
    memory: Arc<Memory>,
    reloader: ConfigReloader,
}

impl Admin {
    pub fn new(config_path: PathBuf, memory: Arc<Memory>) -> Self {
        let reloader = ConfigReloader::new(memory.clone());
        Self {
            config_path,
            memory,
            reloader,
        }
    }

    /// Запуск мониторинга конфигурации
    pub async fn start_config_watch(&self) -> anyhow::Result<()> {
        let (tx, mut rx) = mpsc::channel(100);
        let config_path = self.config_path.clone();

        // File watcher
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    if let Err(e) = tx.blocking_send(event) {
                        tracing::error!("Failed to send watch event: {}", e);
                    }
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )?;

        watcher.watch(&config_path, RecursiveMode::NonRecursive)?;

        tracing::info!("Started config watch for: {:?}", config_path);

        // Event loop
        let memory = self.memory.clone();
        let config_path_clone = config_path.clone();

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                match event.kind {
                    notify::EventKind::Modify(_) | notify::EventKind::Create(_) => {
                        tracing::info!("Config file changed, reloading...");

                        match DaoConfig::from_file(&config_path_clone) {
                            Ok(new_config) => {
                                if let Err(e) = memory.update_config(new_config) {
                                    tracing::error!("Failed to update config: {}", e);
                                } else {
                                    tracing::info!("Config reloaded successfully");
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to load config: {}", e);
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        // Держим watcher живым
        std::mem::forget(watcher);

        Ok(())
    }

    /// Ручная перезагрузка конфигурации
    pub async fn reload_config(&self) -> anyhow::Result<()> {
        self.reloader.reload_from_file(&self.config_path).await
    }

    /// Получение текущей конфигурации
    pub fn get_current_config(&self) -> DaoConfig {
        self.memory.get_config()
    }

    /// Откат к предыдущему snapshot
    pub fn rollback(&self, snapshot_index: usize) -> anyhow::Result<()> {
        self.memory
            .rollback_to_snapshot(snapshot_index)
            .map_err(|e| anyhow::anyhow!("Rollback failed: {}", e))
    }
}
