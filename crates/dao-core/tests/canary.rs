//! Тесты canary routing
//!
//! Сценарии:
//! 1. Weighted random: вес 95/5 даёт ~95%/~5% за N запросов
//! 2. Нулевой вес — upstream не выбирается
//! 3. Один upstream с весом — всегда выбирается он
//! 4. Canary обходит upstream с открытым circuit
//! 5. Все circuit открыты — fallback на всех
//! 6. Align.select_upstream диспатчит на canary при policy="canary"
//! 7. explain_canary показывает правильные weight_pct

use dao_core::{
    align::{canary_select, Align, PolicyWeights},
    sense::Sense,
    upstream::{CircuitBreakerConfig, UpstreamState},
    Intent,
};
use std::sync::Arc;
use std::time::Duration;

fn up(name: &str, weight: u32) -> UpstreamState {
    let mut u = UpstreamState::new(
        name.to_string(),
        format!("http://127.0.0.1:808{}", name.len() % 9),
        vec![],
        weight,
    );
    // Заменяем weight через with_circuit (weight задаётся в new)
    u
}

fn up_w(name: &str, weight: u32) -> UpstreamState {
    UpstreamState::with_circuit(
        name.to_string(),
        format!("http://127.0.0.1:808{}", name.len() % 9),
        vec![],
        weight,
        CircuitBreakerConfig::default(),
    )
}

fn arcs(us: &[UpstreamState]) -> Vec<Arc<UpstreamState>> {
    us.iter().cloned().map(Arc::new).collect()
}

fn make_align(upstreams: Arc<Vec<UpstreamState>>) -> Align {
    let sense = Sense::new(upstreams);
    Align::new(sense)
}

// ── 1. Weighted distribution ──────────────────────────────────────────────────

#[test]
fn test_canary_distribution_matches_weights() {
    let upstreams = vec![
        up_w("stable", 95),
        up_w("canary", 5),
    ];
    let arcs = arcs(&upstreams);

    let n = 10_000;
    let mut stable_count = 0usize;
    let mut canary_count = 0usize;

    for _ in 0..n {
        match canary_select(&arcs).map(|u| u.name.clone()).as_deref() {
            Some("stable") => stable_count += 1,
            Some("canary") => canary_count += 1,
            _ => {}
        }
    }

    let stable_pct = stable_count as f64 / n as f64 * 100.0;
    let canary_pct = canary_count as f64 / n as f64 * 100.0;

    // Допускаем отклонение ±3pp при n=10000
    assert!(stable_pct > 92.0 && stable_pct < 98.0,
        "stable должен получать ~95%, получил {:.1}%", stable_pct);
    assert!(canary_pct > 2.0 && canary_pct < 8.0,
        "canary должен получать ~5%, получил {:.1}%", canary_pct);
}

#[test]
fn test_canary_equal_weights_roughly_equal() {
    let upstreams = vec![
        up_w("a", 1),
        up_w("b", 1),
    ];
    let arcs = arcs(&upstreams);

    let n = 2000;
    let mut a = 0usize;
    let mut b = 0usize;
    for _ in 0..n {
        match canary_select(&arcs).map(|u| u.name.clone()).as_deref() {
            Some("a") => a += 1,
            Some("b") => b += 1,
            _ => {}
        }
    }

    let a_pct = a as f64 / n as f64 * 100.0;
    assert!(a_pct > 45.0 && a_pct < 55.0,
        "при равных весах каждый должен получать ~50%, a={:.1}%", a_pct);
}

// ── 2. Нулевой вес ────────────────────────────────────────────────────────────

#[test]
fn test_canary_zero_weight_never_selected() {
    let upstreams = vec![
        up_w("active",  10),
        up_w("dormant",  0),
    ];
    let arcs = arcs(&upstreams);

    for _ in 0..200 {
        let selected = canary_select(&arcs).unwrap();
        assert_eq!(selected.name, "active",
            "upstream с weight=0 не должен выбираться");
    }
}

// ── 3. Единственный upstream ──────────────────────────────────────────────────

#[test]
fn test_canary_single_upstream_always_selected() {
    let upstreams = vec![up_w("only", 1)];
    let arcs = arcs(&upstreams);

    for _ in 0..10 {
        let selected = canary_select(&arcs).unwrap();
        assert_eq!(selected.name, "only");
    }
}

// ── 4. Обход открытого circuit ────────────────────────────────────────────────

#[test]
fn test_canary_skips_open_circuit() {
    let upstreams = vec![
        UpstreamState::with_circuit(
            "stable".to_string(), "http://127.0.0.1:8081".to_string(), vec![], 95,
            CircuitBreakerConfig { failure_threshold: 1, success_threshold: 1, timeout: Duration::from_secs(60) },
        ),
        UpstreamState::with_circuit(
            "canary".to_string(), "http://127.0.0.1:8082".to_string(), vec![], 5,
            CircuitBreakerConfig { failure_threshold: 1, success_threshold: 1, timeout: Duration::from_secs(60) },
        ),
    ];

    // Открываем circuit у canary
    upstreams[1].record_request(Duration::from_millis(1), false);
    assert!(!upstreams[1].is_available());

    let arcs = arcs(&upstreams);

    for _ in 0..50 {
        let selected = canary_select(&arcs).unwrap();
        assert_eq!(selected.name, "stable",
            "canary с открытым circuit не должен выбираться");
    }
}

// ── 5. Все circuit открыты → fallback ─────────────────────────────────────────

#[test]
fn test_canary_fallback_when_all_circuits_open() {
    let upstreams = vec![
        UpstreamState::with_circuit(
            "a".to_string(), "http://127.0.0.1:8081".to_string(), vec![], 1,
            CircuitBreakerConfig { failure_threshold: 1, success_threshold: 1, timeout: Duration::from_secs(60) },
        ),
        UpstreamState::with_circuit(
            "b".to_string(), "http://127.0.0.1:8082".to_string(), vec![], 1,
            CircuitBreakerConfig { failure_threshold: 1, success_threshold: 1, timeout: Duration::from_secs(60) },
        ),
    ];

    upstreams[0].record_request(Duration::from_millis(1), false);
    upstreams[1].record_request(Duration::from_millis(1), false);
    assert!(!upstreams[0].is_available());
    assert!(!upstreams[1].is_available());

    let arcs = arcs(&upstreams);
    let selected = canary_select(&arcs);
    assert!(selected.is_some(), "при всех открытых circuit должен вернуть fallback");
}

// ── 6. Align диспатчит на canary ──────────────────────────────────────────────

#[test]
fn test_align_dispatches_canary_policy() {
    let upstreams = vec![
        up_w("stable", 90),
        up_w("canary",  10),
    ];
    let ups = Arc::new(upstreams.clone());
    let align = make_align(ups);
    let arcs = arcs(&upstreams);

    // С policy="canary" должен возвращать результат
    let selected = align.select_upstream("canary", &arcs, None);
    assert!(selected.is_some(), "canary policy должна выбирать upstream");

    // За 100 запросов должны получить оба
    let mut names = std::collections::HashSet::new();
    for _ in 0..100 {
        if let Some(u) = align.select_upstream("canary", &arcs, None) {
            names.insert(u.name.clone());
        }
    }
    assert!(names.contains("stable"));
    assert!(names.contains("canary"),
        "при weight=10 canary должен появляться хотя бы раз за 100 запросов");
}

// ── 7. explain_canary ─────────────────────────────────────────────────────────

#[test]
fn test_explain_canary_weight_pct() {
    let upstreams = vec![
        up_w("stable", 95),
        up_w("canary",  5),
    ];
    let ups = Arc::new(upstreams.clone());
    let align = make_align(ups);
    let arcs = arcs(&upstreams);

    let info = align.explain_canary("test-route", &arcs);

    assert_eq!(info.upstreams.len(), 2);

    let stable = info.upstreams.iter().find(|u| u.name == "stable").unwrap();
    let canary = info.upstreams.iter().find(|u| u.name == "canary").unwrap();

    assert!((stable.weight_pct - 95.0).abs() < 0.01,
        "stable weight_pct должен быть ~95%, получил {}", stable.weight_pct);
    assert!((canary.weight_pct - 5.0).abs() < 0.01,
        "canary weight_pct должен быть ~5%, получил {}", canary.weight_pct);

    assert_eq!(stable.weight, 95);
    assert_eq!(canary.weight, 5);
}

#[test]
fn test_explain_canary_requests_pct_after_traffic() {
    let upstreams = vec![
        up_w("stable", 1),
        up_w("canary", 1),
    ];

    // Симулируем 7 запросов к stable, 3 к canary
    for _ in 0..7 { upstreams[0].record_request(Duration::from_millis(10), true); }
    for _ in 0..3 { upstreams[1].record_request(Duration::from_millis(10), true); }

    let ups = Arc::new(upstreams.clone());
    let align = make_align(ups);
    let arcs = arcs(&upstreams);

    let info = align.explain_canary("route", &arcs);
    let stable = info.upstreams.iter().find(|u| u.name == "stable").unwrap();
    let canary = info.upstreams.iter().find(|u| u.name == "canary").unwrap();

    assert_eq!(stable.requests_total, 7);
    assert_eq!(canary.requests_total, 3);
    assert!((stable.requests_pct - 70.0).abs() < 0.01);
    assert!((canary.requests_pct - 30.0).abs() < 0.01);
}

// ── 8. Пустой список ─────────────────────────────────────────────────────────

#[test]
fn test_canary_empty_returns_none() {
    let result = canary_select(&[]);
    assert!(result.is_none());
}
