//! HTTP client для upstream соединений

use crate::Result;
use hyper::body::Incoming;
use hyper::{Request, Response, Uri};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use std::time::Instant;
use tracing::{debug, error};

/// HTTP client для проксирования запросов к upstreams
#[derive(Clone)]
pub struct UpstreamClient {
    client: Client<hyper_util::client::legacy::connect::HttpConnector, Incoming>,
}

impl UpstreamClient {
    /// Создание нового клиента
    pub fn new() -> Self {
        let client = Client::builder(TokioExecutor::new()).build_http();
        Self { client }
    }

    /// Проксирование запроса к upstream
    pub async fn proxy_request(
        &self,
        upstream_url: &str,
        mut req: Request<Incoming>,
    ) -> Result<(Response<Incoming>, std::time::Duration)> {
        let start = Instant::now();

        // Парсинг upstream URL
        let upstream_uri: Uri = upstream_url
            .parse()
            .map_err(|e| crate::DaoError::Upstream(format!("Invalid upstream URL: {}", e)))?;

        // Построение нового URI с upstream хостом
        let path_and_query = req
            .uri()
            .path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or("/");

        let new_uri = Uri::builder()
            .scheme(upstream_uri.scheme().cloned().unwrap_or("http".parse().unwrap()))
            .authority(
                upstream_uri
                    .authority()
                    .cloned()
                    .ok_or_else(|| crate::DaoError::Upstream("No authority in upstream URL".to_string()))?,
            )
            .path_and_query(path_and_query)
            .build()
            .map_err(|e| crate::DaoError::Upstream(format!("Failed to build URI: {}", e)))?;

        debug!("Proxying request to: {}", new_uri);

        // Обновление URI в запросе
        *req.uri_mut() = new_uri;

        // Удаление hop-by-hop headers
        remove_hop_by_hop_headers(req.headers_mut());

        // Отправка запроса
        let response = self
            .client
            .request(req)
            .await
            .map_err(|e| {
                error!("Upstream request failed: {}", e);
                crate::DaoError::Upstream(format!("Request failed: {}", e))
            })?;

        let latency = start.elapsed();
        debug!("Upstream responded in {:?}", latency);

        Ok((response, latency))
    }
}

impl Default for UpstreamClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Удаление hop-by-hop headers
fn remove_hop_by_hop_headers(headers: &mut http::HeaderMap) {
    // Список hop-by-hop headers согласно RFC 2616
    let hop_by_hop = [
        http::header::CONNECTION,
        http::header::TRANSFER_ENCODING,
        http::header::UPGRADE,
        http::HeaderName::from_static("keep-alive"),
        http::HeaderName::from_static("proxy-authenticate"),
        http::HeaderName::from_static("proxy-authorization"),
        http::HeaderName::from_static("te"),
        http::HeaderName::from_static("trailer"),
    ];

    for header in &hop_by_hop {
        headers.remove(header);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = UpstreamClient::new();
        assert!(true); // Базовая проверка создания
    }
}
