//! Upstream management — работа с backend серверами

pub mod state;
pub mod client;
pub mod pool;
pub mod circuit_breaker;
pub mod health;

pub use state::{UpstreamState, UpstreamStats, HealthProbeResult};
pub use client::UpstreamClient;
pub use pool::ConnectionPool;
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitStatus};
pub use health::{HealthCheckConfig, spawn_health_checker};
