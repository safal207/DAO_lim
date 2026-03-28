//! Тесты circuit breaker
//!
//! Сценарии:
//! 1. Closed → Open при достижении failure_threshold
//! 2. Open блокирует запросы
//! 3. Open → HalfOpen после timeout
//! 4. HalfOpen → Closed при достаточном количестве успехов
//! 5. HalfOpen → Open при ошибке
//! 6. Routing обходит upstream с открытым circuit
//! 7. Если все circuit открыты — всё равно возвращает upstream

use dao_core::{
    align::{Align, PolicyWeights},
    sense::Sense,
    upstream::{CircuitBreaker, CircuitBreakerConfig, CircuitStatus, UpstreamState},
    Intent,
};
use std::sync::Arc;
use std::time::Duration;

fn fast_cb() -> CircuitBreakerConfig {
    CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout: Duration::from_millis(50), // очень короткий таймаут для тестов
    }
}

fn make_upstream_with_cb(name: &str, intents: &[&str]) -> UpstreamState {
    UpstreamState::with_circuit(
        name.to_string(),
        format!("http://127.0.0.1:808{}", name.len()),
        intents.iter().map(|s| Intent::new(*s)).collect(),
        1,
        fast_cb(),
    )
}

fn arcs(upstreams: &[UpstreamState]) -> Vec<Arc<UpstreamState>> {
    upstreams.iter().cloned().map(Arc::new).collect()
}

fn make_align(upstreams: Arc<Vec<UpstreamState>>) -> Align {
    let sense = Sense::new(upstreams.clone());
    let mut align = Align::new(sense);
    align.register_policy("resonant".to_string(), PolicyWeights::default());
    align
}

// ── 1. Closed → Open ─────────────────────────────────────────────────────────

#[test]
fn test_cb_opens_after_failure_threshold() {
    let cb = CircuitBreaker::new(fast_cb());

    assert!(cb.is_available(), "изначально цепь закрыта");

    // Две ошибки — цепь ещё закрыта
    cb.record_failure();
    cb.record_failure();
    assert!(cb.is_available(), "цепь закрыта после 2 ошибок (threshold=3)");

    // Третья ошибка — цепь открывается
    cb.record_failure();
    assert!(!cb.is_available(), "цепь должна открыться после 3 ошибок");
    assert!(cb.is_open());
}

#[test]
fn test_cb_success_resets_failure_count() {
    let cb = CircuitBreaker::new(fast_cb());

    cb.record_failure();
    cb.record_failure();
    cb.record_success(); // сброс счётчика

    cb.record_failure();
    cb.record_failure();
    assert!(cb.is_available(), "счётчик ошибок должен был сброситься после успеха");

    cb.record_failure(); // теперь снова threshold=3
    assert!(!cb.is_available(), "цепь открывается после 3 ошибок подряд");
}

// ── 2. Open блокирует запросы ─────────────────────────────────────────────────

#[test]
fn test_cb_open_blocks_requests() {
    let cb = CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 1,
        success_threshold: 1,
        timeout: Duration::from_secs(60), // долгий таймаут
    });

    cb.record_failure();
    assert!(!cb.is_available(), "Open должен блокировать запросы");

    let status = cb.status();
    assert_eq!(status.label(), "open");
    assert!(!status.is_available());
}

// ── 3. Open → HalfOpen после timeout ─────────────────────────────────────────

#[test]
fn test_cb_transitions_to_half_open_after_timeout() {
    let cb = CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 1,
        success_threshold: 2,
        timeout: Duration::from_millis(10),
    });

    cb.record_failure();
    assert!(!cb.is_available(), "сразу после открытия — заблокирован");

    std::thread::sleep(Duration::from_millis(20));

    assert!(cb.is_available(), "после timeout должен перейти в HalfOpen");
    assert_eq!(cb.status().label(), "half_open");
}

// ── 4. HalfOpen → Closed ──────────────────────────────────────────────────────

#[test]
fn test_cb_closes_after_success_threshold_in_half_open() {
    let cb = CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 1,
        success_threshold: 2,
        timeout: Duration::from_millis(10),
    });

    cb.record_failure();
    std::thread::sleep(Duration::from_millis(20));
    assert!(cb.is_available(), "после timeout is_available() переводит в HalfOpen");
    assert_eq!(cb.status().label(), "half_open");

    cb.record_success(); // 1 из 2 успехов
    assert_eq!(cb.status().label(), "half_open", "нужно 2 успеха для закрытия");

    cb.record_success(); // 2 из 2
    assert_eq!(cb.status().label(), "closed", "должен закрыться после 2 успехов");
    assert!(cb.is_available());
}

// ── 5. HalfOpen → Open при ошибке ────────────────────────────────────────────

#[test]
fn test_cb_reopens_on_failure_in_half_open() {
    let cb = CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 1,
        success_threshold: 2,
        timeout: Duration::from_millis(10),
    });

    cb.record_failure();
    std::thread::sleep(Duration::from_millis(20));
    assert!(cb.is_available(), "после timeout is_available() переводит в HalfOpen");
    assert_eq!(cb.status().label(), "half_open");

    cb.record_failure(); // пробный запрос провалился
    assert_eq!(cb.status().label(), "open", "пробный провал должен вернуть в Open");
    assert!(!cb.is_available());
}

// ── 6. Routing обходит открытый circuit ───────────────────────────────────────

#[test]
fn test_routing_skips_upstream_with_open_circuit() {
    let upstreams = vec![
        make_upstream_with_cb("healthy", &["realtime"]),
        make_upstream_with_cb("broken",  &["realtime"]),
    ];

    // Открываем circuit у "broken"
    for _ in 0..3 {
        upstreams[1].record_request(Duration::from_millis(10), false);
    }
    assert!(!upstreams[1].is_available(), "circuit 'broken' должен быть открыт");
    assert!(upstreams[0].is_available(), "circuit 'healthy' должен быть закрыт");

    let ups = Arc::new(upstreams.clone());
    let align = make_align(ups);
    let arcs = arcs(&upstreams);

    let selected = align
        .select_upstream("resonant", &arcs, Some(&Intent::new("realtime")))
        .expect("должен выбрать upstream");

    assert_eq!(selected.name, "healthy",
        "должен выбрать healthy, игнорируя broken с открытым circuit");
}

#[test]
fn test_routing_explain_marks_open_circuit() {
    let upstreams = vec![
        make_upstream_with_cb("good",   &["realtime"]),
        make_upstream_with_cb("broken", &["realtime"]),
    ];

    for _ in 0..3 {
        upstreams[1].record_request(Duration::from_millis(1), false);
    }

    let ups = Arc::new(upstreams.clone());
    let align = make_align(ups);
    let arcs = arcs(&upstreams);

    let result = align.explain_selection("r", "resonant", &arcs, Some(&Intent::new("realtime")));

    let broken = result.candidates.iter().find(|c| c.name == "broken").unwrap();
    assert!(broken.circuit_open, "broken должен быть помечен circuit_open=true в explain");
    assert_eq!(broken.circuit.label(), "open");

    let good = result.candidates.iter().find(|c| c.name == "good").unwrap();
    assert!(!good.circuit_open, "good должен быть circuit_open=false");
    assert_eq!(result.selected.as_deref(), Some("good"));
}

// ── 7. Все circuit открыты → всё равно выбираем upstream ─────────────────────

#[test]
fn test_routing_fallback_when_all_circuits_open() {
    let upstreams = vec![
        make_upstream_with_cb("a", &["realtime"]),
        make_upstream_with_cb("b", &["realtime"]),
    ];

    // Открываем оба
    for _ in 0..3 {
        upstreams[0].record_request(Duration::from_millis(1), false);
        upstreams[1].record_request(Duration::from_millis(1), false);
    }
    assert!(!upstreams[0].is_available());
    assert!(!upstreams[1].is_available());

    let ups = Arc::new(upstreams.clone());
    let align = make_align(ups);
    let arcs = arcs(&upstreams);

    // Лучше попробовать чем отдать 503
    let selected = align.select_upstream("resonant", &arcs, None);
    assert!(selected.is_some(),
        "при всех открытых circuit всё равно должен вернуть upstream");
}

// ── 8. UpstreamState интеграция ───────────────────────────────────────────────

#[test]
fn test_upstream_record_request_updates_circuit() {
    let upstream = UpstreamState::with_circuit(
        "test".to_string(),
        "http://127.0.0.1:8080".to_string(),
        vec![],
        1,
        CircuitBreakerConfig { failure_threshold: 2, success_threshold: 1, timeout: Duration::from_secs(60) },
    );

    assert!(upstream.is_available());

    upstream.record_request(Duration::from_millis(10), false);
    upstream.record_request(Duration::from_millis(10), false);

    assert!(!upstream.is_available(), "после 2 ошибок circuit должен открыться");

    let status = upstream.circuit_status();
    assert_eq!(status.label(), "open");
}

#[test]
fn test_upstream_circuit_status_closed_by_default() {
    let upstream = UpstreamState::new(
        "default".to_string(),
        "http://127.0.0.1:8080".to_string(),
        vec![],
        1,
    );

    assert!(upstream.is_available());
    assert_eq!(upstream.circuit_status().label(), "closed");
}

// ── 9. CircuitStatus serde ────────────────────────────────────────────────────

#[test]
fn test_circuit_status_serializes_correctly() {
    let closed = CircuitStatus::Closed { consecutive_failures: 0 };
    let json = serde_json::to_string(&closed).unwrap();
    assert!(json.contains("\"state\":\"closed\""));

    let open = CircuitStatus::Open { opened_secs_ago: 5, recovers_in_secs: 25 };
    let json = serde_json::to_string(&open).unwrap();
    assert!(json.contains("\"state\":\"open\""));

    let half = CircuitStatus::HalfOpen { consecutive_successes: 1 };
    let json = serde_json::to_string(&half).unwrap();
    assert!(json.contains("\"state\":\"half_open\""));
}
