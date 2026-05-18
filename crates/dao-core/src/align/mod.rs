//! Align — выравнивание и принятие решений
//!
//! Модуль политик маршрутизации:
//! - Resonant load balancing
//! - Circuit breaker
//! - Canary routing
//! - A/B testing

use crate::{Intent, upstream::UpstreamState};
use crate::sense::Sense;
use crate::explain::{CandidateScore, CanaryInfo, ExplainResult, PolicyWeightsInfo, ScoreComponents};
use std::sync::Arc;

/// Лёгкая инлайн-версия `Sense::get_resonance_metrics` для одного upstream'а.
/// Используется в горячем пути выбора, чтобы избежать линейного поиска по имени
/// и повторных клонов UpstreamStats (с дорогой Histogram внутри).
fn resonance_from(stats: &crate::upstream::UpstreamStats) -> (f64, f64) {
    let latency = (stats.p95_latency_ms() / 100.0).min(10.0);
    let error = stats.error_rate() * 10.0;
    let queue = stats.queue_depth_norm() * 10.0;
    (latency + error + queue, stats.tempo_spikiness())
}

pub mod policy;
pub mod selector;

pub use policy::{Policy, PolicyWeights};
pub use selector::UpstreamSelector;

/// Align — система принятия решений
#[derive(Clone)]
pub struct Align {
    sense: Sense,
    policies: PolicyRegistry,
}

impl Align {
    pub fn new(sense: Sense) -> Self {
        Self {
            sense,
            policies: PolicyRegistry::new(),
        }
    }

    /// Регистрация политики
    pub fn register_policy(&mut self, name: String, weights: PolicyWeights) {
        self.policies.register(name, weights);
    }

    /// Выбор upstream для запроса
    pub fn select_upstream(
        &self,
        policy_name: &str,
        upstreams: &[Arc<UpstreamState>],
        request_intent: Option<&Intent>,
    ) -> Option<Arc<UpstreamState>> {
        // Canary — отдельная ветка: взвешенный случайный выбор
        if policy_name == "canary" {
            return self.select_canary(upstreams);
        }

        let default_weights = PolicyWeights::default();
        let weights = self.policies.get(policy_name)
            .unwrap_or(&default_weights);

        // Вычисление resonant score для каждого upstream
        // Upstreams с открытым circuit breaker исключаются, но если все
        // открыты — используем всех (лучше попробовать, чем 503).
        // Без аллокации Vec для available: один проход + флаг.
        let any_available = upstreams.iter().any(|u| u.is_available());

        let mut best: Option<(&Arc<UpstreamState>, f64)> = None;
        for upstream in upstreams {
            if any_available && !upstream.is_available() {
                continue;
            }

            let stats = upstream.get_stats();
            let (resonance, tempo_spike) = resonance_from(&stats);
            let intent_gap = request_intent.map(|i| upstream.intent_gap(i)).unwrap_or(0.0);

            let score = weights.w_load * resonance
                + weights.w_intent * intent_gap
                + weights.w_tempo * tempo_spike;

            match best {
                Some((_, best_score)) if !(score < best_score) => {}
                _ => best = Some((upstream, score)),
            }
        }

        best.map(|(u, _)| u.clone())
    }

    /// Выбор upstream через canary (взвешенный случайный)
    pub fn select_canary(&self, upstreams: &[Arc<UpstreamState>]) -> Option<Arc<UpstreamState>> {
        canary_select(upstreams)
    }

    /// Explain для canary policy
    pub fn explain_canary(
        &self,
        route_name: &str,
        upstreams: &[Arc<UpstreamState>],
    ) -> CanaryInfo {
        let total_weight: u32 = upstreams.iter().map(|u| u.weight).sum();
        let stats: Vec<_> = upstreams.iter().map(|u| {
            let s = u.get_stats();
            let total_reqs = s.success_count + s.error_count;
            let weight_pct = if total_weight > 0 {
                u.weight as f64 / total_weight as f64 * 100.0
            } else { 0.0 };
            crate::explain::CanaryUpstreamStat {
                name: u.name.clone(),
                url: u.url.clone(),
                weight: u.weight,
                weight_pct,
                requests_total: total_reqs,
                requests_pct: 0.0, // вычисляется ниже
                error_rate: s.error_rate(),
                p95_latency_ms: s.p95_latency_ms(),
                circuit: u.circuit_status(),
            }
        }).collect();

        let grand_total: u64 = stats.iter().map(|s| s.requests_total).sum();
        let upstreams_with_pct = stats.into_iter().map(|mut s| {
            s.requests_pct = if grand_total > 0 {
                s.requests_total as f64 / grand_total as f64 * 100.0
            } else { 0.0 };
            s
        }).collect();

        CanaryInfo {
            policy: route_name.to_string(),
            upstreams: upstreams_with_pct,
        }
    }

    /// Выбор upstream с полным объяснением решения
    pub fn explain_selection(
        &self,
        route_name: &str,
        policy_name: &str,
        upstreams: &[Arc<UpstreamState>],
        request_intent: Option<&Intent>,
    ) -> ExplainResult {
        let default_weights = PolicyWeights::default();
        let weights = self.policies.get(policy_name).unwrap_or(&default_weights);

        // Все кандидаты — включая с открытым circuit (для отображения в explain).
        // Один проход для определения наличия доступных без аллокации HashSet имён.
        let has_available = upstreams.iter().any(|u| u.is_available());

        let mut candidates: Vec<CandidateScore> = upstreams
            .iter()
            .map(|upstream| {
                let circuit_status = upstream.circuit_status();
                let circuit_open = has_available && !upstream.is_available();

                // Один клон UpstreamStats на upstream вместо двух вызовов
                let stats = upstream.get_stats();
                let (resonance, tempo_spike) = resonance_from(&stats);

                let intent_gap = if let Some(req_intent) = request_intent {
                    upstream.intent_gap(req_intent)
                } else {
                    0.0
                };

                let load_component = weights.w_load * resonance;
                let intent_component = weights.w_intent * intent_gap;
                let tempo_component = weights.w_tempo * tempo_spike;
                // Открытый circuit получает бесконечный score — не может выиграть
                let total_score = if circuit_open {
                    f64::MAX
                } else {
                    load_component + intent_component + tempo_component
                };

                CandidateScore {
                    name: upstream.name.clone(),
                    url: upstream.url.clone(),
                    total_score,
                    load_resonance: resonance,
                    intent_gap,
                    tempo_spikiness: tempo_spike,
                    p95_latency_ms: stats.p95_latency_ms(),
                    error_rate: stats.error_rate(),
                    current_rps: stats.current_rps(),
                    components: ScoreComponents {
                        load_component,
                        intent_component,
                        tempo_component,
                    },
                    winner: false,
                    circuit: circuit_status,
                    circuit_open,
                }
            })
            .collect();

        // Сортировка: open circuit последними, затем по score
        candidates.sort_by(|a, b| {
            match (a.circuit_open, b.circuit_open) {
                (true, false) => std::cmp::Ordering::Greater,
                (false, true) => std::cmp::Ordering::Less,
                _ => a.total_score
                    .partial_cmp(&b.total_score)
                    .unwrap_or(std::cmp::Ordering::Equal),
            }
        });

        // Отметить победителя (первый доступный)
        let selected = candidates.first().map(|c| c.name.clone());
        if let Some(first) = candidates.first_mut() {
            first.winner = true;
        }

        ExplainResult {
            selected,
            route: route_name.to_string(),
            policy: policy_name.to_string(),
            request_intent: request_intent.map(|i| i.0.clone()),
            weights: PolicyWeightsInfo {
                w_load: weights.w_load,
                w_intent: weights.w_intent,
                w_tempo: weights.w_tempo,
            },
            candidates,
            no_route: false,
        }
    }
}

/// Взвешенный случайный выбор upstream (canary policy).
/// Upstreams с открытым circuit исключаются автоматически.
/// Если все circuit открыты — используем всех как fallback.
pub fn canary_select(upstreams: &[Arc<UpstreamState>]) -> Option<Arc<UpstreamState>> {
    use rand::Rng;

    if upstreams.is_empty() {
        return None;
    }

    let available: Vec<_> = upstreams.iter().filter(|u| u.is_available()).collect();
    let pool: Vec<_> = if available.is_empty() {
        upstreams.iter().collect()
    } else {
        available
    };

    let total_weight: u32 = pool.iter().map(|u| u.weight).sum();
    if total_weight == 0 {
        return pool.first().map(|u| (*u).clone());
    }

    let mut pick = rand::thread_rng().gen_range(0..total_weight);
    for upstream in &pool {
        if pick < upstream.weight {
            return Some((*upstream).clone());
        }
        pick -= upstream.weight;
    }
    pool.last().map(|u| (*u).clone())
}

/// Реестр политик
#[derive(Clone)]
struct PolicyRegistry {
    policies: std::collections::HashMap<String, PolicyWeights>,
}

impl PolicyRegistry {
    fn new() -> Self {
        let mut policies = std::collections::HashMap::new();
        // Дефолтная политика
        policies.insert("resonant".to_string(), PolicyWeights::default());
        Self { policies }
    }

    fn register(&mut self, name: String, weights: PolicyWeights) {
        self.policies.insert(name, weights);
    }

    fn get(&self, name: &str) -> Option<&PolicyWeights> {
        self.policies.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_align_selection() {
        let upstreams = vec![
            UpstreamState::new(
                "upstream1".to_string(),
                "http://localhost:8081".to_string(),
                vec![Intent::new("realtime")],
                1,
            ),
            UpstreamState::new(
                "upstream2".to_string(),
                "http://localhost:8082".to_string(),
                vec![Intent::new("batch")],
                1,
            ),
        ];

        // Симулируем нагрузку на первый upstream
        upstreams[0].record_request(Duration::from_millis(100), true);
        upstreams[1].record_request(Duration::from_millis(10), true);

        let sense = Sense::new(Arc::new(upstreams.clone()));
        let align = Align::new(sense);

        let upstreams_arc: Vec<_> = upstreams.into_iter().map(Arc::new).collect();
        let selected = align.select_upstream(
            "resonant",
            &upstreams_arc,
            Some(&Intent::new("realtime")),
        );

        assert!(selected.is_some());
    }
}
