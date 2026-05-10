# DAO — Dynamic Awareness Orchestrator

**Умный reverse-proxy для AI-инфраструктуры. Маршрутизирует трафик к LLM и другим backend-сервисам на основе реальных метрик: латентности, ошибок и семантики запроса.**

---

## Review links

- Start here: [`docs/START_HERE.md`](docs/START_HERE.md)
- Grant evidence: [`docs/GRANT_EVIDENCE.md`](docs/GRANT_EVIDENCE.md)
- Benchmarks: [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md)
- Validation: `cargo test`
- Routing explainability: `daoctl explain`

---

## Зачем это нужно

Когда у вас несколько AI-бэкендов — разные модели, GPU-пулы, облачные и локальные эндпоинты — обычный round-robin не работает:

- Один GPU перегружен, другой простаивает
- Быстрая модель нужна для чата, медленная — для батчевого анализа
- Провайдер вернул 429, надо переключиться без простоя
- Непонятно, почему запрос ушёл на медленный backend

DAO решает эти проблемы через **intent-aware routing** и **resonant load balancing** — выбирает backend по реальным p95-латентности, error rate и семантике запроса.

---

## Быстрый старт

```bash
git clone https://github.com/safal207/DAO_lim.git
cd DAO_lim
cargo build --release

./target/release/dao --config configs/dao.toml
```

Запустить отладку маршрутизации:

```bash
./target/release/daoctl explain \
  --host llm.myapp.com \
  --path /v1/chat/completions \
  --intent realtime
```

```
  ✓ Выбран: gpt-4o-mini-pool

  UPSTREAM               SCORE     p95ms     err%       RPS   load×w  intent×w
  ────────────────────────────────────────────────────────────────────────────
▶ gpt-4o-mini-pool     0.0540      23.0      0.0%      87.3   0.0540    0.0000
· gpt-4-turbo-pool     0.3800     450.0      1.2%      12.1   0.3600    0.0000
· local-llama-70b      0.9100    1200.0      4.5%       3.2   0.9000    0.0000

    ┌ Почему выбран gpt-4o-mini-pool:
    │   load_resonance  = 0.0900  (p95=23ms, err=0.0%)
    │   intent_gap      = 0.0000  (intent совпадает)
    └   score = 0.60×0.0900 + 0.30×0.0000 + 0.10×0.0000 = 0.0540
```

---

## Типичные сценарии для AI

### 1. Разные модели для разных задач

```toml
# Чат-запросы → быстрая модель
[[routes.rule]]
name    = "chat"
intent  = "realtime"
policy  = "resonant"

  [routes.rule.match]
  path_prefix = "/v1/chat"

  [[routes.rule.upstreams]]
  name   = "gpt-4o-mini"
  url    = "http://openai-proxy:8080"
  intent = ["realtime", "low-latency"]
  weight = 3

  [[routes.rule.upstreams]]
  name   = "local-llama-8b"
  url    = "http://gpu-node-1:11434"
  intent = ["realtime"]
  weight = 1

# Аналитика → мощная модель
[[routes.rule]]
name   = "analysis"
intent = "batch"
policy = "resonant"

  [routes.rule.match]
  path_prefix = "/v1/analyze"

  [[routes.rule.upstreams]]
  name   = "gpt-4-turbo"
  url    = "http://openai-proxy:8081"
  intent = ["batch", "high-quality"]
  weight = 1

  [[routes.rule.upstreams]]
  name   = "local-llama-70b"
  url    = "http://gpu-node-2:11434"
  intent = ["batch"]
  weight = 2
```

### 2. Streaming-ответы

```toml
[[routes.rule]]
name   = "streaming"
intent = "streaming"

  [routes.rule.match]
  path_prefix = "/v1/stream"
  upgrade     = "websocket"

  [[routes.rule.upstreams]]
  name   = "streaming-backend"
  url    = "ws://gpu-node-1:11434"
  intent = ["streaming", "realtime"]
```

### 3. Автоматический fallback при перегрузке

DAO отслеживает p95-латентность и error rate в реальном времени. Если `gpu-node-1` начинает возвращать ошибки — трафик автоматически перетекает к `gpu-node-2` без изменения конфига.

Настройка весов:
```toml
[policies.ai-balanced]
w_load   = 0.7   # Больше веса нагрузке (latency + errors)
w_intent = 0.2   # Меньше — intent matching
w_tempo  = 0.1   # Меньше — RPS spikiness
```

---

## Как работает выбор backend

```
score = w_load × load_resonance
      + w_intent × intent_gap
      + w_tempo × tempo_spikiness
```

| Компонент | Что измеряет | Значение |
|-----------|-------------|---------|
| `load_resonance` | p95 latency + error rate + queue | 0 = idle, 10 = перегружен |
| `intent_gap` | Совпадение тега с backend | 0 = совпадает, 1 = нет |
| `tempo_spikiness` | Вариативность RPS (CV) | 0 = стабильный, >1 = спайки |

**Backend с минимальным score выигрывает.**

Команда `daoctl explain` показывает полный расчёт для любого запроса — без гадания.

---

## daoctl — инструмент отладки

```bash
# Объяснить решение маршрутизации
daoctl explain --host llm.myapp.com --path /v1/chat --intent realtime

# Состояние всех backend в реальном времени
daoctl upstreams

# Проверить что DAO живой
daoctl health

# Подключиться к другому экземпляру
daoctl --server http://prod-dao:9103 upstreams

# Сырой JSON для скриптов
daoctl explain --host api.example.com --path /v1/users --json | jq .
```

Admin API доступен на `http://127.0.0.1:9103` (настраивается через `server.admin_bind`).

---

## Архитектура

```
CLIENT REQUEST
      │
      ▼
  ┌───────┐     TCP/TLS accept, protocol negotiation
  │ Gate  │
  └───┬───┘
      │
      ▼
  ┌───────┐     p95 latency, error rate, RPS per upstream
  │ Sense │
  └───┬───┘
      │
      ▼
  ┌───────┐     resonant score → выбор лучшего backend
  │ Align │
  └───┬───┘
      │
      ▼
  ┌───────┐     header manipulation, rate limiting, WASM filters
  │ Flow  │
  └───┬───┘
      │
      ▼
  UPSTREAM
```

Состояние хранится в **Memory** — горячая перезагрузка конфигурации без рестарта, снапшоты для отката.

---

## Конфигурация

```toml
[server]
bind       = "0.0.0.0:8080"
admin_bind = "127.0.0.1:9103"    # daoctl подключается сюда
workers    = 4

[telemetry]
prometheus_bind = "0.0.0.0:9102"  # /metrics для Grafana

[[routes.rule]]
name   = "llm-gateway"
policy = "resonant"
intent = "realtime"

  [routes.rule.match]
  host        = "llm.myapp.com"
  path_prefix = "/v1/"

  [[routes.rule.upstreams]]
  name   = "fast-model"
  url    = "http://gpu-1:11434"
  intent = ["realtime", "low-latency"]
  weight = 2

  [[routes.rule.upstreams]]
  name   = "fallback-model"
  url    = "http://gpu-2:11434"
  intent = ["realtime"]
  weight = 1

[policies.resonant]
w_load   = 0.6
w_intent = 0.3
w_tempo  = 0.1
```

---

## Установка

**Требования:** Rust 1.75+ (`rustup`)

```bash
# Собрать всё
cargo build --release

# Бинарники
./target/release/dao      # Основной gateway
./target/release/daoctl   # CLI инструмент
```

---

## Метрики (Prometheus)

```bash
curl http://localhost:9102/metrics
```

Экспортируются: latency histograms, request counts, error rates, resonance scores — по каждому upstream отдельно. Готово для Grafana.

---

## Тесты

```bash
cargo test
cargo test -p dao-core    # только ядро
RUST_LOG=debug cargo test
```

---

## Дорожная карта

### ✅ Реализовано
- HTTP/1.1, HTTP/2, WebSocket
- Resonant load balancing с intent-routing
- Hot-reload конфигурации
- Prometheus метрики
- `daoctl` CLI (explain, upstreams, health)
- Admin HTTP API

### 🔄 В работе
- gRPC proxy
- Circuit breaker
- Полное WebSocket proxying

### 🚀 Планируется
- HTTP/3/QUIC
- Canary routing / A/B testing
- OpenTelemetry distributed tracing
- ML-based автотюнинг весов политик
- WASM plugin marketplace

---

## Стек

| Компонент | Технология |
|-----------|-----------|
| Runtime | tokio |
| HTTP | hyper + tower |
| TLS | rustls (без OpenSSL) |
| Метрики | prometheus |
| Config | TOML + hot-reload |
| Plugins | WebAssembly (wasmtime) |
| Latency stats | HDR histogram |

---

## Лицензия

MIT OR Apache-2.0

---

*DAO — умный шлюз для команд, которые запускают AI в продакшне.*
