//! DAO Server — обработка запросов

use dao_core::{
    align::Align,
    gate::{Connection, Gate, Protocol},
    memory::Memory,
    sense::Sense,
    upstream::{ConnectionPool, UpstreamState},
    Intent, Result,
};
use http_body_util::{combinators::BoxBody, BodyExt, Empty};
use hyper::body::Incoming;
use hyper::server::conn::{http1, http2};
use hyper::service::service_fn;
use hyper::{body::Bytes, Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info, warn};

/// DAO Server
pub struct DaoServer {
    gate: Arc<Gate>,
    sense: Arc<Sense>,
    align: Arc<Align>,
    memory: Arc<Memory>,
    upstreams: Arc<Vec<UpstreamState>>,
    pool: Arc<ConnectionPool>,
}

impl DaoServer {
    pub fn new(
        gate: Gate,
        sense: Sense,
        align: Align,
        memory: Arc<Memory>,
        upstreams: Arc<Vec<UpstreamState>>,
    ) -> Self {
        Self {
            gate: Arc::new(gate),
            sense: Arc::new(sense),
            align: Arc::new(align),
            memory,
            upstreams,
            pool: Arc::new(ConnectionPool::new()),
        }
    }

    /// Запуск сервера
    pub async fn run(self) -> anyhow::Result<()> {
        let self_arc = Arc::new(self);

        loop {
            match self_arc.gate.accept().await {
                Ok(conn) => {
                    let server = self_arc.clone();
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(conn).await {
                            error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                }
            }
        }
    }

    /// Обработка соединения
    async fn handle_connection(self: Arc<Self>, conn: Connection) -> Result<()> {
        let peer_addr = conn.peer_addr();
        let protocol = conn.protocol();

        debug!(
            "New connection from {} with protocol {:?}",
            peer_addr, protocol
        );

        match protocol {
            Protocol::Http1 | Protocol::Http2 => {
                self.handle_http_connection(conn).await?;
            }
            Protocol::WebSocket => {
                self.handle_websocket_connection(conn).await?;
            }
        }

        Ok(())
    }

    /// Обработка HTTP соединения
    async fn handle_http_connection(self: Arc<Self>, conn: Connection) -> Result<()> {
        match conn {
            Connection::Plain { stream, protocol, .. } => {
                let io = TokioIo::new(stream);
                let server = self.clone();

                let service = service_fn(move |req| {
                    let server = server.clone();
                    async move { server.handle_request(req).await }
                });

                match protocol {
                    Protocol::Http1 => {
                        if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                            error!("HTTP/1.1 connection error: {}", e);
                        }
                    }
                    Protocol::Http2 => {
                        if let Err(e) = http2::Builder::new(hyper_util::rt::TokioExecutor::new())
                            .serve_connection(io, service)
                            .await
                        {
                            error!("HTTP/2 connection error: {}", e);
                        }
                    }
                    _ => {}
                }
            }
            Connection::Tls { stream, protocol, .. } => {
                let io = TokioIo::new(stream);
                let server = self.clone();

                let service = service_fn(move |req| {
                    let server = server.clone();
                    async move { server.handle_request(req).await }
                });

                match protocol {
                    Protocol::Http1 => {
                        if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                            error!("HTTP/1.1 TLS connection error: {}", e);
                        }
                    }
                    Protocol::Http2 => {
                        if let Err(e) = http2::Builder::new(hyper_util::rt::TokioExecutor::new())
                            .serve_connection(io, service)
                            .await
                        {
                            error!("HTTP/2 TLS connection error: {}", e);
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// Обработка WebSocket соединения
    async fn handle_websocket_connection(&self, _conn: Connection) -> Result<()> {
        info!("WebSocket connection handled (placeholder)");
        // TODO: WebSocket proxying
        Ok(())
    }


    /// Обработка HTTP запроса
    async fn handle_request(
        self: Arc<Self>,
        req: Request<Incoming>,
    ) -> std::result::Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
        let start = Instant::now();
        let method = req.method().clone();
        let uri = req.uri().clone();

        debug!("Handling request: {} {}", method, uri);

        match self.process_request(req).await {
            Ok(response) => {
                let status = response.status();
                let latency = start.elapsed();
                debug!(
                    "Request completed: {} {} -> {} in {:?}",
                    method, uri, status, latency
                );
                Ok(response)
            }
            Err(e) => {
                error!("Request processing failed: {}", e);
                let response = Response::builder()
                    .status(502)
                    .body(
                        Empty::<Bytes>::new()
                            .map_err(|never: Infallible| match never {})
                            .boxed(),
                    )
                    .unwrap();
                Ok(response)
            }
        }
    }

    /// Обработка запроса с маршрутизацией
    async fn process_request(&self, req: Request<Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>> {
        let config = self.memory.get_config();

        // Поиск подходящего маршрута
        let route = config
            .routes
            .rule
            .iter()
            .find(|r| r.match_rule.matches(&req));

        if let Some(route) = route {
            debug!("Matched route: {}", route.name);

            // Получение upstream'ов для маршрута
            let route_upstreams: Vec<_> = route
                .upstreams
                .iter()
                .filter_map(|uc| {
                    self.upstreams
                        .iter()
                        .find(|u| u.name == uc.name)
                        .map(|u| Arc::new(u.clone()))
                })
                .collect();

            if route_upstreams.is_empty() {
                warn!("No upstreams available for route: {}", route.name);
                return self.error_response(503, "Service Unavailable");
            }

            // Выбор upstream через Align
            let request_intent = route.intent();
            let selected = self
                .align
                .select_upstream(&route.policy, &route_upstreams, request_intent.as_ref());

            if let Some(upstream) = selected {
                info!(
                    "Selected upstream: {} for route: {}",
                    upstream.name, route.name
                );

                // Проксирование к upstream
                match self.proxy_to_upstream(&upstream, req).await {
                    Ok((response, latency)) => {
                        let success = response.status().is_success();
                        upstream.record_request(latency, success);
                        self.sense
                            .record_upstream_request(&upstream.name, latency, success);

                        // Конвертация Response<Incoming> в Response<BoxBody>
                        let (parts, body) = response.into_parts();
                        let boxed_body = body.map_err(|e| hyper::Error::from(e)).boxed();
                        Ok(Response::from_parts(parts, boxed_body))
                    }
                    Err(e) => {
                        error!("Proxy to upstream {} failed: {}", upstream.name, e);
                        upstream.record_request(std::time::Duration::from_secs(0), false);
                        self.sense.record_upstream_request(
                            &upstream.name,
                            std::time::Duration::from_secs(0),
                            false,
                        );
                        self.error_response(502, "Bad Gateway")
                    }
                }
            } else {
                warn!("No suitable upstream selected for route: {}", route.name);
                self.error_response(503, "Service Unavailable")
            }
        } else {
            // Маршрут не найден
            debug!("No route matched for: {}", req.uri());
            self.error_response(404, "Not Found")
        }
    }

    /// Проксирование запроса к upstream
    async fn proxy_to_upstream(
        &self,
        upstream: &UpstreamState,
        req: Request<Incoming>,
    ) -> Result<(Response<Incoming>, std::time::Duration)> {
        let client = self.pool.get_client(&upstream.url);

        // Конвертация запроса для проксирования
        let (parts, body) = req.into_parts();
        let new_req = Request::from_parts(parts, body);

        client.proxy_request(&upstream.url, new_req).await
    }

    /// Создание error response
    fn error_response(&self, status: u16, _message: &str) -> Result<Response<BoxBody<Bytes, hyper::Error>>> {
        let response = Response::builder()
            .status(status)
            .body(
                Empty::<Bytes>::new()
                    .map_err(|never: Infallible| match never {})
                    .boxed(),
            )
            .map_err(|e| {
                dao_core::DaoError::Internal(format!("Failed to build response: {}", e))
            })?;
        Ok(response)
    }
}
