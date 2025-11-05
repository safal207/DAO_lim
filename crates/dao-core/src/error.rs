//! Типы ошибок DAO

use std::fmt;

pub type Result<T> = std::result::Result<T, DaoError>;

#[derive(Debug, thiserror::Error)]
pub enum DaoError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] hyper::Error),

    #[error("TLS error: {0}")]
    Tls(String),

    #[error("Upstream error: {0}")]
    Upstream(String),

    #[error("Policy error: {0}")]
    Policy(String),

    #[error("Filter error: {0}")]
    Filter(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl DaoError {
    pub fn config(msg: impl fmt::Display) -> Self {
        Self::Config(msg.to_string())
    }

    pub fn upstream(msg: impl fmt::Display) -> Self {
        Self::Upstream(msg.to_string())
    }

    pub fn internal(msg: impl fmt::Display) -> Self {
        Self::Internal(msg.to_string())
    }
}
