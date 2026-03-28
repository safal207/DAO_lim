//! Интеграционные тесты маршрутизации
//!
//! Сценарии:
//! 1. Intent routing — backend с совпадающим intent побеждает
//! 2. Load routing — backend с меньшей латентностью побеждает
//! 3. Fallback on errors — перегруженный backend проигрывает
//! 4. Explain output — explain показывает все кандидаты и победителя
//! 5. Combined scoring — учёт всех трёх компонентов score

use dao_core::{
    align::{Align, PolicyWeights},
    sense::Sense,
    upstream::UpstreamState,
    Intent,
};
use std::sync::Arc;
use std::time::Duration;

// ── helpers ───────────────────────────────────────────────────────────────────

fn make_upstream(name: &str, url: &str, intents: &[&str]) -> UpstreamState {
    UpstreamState::new(
        name.to_string(),
        url.to_string(),
        intents.iter().map(|s| Intent::new(*s)).collect(),
        1,
    )
}

fn make_align(upstreams: Arc<Vec<UpstreamState>>) -> Align {
    let sense = Sense::new(upstreams.clone());
    let mut align = Align::new(sense);
    align.register_policy(
        "resonant".to_string(),
        PolicyWeights { w_load: 0.6, w_intent: 0.3, w_tempo: 0.1 },
    );
    align.register_policy(
        "intent-first".to_string(),
        PolicyWeights { w_load: 0.2, w_intent: 0.7, w_tempo: 0.1 },
    );
    align
}

fn arcs(upstreams: &[UpstreamState]) -> Vec<Arc<UpstreamState>> {
    upstreams.iter().cloned().map(Arc::new).collect()
}

// ── 1. Intent routing ─────────────────────────────────────────────────────────

/// Backend с совпадающим intent выбирается при нулевой нагрузке
#[test]
fn test_routing_prefers_intent_match() {
    let upstreams = vec![
        make_upstream("realtime-backend", "http://127.0.0.1:8081", &["realtime", "low-latency"]),
        make_upstream("batch-backend",    "http://127.0.0.1:8082", &["batch"]),
    ];
    let ups = Arc::new(upstreams.clone());
    let align = make_align(ups);
    let arcs = arcs(&upstreams);

    let selected = align
        .select_upstream("resonant", &arcs, Some(&Intent::new("realtime")))
        .expect("должен выбрать upstream");

    assert_eq!(selected.name, "realtime-backend",
        "intent=realtime должен выбрать realtime-backend");
}

#[test]
fn test_routing_batch_intent() {
    let upstreams = vec![
        make_upstream("realtime-backend", "http://127.0.0.1:8081", &["realtime"]),
        make_upstream("batch-backend",    "http://127.0.0.1:8082", &["batch", "high-throughput"]),
    ];
    let ups = Arc::new(upstreams.clone());
    let align = make_align(ups);
    let arcs = arcs(&upstreams);

    let selected = align
        .select_upstream("resonant", &arcs, Some(&Intent::new("batch")))
        .expect("должен выбрать upstream");

    assert_eq!(selected.name, "batch-backend");
}

/// Без intent — первый по алфавиту score не влияет, выбирается любой
#[test]
fn test_routing_no_intent_selects_some() {
    let upstreams = vec![
        make_upstream("backend-a", "http://127.0.0.1:8081", &["realtime"]),
        make_upstream("backend-b", "http://127.0.0.1:8082", &["batch"]),
    ];
    let ups = Arc::new(upstreams.clone());
    let align = make_align(ups);
    let arcs = arcs(&upstreams);

    let selected = align.select_upstream("resonant", &arcs, None);
    assert!(selected.is_some(), "без intent всё равно должен выбрать upstream");
}

// ── 2. Load routing ───────────────────────────────────────────────────────────

/// Backend с низкой латентностью побеждает при одинаковых intent
#[test]
fn test_routing_prefers_lower_latency() {
    let upstreams = vec![
        make_upstream("fast",  "http://127.0.0.1:8081", &["realtime"]),
        make_upstream("slow",  "http://127.0.0.1:8082", &["realtime"]),
    ];

    // Нагружаем slow большими латентностями
    for _ in 0..20 {
        upstreams[1].record_request(Duration::from_millis(500), true);
    }
    // fast — быстрые запросы
    for _ in 0..20 {
        upstreams[0].record_request(Duration::from_millis(10), true);
    }

    let ups = Arc::new(upstreams.clone());
    let align = make_align(ups);
    let arcs = arcs(&upstreams);

    let selected = align
        .select_upstream("resonant", &arcs, Some(&Intent::new("realtime")))
        .expect("должен выбрать upstream");

    assert_eq!(selected.name, "fast",
        "должен выбрать backend с меньшей латентностью");
}

// ── 3. Fallback on errors ─────────────────────────────────────────────────────

/// Backend с высоким error rate проигрывает здоровому
#[test]
fn test_routing_avoids_high_error_rate() {
    let upstreams = vec![
        make_upstream("healthy",  "http://127.0.0.1:8081", &["realtime"]),
        make_upstream("degraded", "http://127.0.0.1:8082", &["realtime"]),
    ];

    // degraded: 50% ошибок
    for i in 0..20 {
        upstreams[1].record_request(Duration::from_millis(50), i % 2 == 0);
    }
    // healthy: без ошибок
    for _ in 0..20 {
        upstreams[0].record_request(Duration::from_millis(50), true);
    }

    let ups = Arc::new(upstreams.clone());
    let align = make_align(ups);
    let arcs = arcs(&upstreams);

    let selected = align
        .select_upstream("resonant", &arcs, Some(&Intent::new("realtime")))
        .expect("должен выбрать upstream");

    assert_eq!(selected.name, "healthy",
        "должен избегать backend с высоким error rate");
}

/// Если все перегружены — выбирает менее плохой
#[test]
fn test_routing_picks_least_bad_when_all_degraded() {
    let upstreams = vec![
        make_upstream("bad",  "http://127.0.0.1:8081", &["realtime"]),
        make_upstream("worse","http://127.0.0.1:8082", &["realtime"]),
    ];

    // bad: 30% ошибок
    for i in 0..10 {
        upstreams[0].record_request(Duration::from_millis(100), i % 3 != 0);
    }
    // worse: 70% ошибок
    for i in 0..10 {
        upstreams[1].record_request(Duration::from_millis(100), i % 10 < 3);
    }

    let ups = Arc::new(upstreams.clone());
    let align = make_align(ups);
    let arcs = arcs(&upstreams);

    let selected = align
        .select_upstream("resonant", &arcs, Some(&Intent::new("realtime")))
        .expect("должен выбрать upstream даже если оба плохие");

    assert_eq!(selected.name, "bad",
        "должен выбрать менее плохой backend");
}

// ── 4. Explain output ─────────────────────────────────────────────────────────

/// explain_selection возвращает всех кандидатов
#[test]
fn test_explain_includes_all_candidates() {
    let upstreams = vec![
        make_upstream("backend-a", "http://127.0.0.1:8081", &["realtime"]),
        make_upstream("backend-b", "http://127.0.0.1:8082", &["batch"]),
        make_upstream("backend-c", "http://127.0.0.1:8083", &["realtime", "batch"]),
    ];
    let ups = Arc::new(upstreams.clone());
    let align = make_align(ups);
    let arcs = arcs(&upstreams);

    let result = align.explain_selection(
        "test-route",
        "resonant",
        &arcs,
        Some(&Intent::new("realtime")),
    );

    assert_eq!(result.candidates.len(), 3, "должны быть все 3 кандидата");
    assert_eq!(result.route, "test-route");
    assert_eq!(result.policy, "resonant");
    assert_eq!(result.request_intent, Some("realtime".to_string()));
    assert!(!result.no_route);
}

/// explain_selection ровно один победитель
#[test]
fn test_explain_has_exactly_one_winner() {
    let upstreams = vec![
        make_upstream("a", "http://127.0.0.1:8081", &["realtime"]),
        make_upstream("b", "http://127.0.0.1:8082", &["realtime"]),
        make_upstream("c", "http://127.0.0.1:8083", &["realtime"]),
    ];
    let ups = Arc::new(upstreams.clone());
    let align = make_align(ups);
    let arcs = arcs(&upstreams);

    let result = align.explain_selection("r", "resonant", &arcs, None);

    let winners: Vec<_> = result.candidates.iter().filter(|c| c.winner).collect();
    assert_eq!(winners.len(), 1, "ровно один победитель");

    let selected_name = result.selected.as_deref().unwrap_or("");
    assert_eq!(winners[0].name, selected_name,
        "победитель в candidates совпадает с selected");
}

/// explain показывает score в правильном порядке (меньше = первый)
#[test]
fn test_explain_sorted_by_score_ascending() {
    let upstreams = vec![
        make_upstream("slow",  "http://127.0.0.1:8081", &["realtime"]),
        make_upstream("fast",  "http://127.0.0.1:8082", &["realtime"]),
    ];

    for _ in 0..20 {
        upstreams[0].record_request(Duration::from_millis(300), true);
        upstreams[1].record_request(Duration::from_millis(10),  true);
    }

    let ups = Arc::new(upstreams.clone());
    let align = make_align(ups);
    let arcs = arcs(&upstreams);

    let result = align.explain_selection("r", "resonant", &arcs, Some(&Intent::new("realtime")));

    assert_eq!(result.candidates[0].name, "fast",
        "fast должен быть первым (меньший score)");
    assert!(
        result.candidates[0].total_score <= result.candidates[1].total_score,
        "кандидаты должны быть отсортированы по score"
    );
}

/// explain содержит верную разбивку компонентов
#[test]
fn test_explain_score_components_sum_correctly() {
    let upstreams = vec![
        make_upstream("backend", "http://127.0.0.1:8081", &["realtime"]),
    ];
    let ups = Arc::new(upstreams.clone());
    let align = make_align(ups);
    let arcs = arcs(&upstreams);

    let result = align.explain_selection("r", "resonant", &arcs, Some(&Intent::new("realtime")));

    let c = &result.candidates[0];
    let expected_total = c.components.load_component
        + c.components.intent_component
        + c.components.tempo_component;

    assert!(
        (c.total_score - expected_total).abs() < 1e-10,
        "total_score ({}) должен быть суммой компонентов ({})",
        c.total_score, expected_total
    );
}

// ── 5. Combined scoring ───────────────────────────────────────────────────────

/// Политика intent-first: intent важнее нагрузки
///
/// Расчёт score при w_load=0.2, w_intent=0.7:
///   fast (intent=batch, p95=1ms):   0.2×0.01 + 0.7×1.0 = 0.702  (intent_gap=1)
///   slow (intent=realtime, p95=80ms): 0.2×0.8 + 0.7×0.0 = 0.16   (intent_gap=0)
/// → slow побеждает несмотря на большую латентность
#[test]
fn test_intent_first_policy_overrides_load() {
    let upstreams = vec![
        // fast но неподходящий intent
        make_upstream("fast-wrong-intent", "http://127.0.0.1:8081", &["batch"]),
        // умеренно медленный но правильный intent
        make_upstream("slow-right-intent", "http://127.0.0.1:8082", &["realtime"]),
    ];

    // fast-wrong-intent — отличная латентность
    for _ in 0..20 {
        upstreams[0].record_request(Duration::from_millis(1), true);
    }
    // slow-right-intent — умеренная латентность (p95 ~ 80ms → load_res ~ 0.8)
    for _ in 0..20 {
        upstreams[1].record_request(Duration::from_millis(80), true);
    }

    let ups = Arc::new(upstreams.clone());
    let align = make_align(ups);
    let arcs = arcs(&upstreams);

    // С политикой intent-first intent важнее — выбираем slow с правильным intent
    let result = align.explain_selection(
        "r",
        "intent-first",
        &arcs,
        Some(&Intent::new("realtime")),
    );

    assert_eq!(
        result.selected.as_deref(),
        Some("slow-right-intent"),
        "intent-first политика должна выбирать по intent даже при высокой латентности"
    );
}

/// Стандартная политика: нагрузка важнее intent при большой разнице латентности
#[test]
fn test_resonant_policy_balances_load_and_intent() {
    let upstreams = vec![
        make_upstream("fast",  "http://127.0.0.1:8081", &["realtime"]),
        make_upstream("slow",  "http://127.0.0.1:8082", &["realtime"]),
    ];

    // 30x разница в латентности, оба поддерживают realtime
    for _ in 0..20 {
        upstreams[0].record_request(Duration::from_millis(10),  true);
        upstreams[1].record_request(Duration::from_millis(300), true);
    }

    let ups = Arc::new(upstreams.clone());
    let align = make_align(ups);
    let arcs = arcs(&upstreams);

    let result = align.explain_selection("r", "resonant", &arcs, Some(&Intent::new("realtime")));
    assert_eq!(result.selected.as_deref(), Some("fast"));
}

// ── 6. Edge cases ─────────────────────────────────────────────────────────────

/// Один upstream — всегда выбирается
#[test]
fn test_single_upstream_always_selected() {
    let upstreams = vec![
        make_upstream("only-one", "http://127.0.0.1:8081", &["realtime"]),
    ];
    let ups = Arc::new(upstreams.clone());
    let align = make_align(ups);
    let arcs = arcs(&upstreams);

    let selected = align.select_upstream("resonant", &arcs, Some(&Intent::new("batch")));
    assert!(selected.is_some());
    assert_eq!(selected.unwrap().name, "only-one");
}

/// Пустой список upstreams — возвращает None
#[test]
fn test_empty_upstreams_returns_none() {
    let upstreams: Vec<UpstreamState> = vec![];
    let ups = Arc::new(upstreams);
    let align = make_align(ups);

    let selected = align.select_upstream("resonant", &[], None);
    assert!(selected.is_none());
}

/// Неизвестная политика использует defaults
#[test]
fn test_unknown_policy_uses_defaults() {
    let upstreams = vec![
        make_upstream("backend", "http://127.0.0.1:8081", &["realtime"]),
    ];
    let ups = Arc::new(upstreams.clone());
    let align = make_align(ups);
    let arcs = arcs(&upstreams);

    let selected = align.select_upstream("nonexistent-policy", &arcs, None);
    assert!(selected.is_some(), "неизвестная политика должна вернуть результат через defaults");
}
