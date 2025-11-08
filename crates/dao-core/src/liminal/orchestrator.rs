//! Liminal Orchestrator — центральный координатор лиминальных фич
//!
//! Управляет всеми 10 лиминальными возможностями DAO.

use super::*;
use crate::upstream::UpstreamState;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::info;

/// Конфигурация лиминального оркестратора
#[derive(Debug, Clone)]
pub struct LiminalConfig {
    /// Shadow Traffic
    pub shadow: Option<shadow::ShadowConfig>,
    /// Quantum Routing
    pub quantum: Option<quantum::QuantumConfig>,
    /// Consciousness (всегда включен)
    pub consciousness: consciousness::AwarenessConfig,
    /// Temporal Resonance (всегда включен)
    pub temporal_enabled: bool,
    /// Liminal Zones
    pub zones: Option<Vec<zones::LiminalZoneConfig>>,
    /// Echo Analysis
    pub echo: Option<EchoConfig>,
    /// Adaptive Thresholds (всегда включен)
    pub adaptive_enabled: bool,
    /// Presence Detection (всегда включен)
    pub presence_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct EchoConfig {
    pub buffer_size: usize,
    pub anomaly_threshold: f64,
}

impl Default for LiminalConfig {
    fn default() -> Self {
        Self {
            shadow: None,
            quantum: None,
            consciousness: consciousness::AwarenessConfig::default(),
            temporal_enabled: true,
            zones: Some(vec![]),
            echo: Some(EchoConfig {
                buffer_size: 1000,
                anomaly_threshold: 3.0,
            }),
            adaptive_enabled: true,
            presence_enabled: true,
        }
    }
}

/// Liminal Orchestrator — управляет всеми лиминальными фичами
pub struct LiminalOrchestrator {
    /// Shadow Traffic
    shadow: Option<Arc<ShadowTraffic>>,
    /// Quantum Router
    quantum: Option<Arc<QuantumRouter>>,
    /// Consciousness
    consciousness: Arc<AwarenessOrchestrator>,
    /// Temporal Resonance
    temporal: Option<Arc<TemporalResonance>>,
    /// Liminal Zones
    zones: Option<Arc<LiminalZones>>,
    /// Echo Analyzer
    echo: Option<Arc<EchoAnalyzer>>,
    /// Adaptive Thresholds
    adaptive: Option<Arc<AdaptiveThresholds>>,
    /// Ritual Protocol (для startup)
    ritual: Arc<RitualProtocol>,
    /// Metamorphic Config (для плавных переходов)
    metamorphic: Arc<MetamorphicConfig>,
}

impl LiminalOrchestrator {
    pub fn new(config: LiminalConfig) -> Self {
        info!("Initializing Liminal Orchestrator");

        Self {
            shadow: config.shadow.map(|cfg| Arc::new(ShadowTraffic::new(cfg))),
            quantum: config.quantum.map(|cfg| Arc::new(QuantumRouter::new(cfg))),
            consciousness: Arc::new(AwarenessOrchestrator::new(config.consciousness)),
            temporal: if config.temporal_enabled {
                Some(Arc::new(TemporalResonance::new()))
            } else {
                None
            },
            zones: config.zones.map(|z| Arc::new(LiminalZones::new(z))),
            echo: config.echo.map(|cfg| {
                Arc::new(EchoAnalyzer::new(cfg.buffer_size, cfg.anomaly_threshold))
            }),
            adaptive: if config.adaptive_enabled {
                Some(Arc::new(AdaptiveThresholds::new()))
            } else {
                None
            },
            ritual: Arc::new(RitualProtocol::new()),
            metamorphic: Arc::new(MetamorphicConfig::new()),
        }
    }

    // ===== Getters для компонентов =====

    pub fn shadow(&self) -> Option<&Arc<ShadowTraffic>> {
        self.shadow.as_ref()
    }

    pub fn quantum(&self) -> Option<&Arc<QuantumRouter>> {
        self.quantum.as_ref()
    }

    pub fn consciousness(&self) -> &Arc<AwarenessOrchestrator> {
        &self.consciousness
    }

    pub fn temporal(&self) -> Option<&Arc<TemporalResonance>> {
        self.temporal.as_ref()
    }

    pub fn zones(&self) -> Option<&Arc<LiminalZones>> {
        self.zones.as_ref()
    }

    pub fn echo(&self) -> Option<&Arc<EchoAnalyzer>> {
        self.echo.as_ref()
    }

    pub fn adaptive(&self) -> Option<&Arc<AdaptiveThresholds>> {
        self.adaptive.as_ref()
    }

    pub fn ritual(&self) -> &Arc<RitualProtocol> {
        &self.ritual
    }

    pub fn metamorphic(&self) -> &Arc<MetamorphicConfig> {
        &self.metamorphic
    }

    // ===== Update loop =====

    /// Обновление всех лиминальных систем (вызывать периодически)
    pub fn update(&self, factors: &consciousness::AwarenessFactors) {
        // Обновляем уровень осознанности
        self.consciousness.update_level(factors);

        // Обновляем адаптивные пороги
        if let Some(adaptive) = &self.adaptive {
            adaptive.update(
                factors.current_rps,
                factors.error_rate,
                factors.p95_latency_ms,
            );
        }

        // Обновляем ritual protocol
        self.ritual.update();

        // Обновляем metamorphic config
        self.metamorphic.update_progress();
    }

    /// Записать временное наблюдение
    pub fn record_temporal_observation(&self, obs: temporal::TemporalObservation) {
        if let Some(temporal) = &self.temporal {
            temporal.record_observation(obs);
        }
    }

    /// Записать эхо запроса
    pub fn record_echo(&self, echo: echo::RequestEcho) {
        if let Some(analyzer) = &self.echo {
            analyzer.record_echo(echo);
        }
    }

    /// Проверить, является ли запрос аномальным
    pub fn is_anomaly(&self, echo: &echo::RequestEcho) -> bool {
        if let Some(analyzer) = &self.echo {
            analyzer.is_anomaly(echo)
        } else {
            false
        }
    }

    /// Готов ли к продакшену?
    pub fn is_production_ready(&self) -> bool {
        self.ritual.is_production_ready()
    }
}

impl Default for LiminalOrchestrator {
    fn default() -> Self {
        Self::new(LiminalConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_creation() {
        let orchestrator = LiminalOrchestrator::default();
        assert!(orchestrator.consciousness().current_level() >= consciousness::ConsciousnessLevel::Dormant);
    }
}
