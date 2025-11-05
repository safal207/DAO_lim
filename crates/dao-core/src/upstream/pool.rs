//! Connection pooling для upstreams

use super::client::UpstreamClient;
use dashmap::DashMap;
use std::sync::Arc;

/// Connection pool для upstreams
#[derive(Clone)]
pub struct ConnectionPool {
    // URL -> Client
    clients: Arc<DashMap<String, UpstreamClient>>,
}

impl ConnectionPool {
    /// Создание нового пула
    pub fn new() -> Self {
        Self {
            clients: Arc::new(DashMap::new()),
        }
    }

    /// Получение клиента для upstream (или создание нового)
    pub fn get_client(&self, upstream_url: &str) -> UpstreamClient {
        self.clients
            .entry(upstream_url.to_string())
            .or_insert_with(UpstreamClient::new)
            .clone()
    }

    /// Очистка пула
    pub fn clear(&self) {
        self.clients.clear();
    }

    /// Размер пула
    pub fn size(&self) -> usize {
        self.clients.len()
    }
}

impl Default for ConnectionPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_creation() {
        let pool = ConnectionPool::new();
        assert_eq!(pool.size(), 0);
    }

    #[test]
    fn test_pool_get_client() {
        let pool = ConnectionPool::new();
        let _client = pool.get_client("http://localhost:8080");
        assert_eq!(pool.size(), 1);

        // Повторный get должен вернуть того же клиента
        let _client2 = pool.get_client("http://localhost:8080");
        assert_eq!(pool.size(), 1);
    }
}
