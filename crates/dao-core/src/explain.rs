//! Explain — объяснение решений маршрутизации
//!
//! Типы данных для команды `daoctl explain`:
//! показывает почему был выбран конкретный upstream.

use serde::{Deserialize, Serialize};
use crate::upstream::CircuitStatus;

/// Входные параметры для объяснения решения
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExplainRequest {
    /// HTTP метод (GET, POST, ...)
    pub method: Option<String>,
    /// Путь запроса (например /v1/users)
    pub path: Option<String>,
    /// Host заголовок
    pub host: Option<String>,
    /// Intent тег (realtime, batch, streaming, ai)
    pub intent: Option<String>,
}

/// Разбивка вклада каждого компонента в итоговый score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreComponents {
    /// w_load × load_resonance
    pub load_component: f64,
    /// w_intent × intent_gap
    pub intent_component: f64,
    /// w_tempo × tempo_spikiness
    pub tempo_component: f64,
}

/// Оценка одного кандидата upstream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateScore {
    /// Имя upstream
    pub name: String,
    /// URL upstream
    pub url: String,
    /// Итоговый score (меньше = лучше)
    pub total_score: f64,
    /// Нагрузка (латентность + ошибки + очередь)
    pub load_resonance: f64,
    /// Несовпадение intent (0 = совпадает, 1 = нет)
    pub intent_gap: f64,
    /// Вариативность RPS
    pub tempo_spikiness: f64,
    /// P95 латентность в мс
    pub p95_latency_ms: f64,
    /// Error rate (0.0–1.0)
    pub error_rate: f64,
    /// Текущий RPS
    pub current_rps: f64,
    /// Разбивка по компонентам
    pub components: ScoreComponents,
    /// Этот upstream был выбран
    pub winner: bool,
    /// Состояние circuit breaker
    pub circuit: CircuitStatus,
    /// Был ли upstream исключён из-за открытого circuit breaker
    pub circuit_open: bool,
}

/// Веса политики (для отображения)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyWeightsInfo {
    pub w_load: f64,
    pub w_intent: f64,
    pub w_tempo: f64,
}

/// Полный результат объяснения решения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainResult {
    /// Выбранный upstream (None если ни один не подошёл)
    pub selected: Option<String>,
    /// Имя маршрута
    pub route: String,
    /// Политика балансировки
    pub policy: String,
    /// Intent запроса
    pub request_intent: Option<String>,
    /// Веса политики
    pub weights: PolicyWeightsInfo,
    /// Все кандидаты с их оценками
    pub candidates: Vec<CandidateScore>,
    /// Маршрут не найден
    pub no_route: bool,
}

/// Метрики одного upstream для /admin/upstreams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamInfo {
    pub name: String,
    pub url: String,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub error_rate: f64,
    pub current_rps: f64,
    pub success_count: u64,
    pub error_count: u64,
    pub load_resonance: f64,
    pub tempo_spikiness: f64,
    pub circuit: CircuitStatus,
}

/// Ответ /admin/upstreams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamsResponse {
    pub upstreams: Vec<UpstreamInfo>,
}

/// Статистика одного upstream для canary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryUpstreamStat {
    pub name: String,
    pub url: String,
    pub weight: u32,
    pub weight_pct: f64,
    pub requests_total: u64,
    pub requests_pct: f64,
    pub error_rate: f64,
    pub p95_latency_ms: f64,
    pub circuit: CircuitStatus,
}

/// Информация о canary для explain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryInfo {
    pub policy: String, // "canary"
    pub upstreams: Vec<CanaryUpstreamStat>,
}

/// Ответ /admin/canary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryResponse {
    pub routes: Vec<CanaryRouteInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryRouteInfo {
    pub route: String,
    pub upstreams: Vec<CanaryUpstreamStat>,
    pub total_requests: u64,
}
