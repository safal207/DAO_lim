//! DAO Server — обработка запросов

use dao_core::{
    align::Align,
    flow::RateLimiterMap,
    gate::{Connection, Gate, Protocol},
    memory::Memory,
    sense::Sense,
    upstream::{ConnectionPool, UpstreamState},
    Result,
};
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::body::Incoming;
use hyper::server::conn::{http1, http2};
use hyper::service::service_fn;
use hyper::{body::Bytes, Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Instant;
use opentelemetry::global;
use opentelemetry::propagation::TextMapPropagator;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures::StreamExt;
use tracing::{debug, error, info, info_span, warn, Instrument};

/// DAO Server
pub struct DaoServer {
    gate: Arc<Gate>,
    sense: Arc<Sense>,
    align: Arc<Align>,
    memory: Arc<Memory>,
    upstreams: Arc<Vec<UpstreamState>>,
    pool: Arc<ConnectionPool>,
    rate_limiter: RateLimiterMap,
}

/// Состояние для WebSocket handler (клонируется в spawn)
#[derive(Clone)]
struct WsHandlerState {
    upstreams: Arc<Vec<UpstreamState>>,
    memory: Arc<Memory>,
    sense: Arc<Sense>,
    align: Arc<Align>,
}

impl WsHandlerState {
    async fn handle_ws_request(
        self: Arc<Self>,
        req: Request<Incoming>,
    ) -> std::result::Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
        // Проверяем WebSocket upgrade
        let is_ws = req.headers()
            .get(hyper::header::UPGRADE)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_lowercase().contains("websocket"))
            .unwrap_or(false);

        if !is_ws {
            return Ok(Response::builder()
                .status(400)
                .body(Empty::<Bytes>::new()
                    .map_err(|never: Infallible| match never {})
                    .boxed())
                .unwrap());
        }

        let config = self.memory.get_config();
        let route = config.routes.rule.iter().find(|r| r.match_rule.matches(&req));

        let upstream_url = if let Some(route) = route {
            let route_upstreams: Vec<_> = route.upstreams.iter()
                .filter_map(|uc| {
                    self.upstreams.iter().find(|u| u.name == uc.name).map(|u| Arc::new(u.clone()))
                })
                .collect();

            let intent = route.intent();
            let selected = self.align.select_upstream(&route.policy, &route_upstreams, intent.as_ref());
            selected.map(|u| u.url.clone())
        } else {
            None
        };

        match upstream_url {
            Some(url) => {
                match DaoServer::proxy_websocket_upgrade(&url, req).await {
                    Ok(resp) => Ok(resp),
                    Err(e) => {
                        error!("WebSocket proxy failed: {}", e);
                        Ok(Response::builder()
                            .status(502)
                            .body(Empty::<Bytes>::new()
                                .map_err(|never: Infallible| match never {})
                                .boxed())
                            .unwrap())
                    }
                }
            }
            None => Ok(Response::builder()
                .status(404)
                .body(Empty::<Bytes>::new()
                    .map_err(|never: Infallible| match never {})
                    .boxed())
                .unwrap()),
        }
    }
}

impl DaoServer {
    pub fn new(
        gate: Gate,
        sense: Sense,
        align: Align,
        memory: Arc<Memory>,
        upstreams: Arc<Vec<UpstreamState>>,
        rate_limiter: RateLimiterMap,
    ) -> Self {
        Self {
            gate: Arc::new(gate),
            sense: Arc::new(sense),
            align: Arc::new(align),
            memory,
            upstreams,
            pool: Arc::new(ConnectionPool::new()),
            rate_limiter,
        }
    }

    /// Запуск сервера с поддержкой graceful shutdown.
    ///
    /// Принимает соединения до получения сигнала завершения.
    /// После сигнала перестаёт принимать новые соединения и ждёт
    /// завершения активных (не более `drain_timeout_secs`).
    pub async fn run(self, shutdown: tokio::sync::broadcast::Receiver<()>) -> anyhow::Result<()> {
        let self_arc = Arc::new(self);
        let active = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut shutdown = shutdown;

        loop {
            tokio::select! {
                result = self_arc.gate.accept() => {
                    match result {
                        Ok(conn) => {
                            let server = self_arc.clone();
                            let active = active.clone();
                            active.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            tokio::spawn(async move {
                                if let Err(e) = server.handle_connection(conn).await {
                                    error!("Connection error: {}", e);
                                }
                                active.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                            });
                        }
                        Err(e) => {
                            error!("Accept error: {}", e);
                        }
                    }
                }
                _ = shutdown.recv() => {
                    break;
                }
            }
        }

        // Drain: ждём завершения активных соединений (max 30 секунд)
        const DRAIN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
        let drain_start = std::time::Instant::now();

        loop {
            let n = active.load(std::sync::atomic::Ordering::SeqCst);
            if n == 0 {
                break;
            }
            if drain_start.elapsed() >= DRAIN_TIMEOUT {
                warn!("Drain timeout: {} connections still active, forcing exit", n);
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        info!("Graceful shutdown complete");
        Ok(())
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

    /// Обработка WebSocket соединения — bidirectional proxy
    async fn handle_websocket_connection(self: Arc<Self>, conn: Connection) -> Result<()> {
        let state = Arc::new(WsHandlerState {
            upstreams: self.upstreams.clone(),
            memory: self.memory.clone(),
            sense: self.sense.clone(),
            align: self.align.clone(),
        });

        let service = service_fn(move |req: Request<Incoming>| {
            let state = state.clone();
            async move { state.handle_ws_request(req).await }
        });

        match conn {
            Connection::Plain { stream, .. } => {
                let io = TokioIo::new(stream);
                if let Err(e) = http1::Builder::new()
                    .serve_connection(io, service)
                    .with_upgrades()
                    .await
                {
                    debug!("WebSocket HTTP/1.1 error: {}", e);
                }
            }
            Connection::Tls { stream, .. } => {
                let io = TokioIo::new(stream);
                if let Err(e) = http1::Builder::new()
                    .serve_connection(io, service)
                    .with_upgrades()
                    .await
                {
                    debug!("WebSocket TLS HTTP/1.1 error: {}", e);
                }
            }
        }
        Ok(())
    }

    /// Proxy WebSocket upgrade request
    async fn proxy_websocket_upgrade(
        upstream_url: &str,
        req: Request<Incoming>,
    ) -> Result<Response<BoxBody<Bytes, hyper::Error>>> {
        use hyper::header::{
            CONNECTION, UPGRADE, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_ACCEPT,
            SEC_WEBSOCKET_VERSION,
        };
        use base64::{Engine, engine::general_purpose::STANDARD};
        use sha1::{Sha1, Digest};

        // Валидация WebSocket handshake
        let ws_key = req
            .headers()
            .get(SEC_WEBSOCKET_KEY)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        // Вычисляем Sec-WebSocket-Accept
        let mut hasher = Sha1::new();
        hasher.update(ws_key.as_bytes());
        hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
        let accept = STANDARD.encode(hasher.finalize());

        // Конвертация upstream URL (http → ws, https → wss)
        let ws_upstream_url = upstream_url
            .replacen("http://", "ws://", 1)
            .replacen("https://", "wss://", 1);

        // Подключение к upstream WebSocket
        let path = req.uri().path_and_query().map(|p| p.as_str()).unwrap_or("/");
        let full_url = format!("{}{}", ws_upstream_url.trim_end_matches('/'), path);

        info!("WebSocket proxying to: {}", full_url);

        // Upgrade запрос к upstream
        let (upstream_ws, _) = match connect_async(&full_url).await {
            Ok(result) => result,
            Err(e) => {
                error!("WebSocket upstream connect failed: {}", e);
                return Err(dao_core::DaoError::Upstream(format!(
                    "WebSocket connect failed: {}", e
                )));
            }
        };

        // Ответ клиенту: 101 Switching Protocols
        let response = Response::builder()
            .status(101)
            .header(UPGRADE, "websocket")
            .header(CONNECTION, "Upgrade")
            .header(SEC_WEBSOCKET_ACCEPT, accept)
            .body(
                Empty::<Bytes>::new()
                    .map_err(|never: Infallible| match never {})
                    .boxed(),
            )
            .map_err(|e| dao_core::DaoError::Internal(format!("Response build error: {}", e)))?;

        // Spawn bidirectional proxy
        tokio::spawn(async move {
            let (upstream_sink, upstream_stream) = upstream_ws.split();

            // Получаем upgrade stream от клиента через hyper
            let on_upgrade = hyper::upgrade::on(req);
            match on_upgrade.await {
                Ok(upgraded) => {
                    let client_ws = tokio_tungstenite::WebSocketStream::from_raw_socket(
                        TokioIo::new(upgraded),
                        tokio_tungstenite::tungstenite::protocol::Role::Server,
                        None,
                    ).await;
                    let (client_sink, client_stream) = client_ws.split();

                    // Bidirectional copy
                    let c2u = client_stream.forward(upstream_sink);
                    let u2c = upstream_stream.forward(client_sink);
                    tokio::select! {
                        _ = c2u => debug!("Client WebSocket stream closed"),
                        _ = u2c => debug!("Upstream WebSocket stream closed"),
                    }
                }
                Err(e) => error!("WebSocket upgrade failed: {}", e),
            }
        });

        Ok(response)
    }


    /// Обработка HTTP запроса
    async fn handle_request(
        self: Arc<Self>,
        req: Request<Incoming>,
    ) -> std::result::Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
        let start = Instant::now();
        let method = req.method().to_string();
        let path = req.uri().path().to_string();

        // OTel span для полного жизненного цикла запроса
        let span = info_span!(
            "dao.request",
            http.method = %method,
            http.target = %path,
            http.status_code = tracing::field::Empty,
            otel.kind = "SERVER",
        );

        let result = self.process_request(req).instrument(span.clone()).await;

        match result {
            Ok(response) => {
                let status = response.status().as_u16();
                let latency = start.elapsed();
                span.record("http.status_code", status);
                debug!(
                    method = %method,
                    path = %path,
                    status = status,
                    latency_ms = %format!("{:.1}", latency.as_secs_f64() * 1000.0),
                    "request completed",
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

    /// Обработка запроса с маршрутизацией, retry и Request ID
    async fn process_request(&self, req: Request<Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>> {
        let config = self.memory.get_config();

        // Поиск маршрута ДО разбора тела (matches требует &Request<Incoming>)
        let route_idx = {
            let _span = info_span!("dao.route_match").entered();
            config.routes.rule.iter().position(|r| r.match_rule.matches(&req))
        };

        // Разбираем запрос: headers + буферизуем тело
        let (req_parts, req_body) = req.into_parts();
        let body_bytes = req_body.collect().await
            .map(|c| c.to_bytes())
            .unwrap_or_default();

        // Извлекаем или генерируем Request ID
        let request_id = req_parts.headers
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(generate_request_id);

        let Some(route) = route_idx.map(|i| &config.routes.rule[i]) else {
            debug!(path = %req_parts.uri, "no route matched");
            return self.error_response(404, "Not Found");
        };

        debug!(route = %route.name, "matched route");
        metrics::counter!("dao_requests_total", "route" => route.name.clone()).increment(1);

        // Rate limiting
        if !self.rate_limiter.check(&route.name) {
            metrics::counter!("dao_rate_limited_total", "route" => route.name.clone()).increment(1);
            warn!(route = %route.name, "rate limit exceeded → 429");
            return self.error_response(429, "Too Many Requests");
        }

        // Upstream кандидаты для маршрута
        let route_upstreams: Vec<_> = route.upstreams.iter()
            .filter_map(|uc| {
                self.upstreams.iter().find(|u| u.name == uc.name).map(|u| Arc::new(u.clone()))
            })
            .collect();

        if route_upstreams.is_empty() {
            warn!(route = %route.name, "no upstreams configured");
            return self.error_response(503, "Service Unavailable");
        }

        let request_intent = route.intent();
        let max_retries = route.retry_attempts as usize;
        let mut tried: Vec<String> = Vec::new();

        // Retry loop: пробуем до max_retries+1 раз с разными upstream'ами
        for attempt in 0..=max_retries {
            // Кандидаты = ещё не опробованные
            let candidates: Vec<_> = route_upstreams.iter()
                .filter(|u| !tried.contains(&u.name))
                .cloned()
                .collect();

            if candidates.is_empty() {
                break;
            }

            let upstream = {
                let _span = info_span!(
                    "dao.upstream_select",
                    route = %route.name,
                    policy = %route.policy,
                    attempt = attempt,
                ).entered();
                self.align.select_upstream(&route.policy, &candidates, request_intent.as_ref())
            };

            let Some(upstream) = upstream else {
                break;
            };

            tried.push(upstream.name.clone());

            // Строим запрос для этой попытки (тело клонируется дёшево — Arc)
            let attempt_req = build_attempt_request(
                &req_parts,
                body_bytes.clone(),
                &request_id,
            );

            info!(
                route = %route.name,
                upstream = %upstream.name,
                attempt = attempt,
                "proxying request",
            );

            let proxy_result = self
                .proxy_to_upstream(&upstream, attempt_req)
                .instrument(info_span!(
                    "dao.upstream_proxy",
                    upstream = %upstream.name,
                    upstream.url = %upstream.url,
                    attempt = attempt,
                ))
                .await;

            match proxy_result {
                Ok((response, latency)) => {
                    let success = response.status().is_success()
                        || response.status().as_u16() < 500;

                    upstream.record_request(latency, success);
                    self.sense.record_upstream_request(&upstream.name, latency, success);

                    let status_str = response.status().as_u16().to_string();
                    metrics::counter!(
                        "dao_upstream_requests_total",
                        "upstream" => upstream.name.clone(),
                        "status" => status_str,
                    ).increment(1);
                    metrics::histogram!(
                        "dao_upstream_latency_seconds",
                        "upstream" => upstream.name.clone(),
                    ).record(latency.as_secs_f64());

                    if !success && attempt < max_retries {
                        warn!(
                            upstream = %upstream.name,
                            status = %response.status(),
                            attempt = attempt,
                            "upstream returned 5xx, retrying",
                        );
                        // Дренируем тело и пробуем следующий
                        let _ = response.into_body().collect().await;
                        continue;
                    }

                    // Финальный ответ — добавляем X-Request-ID
                    let (mut parts, body) = response.into_parts();
                    if let Ok(val) = hyper::header::HeaderValue::from_str(&request_id) {
                        parts.headers.insert("x-request-id", val);
                    }
                    let boxed_body = body.map_err(|e| hyper::Error::from(e)).boxed();
                    return Ok(Response::from_parts(parts, boxed_body));
                }
                Err(e) => {
                    upstream.record_request(std::time::Duration::ZERO, false);
                    self.sense.record_upstream_request(&upstream.name, std::time::Duration::ZERO, false);
                    if attempt < max_retries {
                        warn!(upstream = %upstream.name, error = %e, attempt = attempt, "upstream error, retrying");
                        continue;
                    }
                    error!(upstream = %upstream.name, error = %e, "all retries exhausted");
                }
            }
        }

        self.error_response(502, "Bad Gateway")
    }

    /// Проксирование запроса к upstream
    async fn proxy_to_upstream(
        &self,
        upstream: &UpstreamState,
        req: Request<Full<Bytes>>,
    ) -> Result<(Response<Incoming>, std::time::Duration)> {
        let client = self.pool.get_client(&upstream.url);
        client.proxy_request(&upstream.url, req).await
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

/// Строит Request<Full<Bytes>> для одной попытки проксирования.
/// Клонирует parts (метод, URI, заголовки) и тело (Bytes — дёшево, Arc внутри).
fn build_attempt_request(
    orig_parts: &http::request::Parts,
    body: Bytes,
    request_id: &str,
) -> Request<Full<Bytes>> {
    let mut builder = Request::builder()
        .method(orig_parts.method.clone())
        .uri(orig_parts.uri.clone())
        .version(orig_parts.version);

    // Копируем все заголовки из оригинала
    if let Some(headers) = builder.headers_mut() {
        headers.extend(orig_parts.headers.clone());
        // Инжектируем / перезаписываем X-Request-ID
        if let Ok(val) = hyper::header::HeaderValue::from_str(request_id) {
            headers.insert("x-request-id", val);
        }
        // W3C TraceContext
        inject_trace_context(headers);
    }

    builder.body(Full::new(body)).expect("valid request")
}

/// Генерация уникального Request ID (32 hex символа)
fn generate_request_id() -> String {
    use rand::Rng;
    let bytes: [u8; 16] = rand::thread_rng().gen();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Инжектирует W3C TraceContext заголовки (`traceparent`, `tracestate`)
/// в заголовки HTTP запроса из текущего tracing span.
///
/// Если OTLP не настроен — текущий span будет no-op и заголовки не добавятся.
fn inject_trace_context(headers: &mut hyper::HeaderMap) {
    // Получаем OTel контекст из текущего tracing span
    let cx = tracing::Span::current().context();

    // Injector для hyper HeaderMap
    struct HeaderInjector<'a>(&'a mut hyper::HeaderMap);

    impl<'a> opentelemetry::propagation::Injector for HeaderInjector<'a> {
        fn set(&mut self, key: &str, value: String) {
            if let (Ok(name), Ok(val)) = (
                hyper::header::HeaderName::from_bytes(key.as_bytes()),
                hyper::header::HeaderValue::from_str(&value),
            ) {
                self.0.insert(name, val);
            }
        }
    }

    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, &mut HeaderInjector(headers));
    });
}
