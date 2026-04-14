## Reviewer docs

- [Stack demo](docs/STACK_DEMO.md)
- [Benchmarks](docs/BENCHMARKS.md)
- [NLnet grant materials](grants/README.md)

# DAO â€” Dynamic Awareness Orchestrator

**Ð£Ð¼Ð½Ñ‹Ð¹ reverse-proxy Ð´Ð»Ñ AI-Ð¸Ð½Ñ„Ñ€Ð°ÑÑ‚Ñ€ÑƒÐºÑ‚ÑƒÑ€Ñ‹. ÐœÐ°Ñ€ÑˆÑ€ÑƒÑ‚Ð¸Ð·Ð¸Ñ€ÑƒÐµÑ‚ Ñ‚Ñ€Ð°Ñ„Ð¸Ðº Ðº LLM Ð¸ Ð´Ñ€ÑƒÐ³Ð¸Ð¼ backend-ÑÐµÑ€Ð²Ð¸ÑÐ°Ð¼ Ð½Ð° Ð¾ÑÐ½Ð¾Ð²Ðµ Ñ€ÐµÐ°Ð»ÑŒÐ½Ñ‹Ñ… Ð¼ÐµÑ‚Ñ€Ð¸Ðº: Ð»Ð°Ñ‚ÐµÐ½Ñ‚Ð½Ð¾ÑÑ‚Ð¸, Ð¾ÑˆÐ¸Ð±Ð¾Ðº Ð¸ ÑÐµÐ¼Ð°Ð½Ñ‚Ð¸ÐºÐ¸ Ð·Ð°Ð¿Ñ€Ð¾ÑÐ°.**

---

## Ð—Ð°Ñ‡ÐµÐ¼ ÑÑ‚Ð¾ Ð½ÑƒÐ¶Ð½Ð¾

ÐšÐ¾Ð³Ð´Ð° Ñƒ Ð²Ð°Ñ Ð½ÐµÑÐºÐ¾Ð»ÑŒÐºÐ¾ AI-Ð±ÑÐºÐµÐ½Ð´Ð¾Ð² â€” Ñ€Ð°Ð·Ð½Ñ‹Ðµ Ð¼Ð¾Ð´ÐµÐ»Ð¸, GPU-Ð¿ÑƒÐ»Ñ‹, Ð¾Ð±Ð»Ð°Ñ‡Ð½Ñ‹Ðµ Ð¸ Ð»Ð¾ÐºÐ°Ð»ÑŒÐ½Ñ‹Ðµ ÑÐ½Ð´Ð¿Ð¾Ð¸Ð½Ñ‚Ñ‹ â€” Ð¾Ð±Ñ‹Ñ‡Ð½Ñ‹Ð¹ round-robin Ð½Ðµ Ñ€Ð°Ð±Ð¾Ñ‚Ð°ÐµÑ‚:

- ÐžÐ´Ð¸Ð½ GPU Ð¿ÐµÑ€ÐµÐ³Ñ€ÑƒÐ¶ÐµÐ½, Ð´Ñ€ÑƒÐ³Ð¾Ð¹ Ð¿Ñ€Ð¾ÑÑ‚Ð°Ð¸Ð²Ð°ÐµÑ‚
- Ð‘Ñ‹ÑÑ‚Ñ€Ð°Ñ Ð¼Ð¾Ð´ÐµÐ»ÑŒ Ð½ÑƒÐ¶Ð½Ð° Ð´Ð»Ñ Ñ‡Ð°Ñ‚Ð°, Ð¼ÐµÐ´Ð»ÐµÐ½Ð½Ð°Ñ â€” Ð´Ð»Ñ Ð±Ð°Ñ‚Ñ‡ÐµÐ²Ð¾Ð³Ð¾ Ð°Ð½Ð°Ð»Ð¸Ð·Ð°
- ÐŸÑ€Ð¾Ð²Ð°Ð¹Ð´ÐµÑ€ Ð²ÐµÑ€Ð½ÑƒÐ» 429, Ð½Ð°Ð´Ð¾ Ð¿ÐµÑ€ÐµÐºÐ»ÑŽÑ‡Ð¸Ñ‚ÑŒÑÑ Ð±ÐµÐ· Ð¿Ñ€Ð¾ÑÑ‚Ð¾Ñ
- ÐÐµÐ¿Ð¾Ð½ÑÑ‚Ð½Ð¾, Ð¿Ð¾Ñ‡ÐµÐ¼Ñƒ Ð·Ð°Ð¿Ñ€Ð¾Ñ ÑƒÑˆÑ‘Ð» Ð½Ð° Ð¼ÐµÐ´Ð»ÐµÐ½Ð½Ñ‹Ð¹ backend

DAO Ñ€ÐµÑˆÐ°ÐµÑ‚ ÑÑ‚Ð¸ Ð¿Ñ€Ð¾Ð±Ð»ÐµÐ¼Ñ‹ Ñ‡ÐµÑ€ÐµÐ· **intent-aware routing** Ð¸ **resonant load balancing** â€” Ð²Ñ‹Ð±Ð¸Ñ€Ð°ÐµÑ‚ backend Ð¿Ð¾ Ñ€ÐµÐ°Ð»ÑŒÐ½Ñ‹Ð¼ p95-Ð»Ð°Ñ‚ÐµÐ½Ñ‚Ð½Ð¾ÑÑ‚Ð¸, error rate Ð¸ ÑÐµÐ¼Ð°Ð½Ñ‚Ð¸ÐºÐµ Ð·Ð°Ð¿Ñ€Ð¾ÑÐ°.

---

## Ð‘Ñ‹ÑÑ‚Ñ€Ñ‹Ð¹ ÑÑ‚Ð°Ñ€Ñ‚

```bash
git clone https://github.com/safal207/DAO_lim.git
cd DAO_lim
cargo build --release

./target/release/dao --config configs/dao.toml
```

Ð—Ð°Ð¿ÑƒÑÑ‚Ð¸Ñ‚ÑŒ Ð¾Ñ‚Ð»Ð°Ð´ÐºÑƒ Ð¼Ð°Ñ€ÑˆÑ€ÑƒÑ‚Ð¸Ð·Ð°Ñ†Ð¸Ð¸:

```bash
./target/release/daoctl explain \
  --host llm.myapp.com \
  --path /v1/chat/completions \
  --intent realtime
```

```
  âœ“ Ð’Ñ‹Ð±Ñ€Ð°Ð½: gpt-4o-mini-pool

  UPSTREAM               SCORE     p95ms     err%       RPS   loadÃ—w  intentÃ—w
  â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
â–¶ gpt-4o-mini-pool     0.0540      23.0      0.0%      87.3   0.0540    0.0000
Â· gpt-4-turbo-pool     0.3800     450.0      1.2%      12.1   0.3600    0.0000
Â· local-llama-70b      0.9100    1200.0      4.5%       3.2   0.9000    0.0000

    â”Œ ÐŸÐ¾Ñ‡ÐµÐ¼Ñƒ Ð²Ñ‹Ð±Ñ€Ð°Ð½ gpt-4o-mini-pool:
    â”‚   load_resonance  = 0.0900  (p95=23ms, err=0.0%)
    â”‚   intent_gap      = 0.0000  (intent ÑÐ¾Ð²Ð¿Ð°Ð´Ð°ÐµÑ‚)
    â””   score = 0.60Ã—0.0900 + 0.30Ã—0.0000 + 0.10Ã—0.0000 = 0.0540
```

---

## Ð¢Ð¸Ð¿Ð¸Ñ‡Ð½Ñ‹Ðµ ÑÑ†ÐµÐ½Ð°Ñ€Ð¸Ð¸ Ð´Ð»Ñ AI

### 1. Ð Ð°Ð·Ð½Ñ‹Ðµ Ð¼Ð¾Ð´ÐµÐ»Ð¸ Ð´Ð»Ñ Ñ€Ð°Ð·Ð½Ñ‹Ñ… Ð·Ð°Ð´Ð°Ñ‡

```toml
# Ð§Ð°Ñ‚-Ð·Ð°Ð¿Ñ€Ð¾ÑÑ‹ â†’ Ð±Ñ‹ÑÑ‚Ñ€Ð°Ñ Ð¼Ð¾Ð´ÐµÐ»ÑŒ
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

# ÐÐ½Ð°Ð»Ð¸Ñ‚Ð¸ÐºÐ° â†’ Ð¼Ð¾Ñ‰Ð½Ð°Ñ Ð¼Ð¾Ð´ÐµÐ»ÑŒ
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

### 2. Streaming-Ð¾Ñ‚Ð²ÐµÑ‚Ñ‹

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

### 3. ÐÐ²Ñ‚Ð¾Ð¼Ð°Ñ‚Ð¸Ñ‡ÐµÑÐºÐ¸Ð¹ fallback Ð¿Ñ€Ð¸ Ð¿ÐµÑ€ÐµÐ³Ñ€ÑƒÐ·ÐºÐµ

DAO Ð¾Ñ‚ÑÐ»ÐµÐ¶Ð¸Ð²Ð°ÐµÑ‚ p95-Ð»Ð°Ñ‚ÐµÐ½Ñ‚Ð½Ð¾ÑÑ‚ÑŒ Ð¸ error rate Ð² Ñ€ÐµÐ°Ð»ÑŒÐ½Ð¾Ð¼ Ð²Ñ€ÐµÐ¼ÐµÐ½Ð¸. Ð•ÑÐ»Ð¸ `gpu-node-1` Ð½Ð°Ñ‡Ð¸Ð½Ð°ÐµÑ‚ Ð²Ð¾Ð·Ð²Ñ€Ð°Ñ‰Ð°Ñ‚ÑŒ Ð¾ÑˆÐ¸Ð±ÐºÐ¸ â€” Ñ‚Ñ€Ð°Ñ„Ð¸Ðº Ð°Ð²Ñ‚Ð¾Ð¼Ð°Ñ‚Ð¸Ñ‡ÐµÑÐºÐ¸ Ð¿ÐµÑ€ÐµÑ‚ÐµÐºÐ°ÐµÑ‚ Ðº `gpu-node-2` Ð±ÐµÐ· Ð¸Ð·Ð¼ÐµÐ½ÐµÐ½Ð¸Ñ ÐºÐ¾Ð½Ñ„Ð¸Ð³Ð°.

ÐÐ°ÑÑ‚Ñ€Ð¾Ð¹ÐºÐ° Ð²ÐµÑÐ¾Ð²:
```toml
[policies.ai-balanced]
w_load   = 0.7   # Ð‘Ð¾Ð»ÑŒÑˆÐµ Ð²ÐµÑÐ° Ð½Ð°Ð³Ñ€ÑƒÐ·ÐºÐµ (latency + errors)
w_intent = 0.2   # ÐœÐµÐ½ÑŒÑˆÐµ â€” intent matching
w_tempo  = 0.1   # ÐœÐµÐ½ÑŒÑˆÐµ â€” RPS spikiness
```

---

## ÐšÐ°Ðº Ñ€Ð°Ð±Ð¾Ñ‚Ð°ÐµÑ‚ Ð²Ñ‹Ð±Ð¾Ñ€ backend

```
score = w_load Ã— load_resonance
      + w_intent Ã— intent_gap
      + w_tempo Ã— tempo_spikiness
```

| ÐšÐ¾Ð¼Ð¿Ð¾Ð½ÐµÐ½Ñ‚ | Ð§Ñ‚Ð¾ Ð¸Ð·Ð¼ÐµÑ€ÑÐµÑ‚ | Ð—Ð½Ð°Ñ‡ÐµÐ½Ð¸Ðµ |
|-----------|-------------|---------|
| `load_resonance` | p95 latency + error rate + queue | 0 = idle, 10 = Ð¿ÐµÑ€ÐµÐ³Ñ€ÑƒÐ¶ÐµÐ½ |
| `intent_gap` | Ð¡Ð¾Ð²Ð¿Ð°Ð´ÐµÐ½Ð¸Ðµ Ñ‚ÐµÐ³Ð° Ñ backend | 0 = ÑÐ¾Ð²Ð¿Ð°Ð´Ð°ÐµÑ‚, 1 = Ð½ÐµÑ‚ |
| `tempo_spikiness` | Ð’Ð°Ñ€Ð¸Ð°Ñ‚Ð¸Ð²Ð½Ð¾ÑÑ‚ÑŒ RPS (CV) | 0 = ÑÑ‚Ð°Ð±Ð¸Ð»ÑŒÐ½Ñ‹Ð¹, >1 = ÑÐ¿Ð°Ð¹ÐºÐ¸ |

**Backend Ñ Ð¼Ð¸Ð½Ð¸Ð¼Ð°Ð»ÑŒÐ½Ñ‹Ð¼ score Ð²Ñ‹Ð¸Ð³Ñ€Ñ‹Ð²Ð°ÐµÑ‚.**

ÐšÐ¾Ð¼Ð°Ð½Ð´Ð° `daoctl explain` Ð¿Ð¾ÐºÐ°Ð·Ñ‹Ð²Ð°ÐµÑ‚ Ð¿Ð¾Ð»Ð½Ñ‹Ð¹ Ñ€Ð°ÑÑ‡Ñ‘Ñ‚ Ð´Ð»Ñ Ð»ÑŽÐ±Ð¾Ð³Ð¾ Ð·Ð°Ð¿Ñ€Ð¾ÑÐ° â€” Ð±ÐµÐ· Ð³Ð°Ð´Ð°Ð½Ð¸Ñ.

---

## daoctl â€” Ð¸Ð½ÑÑ‚Ñ€ÑƒÐ¼ÐµÐ½Ñ‚ Ð¾Ñ‚Ð»Ð°Ð´ÐºÐ¸

```bash
# ÐžÐ±ÑŠÑÑÐ½Ð¸Ñ‚ÑŒ Ñ€ÐµÑˆÐµÐ½Ð¸Ðµ Ð¼Ð°Ñ€ÑˆÑ€ÑƒÑ‚Ð¸Ð·Ð°Ñ†Ð¸Ð¸
daoctl explain --host llm.myapp.com --path /v1/chat --intent realtime

# Ð¡Ð¾ÑÑ‚Ð¾ÑÐ½Ð¸Ðµ Ð²ÑÐµÑ… backend Ð² Ñ€ÐµÐ°Ð»ÑŒÐ½Ð¾Ð¼ Ð²Ñ€ÐµÐ¼ÐµÐ½Ð¸
daoctl upstreams

# ÐŸÑ€Ð¾Ð²ÐµÑ€Ð¸Ñ‚ÑŒ Ñ‡Ñ‚Ð¾ DAO Ð¶Ð¸Ð²Ð¾Ð¹
daoctl health

# ÐŸÐ¾Ð´ÐºÐ»ÑŽÑ‡Ð¸Ñ‚ÑŒÑÑ Ðº Ð´Ñ€ÑƒÐ³Ð¾Ð¼Ñƒ ÑÐºÐ·ÐµÐ¼Ð¿Ð»ÑÑ€Ñƒ
daoctl --server http://prod-dao:9103 upstreams

# Ð¡Ñ‹Ñ€Ð¾Ð¹ JSON Ð´Ð»Ñ ÑÐºÑ€Ð¸Ð¿Ñ‚Ð¾Ð²
daoctl explain --host api.example.com --path /v1/users --json | jq .
```

Admin API Ð´Ð¾ÑÑ‚ÑƒÐ¿ÐµÐ½ Ð½Ð° `http://127.0.0.1:9103` (Ð½Ð°ÑÑ‚Ñ€Ð°Ð¸Ð²Ð°ÐµÑ‚ÑÑ Ñ‡ÐµÑ€ÐµÐ· `server.admin_bind`).

---

## ÐÑ€Ñ…Ð¸Ñ‚ÐµÐºÑ‚ÑƒÑ€Ð°

```
CLIENT REQUEST
      â”‚
      â–¼
  â”Œâ”€â”€â”€â”€â”€â”€â”€â”     TCP/TLS accept, protocol negotiation
  â”‚ Gate  â”‚
  â””â”€â”€â”€â”¬â”€â”€â”€â”˜
      â”‚
      â–¼
  â”Œâ”€â”€â”€â”€â”€â”€â”€â”     p95 latency, error rate, RPS per upstream
  â”‚ Sense â”‚
  â””â”€â”€â”€â”¬â”€â”€â”€â”˜
      â”‚
      â–¼
  â”Œâ”€â”€â”€â”€â”€â”€â”€â”     resonant score â†’ Ð²Ñ‹Ð±Ð¾Ñ€ Ð»ÑƒÑ‡ÑˆÐµÐ³Ð¾ backend
  â”‚ Align â”‚
  â””â”€â”€â”€â”¬â”€â”€â”€â”˜
      â”‚
      â–¼
  â”Œâ”€â”€â”€â”€â”€â”€â”€â”     header manipulation, rate limiting, WASM filters
  â”‚ Flow  â”‚
  â””â”€â”€â”€â”¬â”€â”€â”€â”˜
      â”‚
      â–¼
  UPSTREAM
```

Ð¡Ð¾ÑÑ‚Ð¾ÑÐ½Ð¸Ðµ Ñ…Ñ€Ð°Ð½Ð¸Ñ‚ÑÑ Ð² **Memory** â€” Ð³Ð¾Ñ€ÑÑ‡Ð°Ñ Ð¿ÐµÑ€ÐµÐ·Ð°Ð³Ñ€ÑƒÐ·ÐºÐ° ÐºÐ¾Ð½Ñ„Ð¸Ð³ÑƒÑ€Ð°Ñ†Ð¸Ð¸ Ð±ÐµÐ· Ñ€ÐµÑÑ‚Ð°Ñ€Ñ‚Ð°, ÑÐ½Ð°Ð¿ÑˆÐ¾Ñ‚Ñ‹ Ð´Ð»Ñ Ð¾Ñ‚ÐºÐ°Ñ‚Ð°.

---

## ÐšÐ¾Ð½Ñ„Ð¸Ð³ÑƒÑ€Ð°Ñ†Ð¸Ñ

```toml
[server]
bind       = "0.0.0.0:8080"
admin_bind = "127.0.0.1:9103"    # daoctl Ð¿Ð¾Ð´ÐºÐ»ÑŽÑ‡Ð°ÐµÑ‚ÑÑ ÑÑŽÐ´Ð°
workers    = 4

[telemetry]
prometheus_bind = "0.0.0.0:9102"  # /metrics Ð´Ð»Ñ Grafana

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

## Ð£ÑÑ‚Ð°Ð½Ð¾Ð²ÐºÐ°

**Ð¢Ñ€ÐµÐ±Ð¾Ð²Ð°Ð½Ð¸Ñ:** Rust 1.75+ (`rustup`)

```bash
# Ð¡Ð¾Ð±Ñ€Ð°Ñ‚ÑŒ Ð²ÑÑ‘
cargo build --release

# Ð‘Ð¸Ð½Ð°Ñ€Ð½Ð¸ÐºÐ¸
./target/release/dao      # ÐžÑÐ½Ð¾Ð²Ð½Ð¾Ð¹ gateway
./target/release/daoctl   # CLI Ð¸Ð½ÑÑ‚Ñ€ÑƒÐ¼ÐµÐ½Ñ‚
```

---

## ÐœÐµÑ‚Ñ€Ð¸ÐºÐ¸ (Prometheus)

```bash
curl http://localhost:9102/metrics
```

Ð­ÐºÑÐ¿Ð¾Ñ€Ñ‚Ð¸Ñ€ÑƒÑŽÑ‚ÑÑ: latency histograms, request counts, error rates, resonance scores â€” Ð¿Ð¾ ÐºÐ°Ð¶Ð´Ð¾Ð¼Ñƒ upstream Ð¾Ñ‚Ð´ÐµÐ»ÑŒÐ½Ð¾. Ð“Ð¾Ñ‚Ð¾Ð²Ð¾ Ð´Ð»Ñ Grafana.

---

## Ð¢ÐµÑÑ‚Ñ‹

```bash
cargo test
cargo test -p dao-core    # Ñ‚Ð¾Ð»ÑŒÐºÐ¾ ÑÐ´Ñ€Ð¾
RUST_LOG=debug cargo test
```

---

## Ð”Ð¾Ñ€Ð¾Ð¶Ð½Ð°Ñ ÐºÐ°Ñ€Ñ‚Ð°

### âœ… Ð ÐµÐ°Ð»Ð¸Ð·Ð¾Ð²Ð°Ð½Ð¾
- HTTP/1.1, HTTP/2, WebSocket
- Resonant load balancing Ñ intent-routing
- Hot-reload ÐºÐ¾Ð½Ñ„Ð¸Ð³ÑƒÑ€Ð°Ñ†Ð¸Ð¸
- Prometheus Ð¼ÐµÑ‚Ñ€Ð¸ÐºÐ¸
- `daoctl` CLI (explain, upstreams, health)
- Admin HTTP API

### ðŸ”„ Ð’ Ñ€Ð°Ð±Ð¾Ñ‚Ðµ
- gRPC proxy
- Circuit breaker
- ÐŸÐ¾Ð»Ð½Ð¾Ðµ WebSocket proxying

### ðŸš€ ÐŸÐ»Ð°Ð½Ð¸Ñ€ÑƒÐµÑ‚ÑÑ
- HTTP/3/QUIC
- Canary routing / A/B testing
- OpenTelemetry distributed tracing
- ML-based Ð°Ð²Ñ‚Ð¾Ñ‚ÑŽÐ½Ð¸Ð½Ð³ Ð²ÐµÑÐ¾Ð² Ð¿Ð¾Ð»Ð¸Ñ‚Ð¸Ðº
- WASM plugin marketplace

---

## Ð¡Ñ‚ÐµÐº

| ÐšÐ¾Ð¼Ð¿Ð¾Ð½ÐµÐ½Ñ‚ | Ð¢ÐµÑ…Ð½Ð¾Ð»Ð¾Ð³Ð¸Ñ |
|-----------|-----------|
| Runtime | tokio |
| HTTP | hyper + tower |
| TLS | rustls (Ð±ÐµÐ· OpenSSL) |
| ÐœÐµÑ‚Ñ€Ð¸ÐºÐ¸ | prometheus |
| Config | TOML + hot-reload |
| Plugins | WebAssembly (wasmtime) |
| Latency stats | HDR histogram |

---

## Ð›Ð¸Ñ†ÐµÐ½Ð·Ð¸Ñ

MIT OR Apache-2.0

---

*DAO â€” ÑƒÐ¼Ð½Ñ‹Ð¹ ÑˆÐ»ÑŽÐ· Ð´Ð»Ñ ÐºÐ¾Ð¼Ð°Ð½Ð´, ÐºÐ¾Ñ‚Ð¾Ñ€Ñ‹Ðµ Ð·Ð°Ð¿ÑƒÑÐºÐ°ÑŽÑ‚ AI Ð² Ð¿Ñ€Ð¾Ð´Ð°ÐºÑˆÐ½Ðµ.*

