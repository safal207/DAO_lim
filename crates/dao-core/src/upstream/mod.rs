//! Upstream management — работа с backend серверами

pub mod state;
pub mod client;
pub mod pool;

pub use state::{UpstreamState, UpstreamStats};
pub use client::UpstreamClient;
pub use pool::ConnectionPool;
