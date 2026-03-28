//! Upstream management — работа с backend серверами

pub mod state;
pub mod client;
pub mod pool;
pub mod circuit_breaker;

pub use state::{UpstreamState, UpstreamStats};
pub use client::UpstreamClient;
pub use pool::ConnectionPool;
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitStatus};
