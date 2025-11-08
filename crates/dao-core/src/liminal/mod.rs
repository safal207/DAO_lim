//! Liminal — пограничные состояния и осознанность
//!
//! Модуль лиминальных фич DAO:
//! - Shadow Traffic — теневое дублирование запросов
//! - Quantum Routing — hedged requests с суперпозицией
//! - Consciousness Levels — адаптивная осознанность
//! - Temporal Resonance — временные паттерны
//! - Liminal Zones — промежуточные ответы
//! - Echo Analysis — детекция аномалий через эхо
//! - Metamorphic Config — градуальные переходы
//! - Ritual Protocols — церемонии переходов
//! - Adaptive Thresholds — самообучающиеся пороги
//! - Presence Detection — состояния upstream
//! - Orchestrator — центральный координатор

pub mod shadow;
pub mod quantum;
pub mod consciousness;
pub mod temporal;
pub mod zones;
pub mod echo;
pub mod metamorphic;
pub mod ritual;
pub mod adaptive;
pub mod presence;
pub mod orchestrator;

pub use shadow::ShadowTraffic;
pub use quantum::QuantumRouter;
pub use consciousness::{ConsciousnessLevel, AwarenessOrchestrator, AwarenessFactors};
pub use temporal::TemporalResonance;
pub use zones::LiminalZones;
pub use echo::EchoAnalyzer;
pub use metamorphic::MetamorphicConfig;
pub use ritual::RitualProtocol;
pub use adaptive::AdaptiveThresholds;
pub use presence::{PresenceState, PresenceDetector};
pub use orchestrator::{LiminalOrchestrator, LiminalConfig};
