//! End-to-end тесты DAO
//!
//! Сценарии:
//! 1. Admin API health — /admin/health отвечает
//! 2. Admin upstreams — /admin/upstreams возвращает JSON
//! 3. Admin explain — /admin/explain объясняет выбор
//! 4. Proxy request — запрос проходит через DAO к mock backend
//! 5. Explain no_route — неизвестный host возвращает no_route=true

use dao_core::{
    align::Align,
    config::{DaoConfig, MatchRule, PolicyConfig, RouteRule, RoutesConfig, ServerConfig, TelemetryConfig, UpstreamConfig},
    memory::Memory,
    sense::Sense,
    upstream::UpstreamState,
    Intent,
};
use dao_admin::{AdminState, start_admin_api};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// ── helpers ───────────────────────────────────────────────────────────────────

/// Запустить mock HTTP backend, отвечающий статусом 200
async fn start_mock_backend() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 4096];
                    let _ = stream.read(&mut buf).await;
                    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Type: text/plain\r\n\r\nok";
                    let _ = stream.write_all(response).await;
                });
            }
        }
    });

    addr
}

/// Создать конфиг, указывающий на два backend-адреса
fn make_test_config(
    admin_addr: SocketAddr,
    backend1: SocketAddr,
    backend2: SocketAddr,
) -> DaoConfig {
    let mut policies = HashMap::new();
    policies.insert("resonant".to_string(), PolicyConfig::default());

    DaoConfig {
        server: ServerConfig {
            bind: "127.0.0.1:0".to_string(),
            tls_cert: None,
            tls_key: None,
            workers: 1,
            admin_bind: Some(admin_addr.to_string()),
        },
        telemetry: None,
        routes: RoutesConfig {
            rule: vec![
                RouteRule {
                    name: "llm-api".to_string(),
                    match_rule: MatchRule {
                        host: Some("llm.test".to_string()),
                        path_prefix: Some("/v1/".to_string()),
                        path_exact: None,
                        upgrade: None,
                        headers: None,
                    },
                    policy: "resonant".to_string(),
                    intent: Some("realtime".to_string()),
                    upstreams: vec![
                        UpstreamConfig {
                            name: "fast-model".to_string(),
                            url: format!("http://{}", backend1),
                            intent: Some(vec!["realtime".to_string(), "low-latency".to_string()]),
                            weight: 2,
                            circuit_breaker: None,
                            health_check: None,
                        },
                        UpstreamConfig {
                            name: "slow-model".to_string(),
                            url: format!("http://{}", backend2),
                            intent: Some(vec!["batch".to_string()]),
                            weight: 1,
                            circuit_breaker: None,
                            health_check: None,
                        },
                    ],
                    filters: None,
                    retry_attempts: 0,
                },
            ],
        },
        policies: Some(policies),
    }
}

/// Запустить admin API и вернуть его адрес
async fn start_test_admin(
    backend1: SocketAddr,
    backend2: SocketAddr,
) -> SocketAddr {
    let admin_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let admin_addr = admin_listener.local_addr().unwrap();
    drop(admin_listener); // освобождаем порт — start_admin_api займёт его сам

    let config = make_test_config(admin_addr, backend1, backend2);

    let upstreams: Arc<Vec<UpstreamState>> = Arc::new(
        config.routes.rule.iter()
            .flat_map(|r| r.upstreams.iter())
            .map(|uc| UpstreamState::new(
                uc.name.clone(),
                uc.url.clone(),
                uc.intents(),
                uc.weight,
            ))
            .collect()
    );

    let memory = Arc::new(Memory::new(config));
    let sense = Sense::new(upstreams.clone());
    let align = Align::new(sense.clone());

    let state = AdminState {
        upstreams,
        memory,
        sense: Arc::new(sense),
        align: Arc::new(align),
    };

    tokio::spawn(async move {
        if let Err(e) = start_admin_api(admin_addr, state).await {
            eprintln!("Admin API error: {}", e);
        }
    });

    // Дать серверу время стартовать
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    admin_addr
}

/// Отправить HTTP GET запрос и вернуть тело ответа
async fn http_get(addr: SocketAddr, path: &str) -> anyhow::Result<(u16, String)> {
    use tokio::net::TcpStream;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut stream = TcpStream::connect(addr).await?;
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        path, addr
    );
    stream.write_all(request.as_bytes()).await?;

    let mut response = Vec::new();
    stream.read_to_end(&mut response).await?;
    let response_str = String::from_utf8_lossy(&response).into_owned();

    // Парсим статус и тело
    let status: u16 = response_str
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let body = response_str
        .split("\r\n\r\n")
        .nth(1)
        .unwrap_or("")
        .to_string();

    Ok((status, body))
}

// ── тесты ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_admin_health_returns_ok() {
    let b1 = start_mock_backend().await;
    let b2 = start_mock_backend().await;
    let admin_addr = start_test_admin(b1, b2).await;

    let (status, body) = http_get(admin_addr, "/admin/health").await.unwrap();

    assert_eq!(status, 200);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_admin_upstreams_returns_both_backends() {
    let b1 = start_mock_backend().await;
    let b2 = start_mock_backend().await;
    let admin_addr = start_test_admin(b1, b2).await;

    let (status, body) = http_get(admin_addr, "/admin/upstreams").await.unwrap();

    assert_eq!(status, 200);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    let upstreams = json["upstreams"].as_array().unwrap();
    assert_eq!(upstreams.len(), 2, "должны быть оба backend");

    let names: Vec<_> = upstreams.iter()
        .map(|u| u["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"fast-model"));
    assert!(names.contains(&"slow-model"));
}

#[tokio::test]
async fn test_admin_upstreams_has_metric_fields() {
    let b1 = start_mock_backend().await;
    let b2 = start_mock_backend().await;
    let admin_addr = start_test_admin(b1, b2).await;

    let (_, body) = http_get(admin_addr, "/admin/upstreams").await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let upstream = &json["upstreams"][0];

    // Все метрические поля должны присутствовать
    for field in &["p50_latency_ms", "p95_latency_ms", "error_rate",
                   "current_rps", "success_count", "error_count",
                   "load_resonance", "tempo_spikiness"] {
        assert!(upstream.get(field).is_some(), "поле {} должно присутствовать", field);
    }
}

#[tokio::test]
async fn test_admin_explain_matched_route() {
    let b1 = start_mock_backend().await;
    let b2 = start_mock_backend().await;
    let admin_addr = start_test_admin(b1, b2).await;

    let (status, body) = http_get(
        admin_addr,
        "/admin/explain?host=llm.test&path=/v1/chat&intent=realtime",
    ).await.unwrap();

    assert_eq!(status, 200);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["no_route"], false, "маршрут должен найтись");
    assert_eq!(json["route"], "llm-api");
    assert_eq!(json["policy"], "resonant");
    assert_eq!(json["request_intent"], "realtime");
    assert!(json["selected"].is_string(), "должен быть выбран upstream");

    let candidates = json["candidates"].as_array().unwrap();
    assert_eq!(candidates.len(), 2, "оба кандидата должны быть в explain");
}

#[tokio::test]
async fn test_admin_explain_winner_is_marked() {
    let b1 = start_mock_backend().await;
    let b2 = start_mock_backend().await;
    let admin_addr = start_test_admin(b1, b2).await;

    let (_, body) = http_get(
        admin_addr,
        "/admin/explain?host=llm.test&path=/v1/chat",
    ).await.unwrap();

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let candidates = json["candidates"].as_array().unwrap();

    let winners: Vec<_> = candidates.iter()
        .filter(|c| c["winner"].as_bool().unwrap_or(false))
        .collect();

    assert_eq!(winners.len(), 1, "ровно один победитель в explain");

    let selected = json["selected"].as_str().unwrap();
    assert_eq!(winners[0]["name"].as_str().unwrap(), selected,
        "победитель в candidates совпадает с selected");
}

#[tokio::test]
async fn test_admin_explain_realtime_intent_picks_fast_model() {
    let b1 = start_mock_backend().await;
    let b2 = start_mock_backend().await;
    let admin_addr = start_test_admin(b1, b2).await;

    // intent=realtime → должен выбрать fast-model (intent=["realtime","low-latency"])
    let (_, body) = http_get(
        admin_addr,
        "/admin/explain?host=llm.test&path=/v1/chat&intent=realtime",
    ).await.unwrap();

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["selected"], "fast-model",
        "realtime intent должен выбирать fast-model");
}

#[tokio::test]
async fn test_admin_explain_no_route_for_unknown_host() {
    let b1 = start_mock_backend().await;
    let b2 = start_mock_backend().await;
    let admin_addr = start_test_admin(b1, b2).await;

    let (status, body) = http_get(
        admin_addr,
        "/admin/explain?host=unknown.host&path=/v1/chat",
    ).await.unwrap();

    assert_eq!(status, 200);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["no_route"], true,
        "неизвестный host должен вернуть no_route=true");
    assert!(json["selected"].is_null(),
        "selected должен быть null при no_route");
}

#[tokio::test]
async fn test_admin_explain_score_components_present() {
    let b1 = start_mock_backend().await;
    let b2 = start_mock_backend().await;
    let admin_addr = start_test_admin(b1, b2).await;

    let (_, body) = http_get(
        admin_addr,
        "/admin/explain?host=llm.test&path=/v1/chat&intent=realtime",
    ).await.unwrap();

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let candidate = &json["candidates"][0];

    assert!(candidate["components"]["load_component"].is_number());
    assert!(candidate["components"]["intent_component"].is_number());
    assert!(candidate["components"]["tempo_component"].is_number());
    assert!(candidate["total_score"].is_number());
    assert!(candidate["p95_latency_ms"].is_number());
    assert!(candidate["error_rate"].is_number());
}

#[tokio::test]
async fn test_admin_unknown_endpoint_returns_404() {
    let b1 = start_mock_backend().await;
    let b2 = start_mock_backend().await;
    let admin_addr = start_test_admin(b1, b2).await;

    let (status, _) = http_get(admin_addr, "/admin/doesnotexist").await.unwrap();
    assert_eq!(status, 404);
}

#[tokio::test]
async fn test_admin_weights_in_explain_match_policy() {
    let b1 = start_mock_backend().await;
    let b2 = start_mock_backend().await;
    let admin_addr = start_test_admin(b1, b2).await;

    let (_, body) = http_get(
        admin_addr,
        "/admin/explain?host=llm.test&path=/v1/chat",
    ).await.unwrap();

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let weights = &json["weights"];

    // Дефолтные веса политики resonant
    let w_load   = weights["w_load"].as_f64().unwrap();
    let w_intent = weights["w_intent"].as_f64().unwrap();
    let w_tempo  = weights["w_tempo"].as_f64().unwrap();

    assert!((w_load + w_intent + w_tempo) > 0.0, "веса должны быть ненулевыми");
    assert_eq!(w_load, 0.6);
    assert_eq!(w_intent, 0.3);
    assert_eq!(w_tempo, 0.1);
}
