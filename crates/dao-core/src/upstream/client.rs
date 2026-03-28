//! HTTP client для upstream соединений

use crate::Result;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response, Uri};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use std::time::Instant;
use tracing::{debug, error};

/// HTTP client для проксирования запросов к upstreams.
///
/// Хранит два клиента:
/// - `client_bytes` — для проксирования с буферизованным телом (`Full<Bytes>`);
///   используется основным путём, поддерживает retry и Request-ID инжекцию.
#[derive(Clone)]
pub struct UpstreamClient {
    client_bytes: Client<HttpConnector, Full<Bytes>>,
}

impl UpstreamClient {
    pub fn new() -> Self {
        let client_bytes = Client::builder(TokioExecutor::new()).build_http::<Full<Bytes>>();
        Self { client_bytes }
    }

    /// Проксирование запроса с буферизованным телом.
    ///
    /// Тело (`Full<Bytes>`) дёшево клонируется — позволяет retry без
    /// повторного чтения входящего потока.
    pub async fn proxy_request(
        &self,
        upstream_url: &str,
        mut req: Request<Full<Bytes>>,
    ) -> Result<(Response<Incoming>, std::time::Duration)> {
        let start = Instant::now();

        let upstream_uri: Uri = upstream_url
            .parse()
            .map_err(|e| crate::DaoError::Upstream(format!("Invalid upstream URL: {}", e)))?;

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

        *req.uri_mut() = new_uri;
        remove_hop_by_hop_headers(req.headers_mut());

        let response = self
            .client_bytes
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

/// Удаление hop-by-hop headers (RFC 2616)
fn remove_hop_by_hop_headers(headers: &mut http::HeaderMap) {
    let hop_by_hop = [
        http::header::CONNECTION,
        http::header::TRANSFER_ENCODING,
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
        let _client = UpstreamClient::new();
    }
}
