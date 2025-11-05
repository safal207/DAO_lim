//! Policy definitions

/// Политика маршрутизации
#[derive(Debug, Clone)]
pub enum Policy {
    /// Resonant load balancing с весами
    Resonant(PolicyWeights),
    /// Round-robin
    RoundRobin,
    /// Random
    Random,
    /// Least connections
    LeastConnections,
}

/// Веса для resonant политики
#[derive(Debug, Clone)]
pub struct PolicyWeights {
    /// Вес load_resonance (латентность + ошибки + очередь)
    pub w_load: f64,
    /// Вес intent gap (несовпадение намерений)
    pub w_intent: f64,
    /// Вес tempo spikiness (вариативность RPS)
    pub w_tempo: f64,
}

impl Default for PolicyWeights {
    fn default() -> Self {
        Self {
            w_load: 0.6,
            w_intent: 0.3,
            w_tempo: 0.1,
        }
    }
}

impl PolicyWeights {
    pub fn new(w_load: f64, w_intent: f64, w_tempo: f64) -> Self {
        Self {
            w_load,
            w_intent,
            w_tempo,
        }
    }

    /// Валидация весов (должны быть положительными)
    pub fn validate(&self) -> bool {
        self.w_load >= 0.0 && self.w_intent >= 0.0 && self.w_tempo >= 0.0
    }
}
