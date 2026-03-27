//! Align — выравнивание и принятие решений
//!
//! Модуль политик маршрутизации:
//! - Resonant load balancing
//! - Circuit breaker
//! - Canary routing
//! - A/B testing

use crate::{Intent, Result, upstream::UpstreamState};
use crate::sense::Sense;
use crate::explain::{CandidateScore, ExplainResult, PolicyWeightsInfo, ScoreComponents};
use std::sync::Arc;

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
        let default_weights = PolicyWeights::default();
        let weights = self.policies.get(policy_name)
            .unwrap_or(&default_weights);

        let metrics = self.sense.get_resonance_metrics();

        // Вычисление resonant score для каждого upstream
        let mut scored_upstreams: Vec<_> = upstreams
            .iter()
            .map(|upstream| {
                let resonance = metrics
                    .iter()
                    .find(|m| m.upstream_name == upstream.name)
                    .map(|m| m.load_resonance)
                    .unwrap_or(0.0);

                let intent_gap = if let Some(req_intent) = request_intent {
                    upstream.intent_gap(req_intent)
                } else {
                    0.0
                };

                let tempo_spike = metrics
                    .iter()
                    .find(|m| m.upstream_name == upstream.name)
                    .map(|m| m.tempo_spikiness)
                    .unwrap_or(0.0);

                let score = weights.w_load * resonance
                    + weights.w_intent * intent_gap
                    + weights.w_tempo * tempo_spike;

                (upstream.clone(), score)
            })
            .collect();

        // Сортировка по score (меньше = лучше)
        scored_upstreams.sort_by(|a, b| {
            a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Возврат лучшего upstream
        scored_upstreams.first().map(|(u, _)| u.clone())
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

        let metrics = self.sense.get_resonance_metrics();

        let mut candidates: Vec<CandidateScore> = upstreams
            .iter()
            .map(|upstream| {
                let resonance = metrics
                    .iter()
                    .find(|m| m.upstream_name == upstream.name)
                    .map(|m| m.load_resonance)
                    .unwrap_or(0.0);

                let intent_gap = if let Some(req_intent) = request_intent {
                    upstream.intent_gap(req_intent)
                } else {
                    0.0
                };

                let tempo_spike = metrics
                    .iter()
                    .find(|m| m.upstream_name == upstream.name)
                    .map(|m| m.tempo_spikiness)
                    .unwrap_or(0.0);

                let load_component = weights.w_load * resonance;
                let intent_component = weights.w_intent * intent_gap;
                let tempo_component = weights.w_tempo * tempo_spike;
                let total_score = load_component + intent_component + tempo_component;

                let stats = upstream.get_stats();

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
                }
            })
            .collect();

        // Сортировка по score (меньше = лучше)
        candidates.sort_by(|a, b| {
            a.total_score
                .partial_cmp(&b.total_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Отметить победителя
        let selected = candidates.first().map(|c| {
            c.name.clone()
        });
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
