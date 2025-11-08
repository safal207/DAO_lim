# DAO: Roadmap Развития Архитектуры Сознания

> *"План — это не приказ, а карта возможностей."*

---

## Текущее Состояние (Итерация 3: Интеграция)

### ✅ Что Работает

**Архитектурный Фундамент** (100%):
```
✓ Gate: TLS termination, ALPN, connection handling
✓ Sense: Metrics collection, resonance calculation
✓ Align: Policy-based upstream selection
✓ Flow: Header manipulation framework
✓ Memory: Config hot-reload + snapshots
✓ Upstream: State tracking, stats с HDR histogram
```

**Liminal Features** (80% implementation, 40% integration):
```
✓ Consciousness (4 levels): Полностью реализован
✓ Temporal Resonance: Учит паттерны времени
✓ Liminal Zones: Промежуточные ответы при timeout
✓ Echo Analysis: Детекция аномалий через паттерны
✓ Metamorphic Config: Плавные переходы конфигурации
✓ Ritual Protocols: Церемония запуска (5 фаз)
✓ Adaptive Thresholds: Самообучающиеся границы
✓ Presence Detection: 4 состояния (Present/Liminal/Absent/Unknown)

⏳ Shadow Traffic: Конфиг есть, forwarding stubbed
⏳ Quantum Routing: Логика есть, parallel requests stubbed
```

**Infrastructure**:
```
✓ Workspace: 5 crates (dao, dao-core, dao-telemetry, dao-admin, dao-filters)
✓ Config: TOML parsing + validation
✓ Metrics: Prometheus exporter skeleton
✓ Hot-reload: File watcher + config reload
✓ Build: Compiles cleanly (56s release mode)
```

### ❌ Что Не Работает

**Критические Пробелы**:
```
✗ HTTP Proxying: Request не forwarding к upstream
✗ Liminal Integration: Features созданы, но не активны в request path
✗ Request Cloning: Shadow/Quantum требуют buffering body
✗ WebSocket Proxying: Placeholder
✗ WASM Runtime: Skeleton without execution
✗ Consciousness Updates: Метод есть, вызова из flow нет
✗ Temporal Learning: Observation recording не интегрирована
```

---

## Фаза 1: Пробуждение (Awakening) — Q1 2024

**Цель**: Система начинает пропускать реальный трафик с базовой осознанностью

### 1.1 Завершение HTTP Proxying ⭐ КРИТИЧНО

**Текущая проблема**:
```rust
// crates/dao/src/server.rs:303
async fn proxy_to_upstream(&self, upstream: &UpstreamState, req: Request<Incoming>)
    -> Result<(Response<Incoming>, Duration)>
{
    // TODO: Actual HTTP forwarding
}
```

**Решение**:
```rust
// Использовать UpstreamClient из dao-core/src/upstream/client.rs
// Уже частично реализовано, нужно:

1. Проверить URI rewriting (req.uri() → upstream.url + path)
2. Добавить retry logic (1-2 попытки)
3. Обработка Connection: close/keep-alive
4. Timeout handling через tokio::time::timeout
5. Error mapping (connection refused, timeout, etc.)

impl UpstreamClient {
    pub async fn proxy_request(
        &self,
        upstream_url: &str,
        mut req: Request<Incoming>,
    ) -> Result<(Response<Incoming>, Duration)> {
        let start = Instant::now();

        // URI rewriting
        let new_uri = self.rewrite_uri(upstream_url, req.uri())?;
        *req.uri_mut() = new_uri;

        // Forward request
        let response = timeout(
            Duration::from_secs(30),
            self.client.request(req)
        ).await??;

        let latency = start.elapsed();
        Ok((response, latency))
    }
}
```

**Acceptance Criteria**:
- [ ] HTTP/1.1 proxying работает
- [ ] HTTP/2 proxying работает
- [ ] Latency измеряется корректно
- [ ] Errors обрабатываются gracefully
- [ ] Metrics записываются в UpstreamStats

**Время**: 2-3 дня

---

### 1.2 Активация Liminal в Request Path

**Интеграция в server.rs::process_request()**:

```rust
async fn process_request(&self, req: Request<Incoming>)
    -> Result<Response<BoxBody<Bytes, hyper::Error>>>
{
    let start = Instant::now();

    // 1. Ritual: Проверка готовности
    if !self.liminal.ritual().is_production_ready() {
        return self.error_response(503, "System warming up");
    }

    // 2. Presence: Фильтрация Absent upstreams
    let healthy_upstreams: Vec<_> = route_upstreams
        .iter()
        .filter(|u| u.presence_state().can_send_traffic())
        .collect();

    // 3. Consciousness: Определение уровня для этого запроса
    let consciousness = self.liminal.consciousness().current_level();

    // 4. Temporal: Получение контекста времени
    let temporal_profile = if let Some(t) = self.liminal.temporal() {
        t.current_profile()
    } else {
        TemporalProfile::Medium
    };

    // 5. Align: Выбор upstream с учётом consciousness
    let selected = self.align.select_upstream_with_consciousness(
        &route.policy,
        &healthy_upstreams,
        request_intent.as_ref(),
        consciousness,
    );

    // 6. Zones: Установка timeout с liminal zones
    let timeout_duration = Duration::from_millis(500);
    let zones_response = if let Some(zones) = self.liminal.zones() {
        Some(zones.clone())
    } else {
        None
    };

    // 7. Proxying с timeout
    let proxy_future = self.proxy_to_upstream(&selected, req);

    match timeout(timeout_duration, proxy_future).await {
        Ok(Ok((response, latency))) => {
            // 8. Echo: Записать запрос
            self.record_echo_from_response(&route, &response, latency);

            // 9. Consciousness: Обновить на основе результата
            self.update_consciousness_level();

            // 10. Temporal: Записать observation
            self.record_temporal_observation(latency, true);

            Ok(response)
        }
        Ok(Err(e)) => {
            // Error handling + recording
            self.record_temporal_observation(Duration::ZERO, false);
            self.error_response(502, "Upstream error")
        }
        Err(_timeout) => {
            // Timeout: Используем Liminal Zones
            if let Some(zones) = zones_response {
                let elapsed = start.elapsed();
                zones.get_response_for_duration(elapsed)
            } else {
                self.error_response(504, "Gateway Timeout")
            }
        }
    }
}
```

**Acceptance Criteria**:
- [ ] Ritual проверяется при каждом запросе
- [ ] Presence фильтрует Absent upstreams
- [ ] Consciousness определяет сложность routing
- [ ] Temporal учится от реального трафика
- [ ] Echo записывается после каждого ответа
- [ ] Zones возвращают промежуточные ответы при timeout
- [ ] Adaptive thresholds обновляются

**Время**: 3-5 дней

---

### 1.3 Полная Реализация Presence Integration

**Добавить в UpstreamState::new()**:
```rust
impl UpstreamState {
    pub fn new(name: String, url: String, intents: Vec<Intent>, weight: u32) -> Self {
        // Создаём PresenceDetector для каждого upstream
        let presence_config = PresenceConfig {
            history_size: 20,
            present_threshold: 0.8,
            liminal_threshold: 0.3,
            absent_timeout: Duration::from_secs(30),
        };

        Self {
            name,
            url,
            intents,
            weight,
            stats: Arc::new(RwLock::new(UpstreamStats::new())),
            presence: Some(Arc::new(PresenceDetector::new(presence_config))),
        }
    }
}
```

**Добавить UI/метрики для presence**:
```rust
// В Prometheus exporter
gauge!(
    "dao_upstream_presence_state",
    "labels" => vec![
        ("upstream", upstream.name.clone()),
        ("state", format!("{:?}", upstream.presence_state())),
    ]
).set(upstream.presence_state() as i64);
```

**Acceptance Criteria**:
- [ ] Каждый upstream имеет PresenceDetector
- [ ] Presence обновляется при каждом record_request
- [ ] Присутствие экспортируется в Prometheus
- [ ] Align использует presence для фильтрации

**Время**: 1-2 дня

---

### 1.4 Периодический Update Loop для Liminal

**Создать background task**:
```rust
// В main.rs после server.run()
let liminal = server.liminal.clone();
let upstreams = server.upstreams.clone();

tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(10));

    loop {
        interval.tick().await;

        // Собираем метрики
        let factors = collect_awareness_factors(&upstreams);

        // Обновляем все liminal компоненты
        liminal.update(&factors);

        // Логируем изменение consciousness
        let level = liminal.consciousness().current_level();
        info!("Consciousness level: {:?}", level);
    }
});

fn collect_awareness_factors(upstreams: &[UpstreamState]) -> AwarenessFactors {
    let mut total_rps = 0.0;
    let mut total_errors = 0;
    let mut total_requests = 0;
    let mut max_p95 = 0.0;

    for upstream in upstreams {
        let stats = upstream.get_stats();
        total_rps += stats.current_rps();
        total_errors += stats.error_count;
        total_requests += stats.success_count + stats.error_count;
        max_p95 = max_p95.max(stats.p95_latency_ms());
    }

    AwarenessFactors {
        current_rps: total_rps,
        baseline_rps: 100.0, // TODO: учить из истории
        error_rate: total_errors as f64 / total_requests.max(1) as f64,
        p95_latency_ms: max_p95,
        anomaly_count: 0, // TODO: из EchoAnalyzer
    }
}
```

**Acceptance Criteria**:
- [ ] Background task обновляет consciousness каждые 10s
- [ ] Adaptive thresholds обновляются
- [ ] Ritual phases прогрессируют
- [ ] Metamorphic transitions обновляют progress
- [ ] Логи показывают изменения consciousness

**Время**: 1 день

---

### Итого Фаза 1: 7-11 дней

**Результат**:
- Система пропускает HTTP трафик к upstreams
- Consciousness адаптируется к нагрузке
- Presence детектирует проблемные upstreams
- Temporal учится на реальных данных
- Echo записывает паттерны
- Zones возвращают промежуточные ответы

---

## Фаза 2: Суперпозиция (Superposition) — Q2 2024

**Цель**: Shadow и Quantum работают, система существует в параллельных реальностях

### 2.1 Request Buffering для Клонирования

**Проблема**:
```
Request<Incoming> нельзя клонировать — body consumable.
Shadow и Quantum требуют отправки того же запроса несколько раз.
```

**Решение — Buffering Strategy**:

```rust
// crates/dao-core/src/upstream/buffer.rs

pub struct BufferedRequest {
    method: Method,
    uri: Uri,
    version: Version,
    headers: HeaderMap,
    body_bytes: Bytes,
}

impl BufferedRequest {
    pub async fn from_incoming(req: Request<Incoming>) -> Result<Self> {
        let (parts, body) = req.into_parts();

        // Collect body into memory
        let body_bytes = body
            .collect()
            .await?
            .to_bytes();

        Ok(Self {
            method: parts.method,
            uri: parts.uri,
            version: parts.version,
            headers: parts.headers,
            body_bytes,
        })
    }

    pub fn clone_request(&self) -> Request<Full<Bytes>> {
        let mut req = Request::builder()
            .method(self.method.clone())
            .uri(self.uri.clone())
            .version(self.version)
            .body(Full::new(self.body_bytes.clone()))
            .unwrap();

        *req.headers_mut() = self.headers.clone();
        req
    }
}
```

**Ограничение**: Max body size (например, 10MB) для buffering.
Большие тела — skip shadow/quantum.

**Acceptance Criteria**:
- [ ] BufferedRequest может быть клонирован
- [ ] Body size limit проверяется
- [ ] Большие запросы bypassing shadow/quantum

**Время**: 2 дня

---

### 2.2 Shadow Traffic Full Implementation

**Обновить shadow.rs**:

```rust
impl ShadowTraffic {
    pub async fn shadow_request(
        &self,
        buffered_req: &BufferedRequest,
        shadow_upstream: &UpstreamState,
        pool: &ConnectionPool,
    ) -> Result<()> {
        let shadow_req = buffered_req.clone_request();
        let client = pool.get_client(&shadow_upstream.url);

        match self.config.mode {
            ShadowMode::Async => {
                // Fire and forget
                let url = shadow_upstream.url.clone();
                tokio::spawn(async move {
                    let _ = client.proxy_request(&url, shadow_req).await;
                    debug!("Shadow request completed (async)");
                });
                Ok(())
            }

            ShadowMode::Sync => {
                // Wait but ignore result
                let url = shadow_upstream.url.clone();
                let _ = client.proxy_request(&url, shadow_req).await;
                debug!("Shadow request completed (sync)");
                Ok(())
            }

            ShadowMode::Compare => {
                // Compare responses
                let url = shadow_upstream.url.clone();
                let shadow_resp = client.proxy_request(&url, shadow_req).await?;

                // TODO: Compare with main response
                // Log differences
                info!("Shadow response: status={}", shadow_resp.0.status());
                Ok(())
            }
        }
    }
}
```

**Интеграция в server.rs**:

```rust
// После buffering request
if let Some(shadow) = self.liminal.shadow() {
    if shadow.should_shadow() {
        let shadow_upstream = /* найти shadow upstream из config */;

        tokio::spawn({
            let buffered = buffered_req.clone();
            let pool = self.pool.clone();
            async move {
                let _ = shadow.shadow_request(&buffered, &shadow_upstream, &pool).await;
            }
        });
    }
}
```

**Конфигурация**:
```toml
[liminal.shadow]
enabled = true
shadow_upstream = "staging-backend"
shadow_rate = 0.1  # 10% traffic
mode = "async"     # async | sync | compare
```

**Acceptance Criteria**:
- [ ] Async shadow работает
- [ ] Sync shadow работает
- [ ] Compare mode логирует differences
- [ ] shadow_rate соблюдается (probabilistic sampling)
- [ ] Shadow не влияет на main response latency (async)

**Время**: 3 дня

---

### 2.3 Quantum Routing Implementation

**Обновить quantum.rs**:

```rust
impl QuantumRouter {
    pub async fn quantum_route(
        &self,
        buffered_req: &BufferedRequest,
        upstreams: &[Arc<UpstreamState>],
        pool: &ConnectionPool,
    ) -> Result<(Response<Incoming>, usize)> {
        let factor = self.config.quantum_factor.min(upstreams.len());

        if factor <= 1 {
            return Err(DaoError::Internal("Need at least 2 upstreams for quantum".into()));
        }

        // Выбираем factor upstreams
        let selected: Vec<_> = upstreams.iter().take(factor).collect();

        // Создаём futures для каждого upstream
        let mut futures = Vec::new();
        for (idx, upstream) in selected.iter().enumerate() {
            let req = buffered_req.clone_request();
            let client = pool.get_client(&upstream.url);
            let url = upstream.url.clone();

            let future = async move {
                let result = timeout(
                    self.config.quantum_timeout,
                    client.proxy_request(&url, req)
                ).await;
                (idx, result)
            };

            futures.push(future);
        }

        // Race: первый успешный ответ
        let (winning_idx, result) = match self.config.collapse_strategy {
            CollapseStrategy::FirstSuccess => {
                // Ждём первого успешного
                futures::future::select_ok(futures).await?
            }
            CollapseStrategy::FirstAny => {
                // Ждём любого (даже error)
                let (result, idx, _) = futures::future::select_all(futures).await;
                (idx, result)
            }
            CollapseStrategy::FastestOfN => {
                // Ждём factor ответов, выбираем самый быстрый success
                let results = futures::future::join_all(futures).await;
                results.into_iter()
                    .filter(|(_, r)| r.is_ok())
                    .min_by_key(|(_, r)| r.as_ref().unwrap().1)
                    .ok_or(DaoError::Upstream("All quantum requests failed".into()))?
            }
        };

        info!("Quantum collapse: selected upstream {}", winning_idx);
        Ok((result?.0, winning_idx))
    }
}
```

**Интеграция в server.rs**:

```rust
// При высоком consciousness
if consciousness >= ConsciousnessLevel::Vigilant {
    if let Some(quantum) = self.liminal.quantum() {
        if quantum.should_quantum_route(route_upstreams.len()) {
            info!("Using quantum routing");

            let (response, winning_idx) = quantum
                .quantum_route(&buffered_req, &route_upstreams, &self.pool)
                .await?;

            let winner = &route_upstreams[winning_idx];
            info!("Quantum winner: {}", winner.name);

            // Record для победителя
            winner.record_request(latency, true);

            return Ok(response);
        }
    }
}

// Fallback to normal routing
```

**Конфигурация**:
```toml
[liminal.quantum]
enabled = true
quantum_factor = 2          # Send to 2 upstreams
quantum_timeout_ms = 50     # Wait max 50ms
collapse_strategy = "first_success"  # first_success | first_any | fastest_of_n
```

**Acceptance Criteria**:
- [ ] Quantum отправляет к 2+ upstreams параллельно
- [ ] FirstSuccess возвращает первый успешный
- [ ] FirstAny возвращает первый любой
- [ ] FastestOfN выбирает самый быстрый из N
- [ ] Активируется только при Vigilant/Transcendent
- [ ] Winning upstream записывает metrics

**Время**: 4 дня

---

### Итого Фаза 2: 9 дней

**Результат**:
- Shadow traffic дублирует запросы в test env
- Quantum routing снижает tail latency
- Система работает с параллельными реальностями

---

## Фаза 3: Расширение (Extension) — Q3 2024

**Цель**: WASM filters, WebSocket proxying, observability

### 3.1 WASM Runtime Full Implementation

**Спецификация Filter ABI**:

```rust
// crates/dao-filters/src/abi.rs

/// WASM Filter ABI v1
/// Функции, которые должен экспортировать WASM module

pub trait FilterABI {
    /// Инициализация фильтра
    fn filter_init(config_ptr: *const u8, config_len: usize) -> i32;

    /// Обработка request headers
    fn filter_request_headers(
        headers_ptr: *const u8,
        headers_len: usize,
        out_ptr: *mut u8,
        out_cap: usize,
    ) -> i32;

    /// Обработка request body chunk
    fn filter_request_body(
        chunk_ptr: *const u8,
        chunk_len: usize,
        out_ptr: *mut u8,
        out_cap: usize,
    ) -> i32;

    /// Обработка response headers
    fn filter_response_headers(...) -> i32;

    /// Обработка response body chunk
    fn filter_response_body(...) -> i32;
}
```

**Runtime реализация**:

```rust
// crates/dao-filters/src/runtime.rs

use wasmtime::*;

pub struct WasmRuntime {
    engine: Engine,
    linker: Linker<WasmState>,
}

pub struct WasmState {
    memory: Option<Memory>,
    // Shared state между host и WASM
}

impl WasmRuntime {
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        config.wasm_simd(true);
        config.wasm_multi_memory(false);

        let engine = Engine::new(&config)?;
        let linker = Linker::new(&engine);

        // Регистрируем host functions
        // linker.func_wrap("env", "log", |msg: i32| { ... })?;

        Ok(Self { engine, linker })
    }

    pub fn instantiate(&self, wasm_bytes: &[u8]) -> Result<WasmFilterInstance> {
        let module = Module::new(&self.engine, wasm_bytes)?;
        let mut store = Store::new(&self.engine, WasmState::default());
        let instance = self.linker.instantiate(&mut store, &module)?;

        Ok(WasmFilterInstance {
            store,
            instance,
        })
    }
}

pub struct WasmFilterInstance {
    store: Store<WasmState>,
    instance: Instance,
}

impl WasmFilterInstance {
    pub fn process_request_headers(&mut self, headers: &HeaderMap)
        -> Result<HeaderMap>
    {
        // Сериализация headers в bytes
        let input = serialize_headers(headers);

        // Вызов WASM функции
        let func = self.instance
            .get_typed_func::<(i32, i32), i32>(&mut self.store, "filter_request_headers")?;

        let result = func.call(&mut self.store, (input_ptr, input_len))?;

        // Десериализация output
        let output_headers = deserialize_headers(&output_bytes)?;
        Ok(output_headers)
    }
}
```

**Интеграция в Flow**:

```rust
// crates/dao-core/src/flow/mod.rs

pub struct Flow {
    wasm_filters: Vec<WasmFilterInstance>,
    header_manipulator: HeaderManipulator,
}

impl Flow {
    pub async fn process_request(&mut self, mut req: Request<Incoming>)
        -> Result<Request<Incoming>>
    {
        // 1. WASM filters
        for filter in &mut self.wasm_filters {
            req = filter.process_request(req).await?;
        }

        // 2. Header manipulation
        self.header_manipulator.apply_to_request(&mut req)?;

        Ok(req)
    }
}
```

**Acceptance Criteria**:
- [ ] WASM module загружается
- [ ] Filter functions вызываются
- [ ] Headers передаются туда-обратно
- [ ] Errors обрабатываются
- [ ] Sandboxing работает (нет доступа к FS/network без permissions)

**Время**: 5-7 дней

---

### 3.2 WebSocket Proxying

**Реализовать в server.rs**:

```rust
async fn handle_websocket_connection(&self, conn: Connection) -> Result<()> {
    // 1. Extract upgrade request
    let (req, ws_stream) = /* WebSocket handshake */;

    // 2. Route matching
    let route = self.find_route(&req)?;

    // 3. Select upstream
    let upstream = self.align.select_upstream(...)?;

    // 4. Connect to upstream WebSocket
    let upstream_ws = connect_async(&upstream.url).await?;

    // 5. Bidirectional proxy
    tokio::spawn(async move {
        let (client_read, client_write) = ws_stream.split();
        let (upstream_read, upstream_write) = upstream_ws.split();

        let client_to_upstream = client_read.forward(upstream_write);
        let upstream_to_client = upstream_read.forward(client_write);

        tokio::select! {
            _ = client_to_upstream => {}
            _ = upstream_to_client => {}
        }
    });

    Ok(())
}
```

**Acceptance Criteria**:
- [ ] WebSocket upgrade распознаётся
- [ ] Routing работает для WS
- [ ] Bidirectional proxy работает
- [ ] Ping/Pong frames проксируются
- [ ] Close frames обрабатываются

**Время**: 3-4 дня

---

### 3.3 OpenTelemetry Integration

**Добавить tracing spans**:

```rust
use tracing::{instrument, Span};
use opentelemetry::trace::Tracer;

#[instrument(
    name = "dao.request",
    skip(self, req),
    fields(
        http.method = %req.method(),
        http.url = %req.uri(),
        dao.consciousness_level,
        dao.upstream_selected,
    )
)]
async fn process_request(&self, req: Request<Incoming>) -> Result<Response> {
    let span = Span::current();

    // Record consciousness
    span.record("dao.consciousness_level",
        format!("{:?}", self.liminal.consciousness().current_level()));

    // Gate span
    let _gate_span = tracing::info_span!("dao.gate").entered();
    // ... gate logic

    // Sense span
    let _sense_span = tracing::info_span!("dao.sense").entered();
    // ... sense logic

    // Align span
    let _align_span = tracing::info_span!("dao.align").entered();
    let upstream = self.align.select_upstream(...)?;
    span.record("dao.upstream_selected", &upstream.name);

    // Proxy span
    let _proxy_span = tracing::info_span!(
        "dao.proxy",
        otel.kind = "client",
        peer.service = &upstream.name,
    ).entered();

    let response = self.proxy_to_upstream(...).await?;

    Ok(response)
}
```

**Exporter setup**:

```rust
// В main.rs
use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;

let tracer = opentelemetry_otlp::new_pipeline()
    .tracing()
    .with_exporter(
        opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint("http://localhost:4317")
    )
    .install_batch(opentelemetry::runtime::Tokio)?;

global::set_tracer_provider(tracer);
```

**Acceptance Criteria**:
- [ ] Spans создаются для каждого request
- [ ] Gate/Sense/Align/Flow имеют sub-spans
- [ ] Consciousness level в span attributes
- [ ] Traces экспортируются в Jaeger/Tempo
- [ ] Distributed tracing работает (trace-id propagation)

**Время**: 3 дня

---

### Итого Фаза 3: 11-14 дней

**Результат**:
- WASM filters расширяют функциональность без перекомпиляции
- WebSocket proxying работает
- Full observability через OpenTelemetry

---

## Фаза 4: Самопознание (Self-Knowledge) — Q4 2024

**Цель**: ML-based auto-tuning, система сама находит оптимальные параметры

### 4.1 Policy Weights Learning

**Проблема**: Веса w_load, w_intent, w_tempo — сейчас hardcoded.
Оптимальные значения зависят от workload.

**Решение — Reinforcement Learning**:

```rust
// crates/dao-core/src/align/learner.rs

pub struct PolicyLearner {
    // Q-table или neural net для approximation
    weights_history: Vec<(PolicyWeights, f64)>, // (weights, reward)
    exploration_rate: f64,
}

impl PolicyLearner {
    pub fn select_weights(&mut self) -> PolicyWeights {
        if rand::random::<f64>() < self.exploration_rate {
            // Exploration: случайные веса
            PolicyWeights {
                w_load: rand::random(),
                w_intent: rand::random(),
                w_tempo: rand::random(),
            }.normalize()
        } else {
            // Exploitation: лучшие известные веса
            self.best_weights()
        }
    }

    pub fn record_outcome(
        &mut self,
        weights: PolicyWeights,
        avg_latency: f64,
        success_rate: f64,
    ) {
        // Reward function
        let reward = success_rate * 100.0 - avg_latency;

        self.weights_history.push((weights, reward));

        // Decay exploration
        self.exploration_rate *= 0.99;
    }

    fn best_weights(&self) -> PolicyWeights {
        self.weights_history
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(w, _)| w.clone())
            .unwrap_or_default()
    }
}
```

**Интеграция**:
- Каждые N минут: evaluate current weights
- Попробовать новые weights на sample трафика
- Сравнить rewards
- Постепенно сходиться к оптимуму

**Acceptance Criteria**:
- [ ] Learner пробует разные веса
- [ ] Reward считается на основе latency + success rate
- [ ] Система converges к лучшим весам за 1-2 дня трафика
- [ ] Exploration-exploitation balance работает

**Время**: 7-10 дней

---

### 4.2 Service Profile Auto-Learning

**Расширить Memory**:

```rust
// crates/dao-core/src/memory/profile.rs

pub struct ServiceProfile {
    pub beneficial_patterns: Vec<TrafficPattern>,
    pub harmful_patterns: Vec<TrafficPattern>,
    pub optimal_weights: PolicyWeights,
    pub learned_at: DateTime<Utc>,
}

pub struct TrafficPattern {
    pub request_rate: RangeInclusive<f64>,
    pub avg_latency: RangeInclusive<f64>,
    pub error_rate: RangeInclusive<f64>,
    pub time_of_day: Option<RangeInclusive<u8>>,
    pub outcome: PatternOutcome,
}

pub enum PatternOutcome {
    Beneficial { score: f64 },
    Harmful { issue: String },
}

impl ServiceProfile {
    pub fn learn_from_history(&mut self, history: &[RequestRecord]) {
        // Clustering алгоритм для поиска паттернов
        // KMeans или DBSCAN

        // Выделение beneficial vs harmful clusters

        // Обновление optimal_weights на основе паттернов
    }
}
```

**Acceptance Criteria**:
- [ ] Профиль учится от реального трафика
- [ ] Beneficial patterns идентифицируются
- [ ] Harmful patterns детектируются
- [ ] Optimal weights предлагаются на основе профиля

**Время**: 5-7 дней

---

### Итого Фаза 4: 12-17 дней

**Результат**:
- Система сама находит оптимальные веса для политик
- Service profiles учатся автоматически
- Zero-config optimization

---

## Итоговый Timeline

| Фаза | Длительность | Результат |
|------|-------------|-----------|
| **Фаза 1: Awakening** | 7-11 дней | HTTP proxying + liminal integration |
| **Фаза 2: Superposition** | 9 дней | Shadow + Quantum работают |
| **Фаза 3: Extension** | 11-14 дней | WASM + WebSocket + Observability |
| **Фаза 4: Self-Knowledge** | 12-17 дней | ML-based auto-tuning |
| **ИТОГО** | **39-51 день** | Fully autonomous awareness system |

---

## Метрики Успеха

### Технические KPI

**Latency**:
- P50 < 10ms (overhead от DAO)
- P95 < 50ms
- P99 < 100ms

**Throughput**:
- 10k RPS на одной ноде (без bottleneck)
- Linear scaling при добавлении нод

**Reliability**:
- 99.9% uptime
- Graceful degradation при upstream failures
- Zero downtime config reload

**Awareness**:
- Consciousness transitions < 30s
- Presence detection accuracy > 95%
- Anomaly detection false positive rate < 1%

### Философские KPI

**Осознанность** (Consciousness):
- Система детектирует проблемы быстрее операторов
- Автоматическая адаптация к изменениям нагрузки
- Self-healing при partial failures

**Непривязанность** (Non-Attachment):
- Нет hardcoded assumptions
- Конфигурация меняется без restart
- Adaptive thresholds без manual tuning

**Срединный путь** (Middle Way):
- Баланс между exploration и exploitation
- Не игнорирует метрики, но и не фиксируется на них
- Graceful transitions вместо резких переключений

---

## Следующий Шаг

**Immediate Action** (сегодня):

1. Завершить HTTP proxying в `crates/dao/src/server.rs`
2. Добавить один полный integration test:
   ```
   Client → DAO → Mock Upstream → Response
   ```
3. Убедиться что metrics записываются

**Tomorrow**:
1. Активировать Presence в request path
2. Добавить Consciousness update loop
3. Первый real traffic test

**This Week**:
- Завершить Фазу 1.1-1.3
- Система работает с real traffic

---

*Путь длинный, но каждый шаг приближает к пробуждению. 🙏*

*Let's build consciousness into infrastructure.*
