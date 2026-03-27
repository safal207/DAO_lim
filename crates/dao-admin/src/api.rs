//! Admin HTTP API
//!
//! Эндпоинты для мониторинга и отладки:
//! - GET  /admin/upstreams        — метрики всех upstream
//! - GET  /admin/explain?...      — объяснение решения маршрутизации
//! - GET  /admin/health           — healthcheck

use dao_core::{
    align::Align,
    explain::{ExplainRequest, ExplainResult, PolicyWeightsInfo, UpstreamInfo, UpstreamsResponse},
    memory::Memory,
    sense::Sense,
    upstream::UpstreamState,
    Intent,
};
use http_body_util::Full;
use hyper::{
    body::{Bytes, Incoming},
    Request, Response, StatusCode,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

/// Состояние admin API, доступное из всех обработчиков
#[derive(Clone)]
pub struct AdminState {
    pub upstreams: Arc<Vec<UpstreamState>>,
    pub memory: Arc<Memory>,
    pub sense: Arc<Sense>,
    pub align: Arc<Align>,
}

/// Точка входа: роутинг запросов admin API
pub async fn handle_admin(
    req: Request<Incoming>,
    state: AdminState,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let path = req.uri().path().to_string();
    let query = req.uri().query().unwrap_or("").to_string();

    let response = match path.as_str() {
        "/admin/upstreams" => handle_upstreams(&state),
        "/admin/explain" => handle_explain(&state, &query),
        "/admin/health" => json_ok(&serde_json::json!({"status": "ok", "version": dao_core::DAO_VERSION})),
        _ => json_err(StatusCode::NOT_FOUND, "endpoint not found"),
    };

    Ok(response)
}

fn handle_upstreams(state: &AdminState) -> Response<Full<Bytes>> {
    let resonance = state.sense.get_resonance_metrics();

    let upstreams: Vec<UpstreamInfo> = state
        .upstreams
        .iter()
        .map(|u| {
            let stats = u.get_stats();
            let rm = resonance.iter().find(|m| m.upstream_name == u.name);
            UpstreamInfo {
                name: u.name.clone(),
                url: u.url.clone(),
                p50_latency_ms: stats.p50_latency_ms(),
                p95_latency_ms: stats.p95_latency_ms(),
                error_rate: stats.error_rate(),
                current_rps: stats.current_rps(),
                success_count: stats.success_count,
                error_count: stats.error_count,
                load_resonance: rm.map(|m| m.load_resonance).unwrap_or(0.0),
                tempo_spikiness: rm.map(|m| m.tempo_spikiness).unwrap_or(0.0),
            }
        })
        .collect();

    json_ok(&UpstreamsResponse { upstreams })
}

fn handle_explain(state: &AdminState, query: &str) -> Response<Full<Bytes>> {
    let params = parse_query(query);
    let req = ExplainRequest {
        method: params.get("method").cloned(),
        path: params.get("path").cloned(),
        host: params.get("host").cloned(),
        intent: params.get("intent").cloned(),
    };

    let config = state.memory.get_config();

    // Поиск маршрута по host + path
    let matched = config.routes.rule.iter().find(|route| {
        if let Some(ref h) = route.match_rule.host {
            if req.host.as_deref() != Some(h.as_str()) {
                return false;
            }
        }
        if let Some(ref prefix) = route.match_rule.path_prefix {
            if !req.path.as_deref().unwrap_or("/").starts_with(prefix.as_str()) {
                return false;
            }
        }
        if let Some(ref exact) = route.match_rule.path_exact {
            if req.path.as_deref() != Some(exact.as_str()) {
                return false;
            }
        }
        true
    });

    let result = match matched {
        None => ExplainResult {
            selected: None,
            route: String::new(),
            policy: String::new(),
            request_intent: req.intent.clone(),
            weights: PolicyWeightsInfo { w_load: 0.6, w_intent: 0.3, w_tempo: 0.1 },
            candidates: vec![],
            no_route: true,
        },
        Some(route) => {
            let route_upstreams: Vec<_> = route
                .upstreams
                .iter()
                .filter_map(|uc| {
                    state
                        .upstreams
                        .iter()
                        .find(|u| u.name == uc.name)
                        .map(|u| Arc::new(u.clone()))
                })
                .collect();

            let intent_str = req.intent.as_ref().or(route.intent.as_ref());
            let intent_obj = intent_str.map(|s| Intent::new(s.clone()));

            state.align.explain_selection(
                &route.name,
                &route.policy,
                &route_upstreams,
                intent_obj.as_ref(),
            )
        }
    };

    json_ok(&result)
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn json_ok<T: serde::Serialize>(body: &T) -> Response<Full<Bytes>> {
    let json = serde_json::to_string_pretty(body).unwrap_or_else(|_| "{}".to_string());
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(json)))
        .unwrap()
}

fn json_err(status: StatusCode, msg: &str) -> Response<Full<Bytes>> {
    let json = serde_json::to_string(&serde_json::json!({"error": msg})).unwrap_or_default();
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(json)))
        .unwrap()
}

fn parse_query(query: &str) -> HashMap<String, String> {
    if query.is_empty() {
        return HashMap::new();
    }
    query
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = urldecode(parts.next()?);
            let val = urldecode(parts.next().unwrap_or(""));
            if key.is_empty() { None } else { Some((key, val)) }
        })
        .collect()
}

fn urldecode(s: &str) -> String {
    let s = s.replace('+', " ");
    let bytes = s.as_bytes();
    let mut result = String::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Ok(hex), true) = (
                std::str::from_utf8(&bytes[i + 1..i + 3]),
                bytes[i + 1].is_ascii_hexdigit() && bytes[i + 2].is_ascii_hexdigit(),
            ) {
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    result.push(byte as char);
                    i += 3;
                    continue;
                }
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

/// Запуск admin HTTP сервера (блокирующий)
pub async fn start_admin_api(bind_addr: SocketAddr, state: AdminState) -> anyhow::Result<()> {
    use hyper::server::conn::http1;
    use hyper::service::service_fn;
    use hyper_util::rt::TokioIo;
    use tokio::net::TcpListener;

    let listener = TcpListener::bind(bind_addr).await?;
    tracing::info!("Admin API listening on http://{}", bind_addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let state = state.clone();

        tokio::spawn(async move {
            let svc = service_fn(move |req| {
                let state = state.clone();
                handle_admin(req, state)
            });
            if let Err(e) = http1::Builder::new().serve_connection(io, svc).await {
                tracing::debug!("Admin conn error: {}", e);
            }
        });
    }
}
