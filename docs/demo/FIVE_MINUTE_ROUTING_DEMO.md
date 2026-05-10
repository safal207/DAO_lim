# Five-Minute Explainable Routing Demo

**Goal:** see DAO_lim's core value quickly: it routes by backend health + request intent and explains why a backend was selected.

This is a local reviewer/contributor demo. It is not a production deployment guide and does not make production performance claims.

## What you will see

In five minutes, you should be able to:

1. build DAO_lim,
2. start the gateway with the sample config,
3. ask `daoctl` why a request would be routed to a backend,
4. read the selected upstream and score explanation,
5. understand why static round-robin is not enough for AI backends.

## Core idea

DAO_lim sits between clients and AI/backend services.

```text
client request -> gate -> sense backend health -> align intent/load -> route/fallback -> explain decision
```

The important part is the explainability loop:

```text
request context + backend health + intent match -> score -> selected backend -> daoctl explain
```

## Prerequisites

- Rust toolchain installed.
- Repository cloned locally.
- Run commands from the repository root.

```bash
git clone https://github.com/safal207/DAO_lim.git
cd DAO_lim
```

## Step 1 — Build the gateway and CLI

```bash
cargo build --release
```

This builds the main gateway binary and the `daoctl` inspection tool.

Expected binaries:

```text
./target/release/dao
./target/release/daoctl
```

## Step 2 — Start DAO_lim with the sample config

The sample config is:

```text
configs/dao.toml
```

It includes an `api-v1` route with:

- host: `api.example.com`
- path prefix: `/v1/`
- route intent: `realtime`
- upstreams:
  - `api-backend-1` with intents `realtime`, `low-latency`
  - `api-backend-2` with intent `realtime`
- resonant policy weights:
  - `w_load = 0.6`
  - `w_intent = 0.3`
  - `w_tempo = 0.1`

Start the gateway:

```bash
./target/release/dao --config configs/dao.toml
```

In another terminal, use `daoctl`.

## Step 3 — Ask DAO_lim to explain a routing decision

```bash
./target/release/daoctl explain \
  --host api.example.com \
  --path /v1/chat/completions \
  --intent realtime
```

If your `daoctl` needs an explicit admin server URL, use:

```bash
./target/release/daoctl \
  --server http://127.0.0.1:9103 \
  explain \
  --host api.example.com \
  --path /v1/chat/completions \
  --intent realtime
```

## Step 4 — Read the explanation

A typical explanation should help you answer:

- Which upstream was selected?
- What were the candidate upstreams?
- Which backend had lower load/resonance score?
- Did the request intent match the backend intent?
- Did errors or latency push the decision away from an unhealthy backend?

The README shows the intended shape of the output:

```text
✓ Selected: api-backend-1

UPSTREAM          SCORE     p95ms     err%       RPS   load×w  intent×w
────────────────────────────────────────────────────────────────────────
▶ api-backend-1   0.0540      23.0      0.0%      87.3   0.0540    0.0000
· api-backend-2   0.3800     450.0      1.2%      12.1   0.3600    0.0000

Why api-backend-1 was selected:
  load_resonance = lower latency/error/load signal
  intent_gap     = 0.0000 because request intent matched backend intent
  score          = w_load×load + w_intent×intent + w_tempo×tempo
```

The exact numbers depend on runtime metrics and backend state.

## Step 5 — Understand the score formula

DAO_lim's resonant policy uses this scoring shape:

```text
score = w_load × load_resonance
      + w_intent × intent_gap
      + w_tempo × tempo_spikiness
```

Lower score wins.

| Component | Meaning | Why it matters |
|---|---|---|
| `load_resonance` | latency + errors + queue/load signal | Avoid overloaded or failing backends. |
| `intent_gap` | mismatch between request intent and backend intent | Prefer the backend suited for the request. |
| `tempo_spikiness` | request-rate variability | Avoid unstable traffic patterns. |

For AI infrastructure, this matters because different backends often have different roles:

- fast model for realtime chat,
- stronger model for batch analysis,
- local model as fallback,
- cloud endpoint as overflow,
- tool server for specialized calls.

Static round-robin cannot explain or adapt to those differences.

## Step 6 — Optional JSON output for scripts

```bash
./target/release/daoctl explain \
  --host api.example.com \
  --path /v1/chat/completions \
  --intent realtime \
  --json | jq .
```

This is useful for tests, dashboards, and future integration with tracing or audit systems.

## Step 7 — Optional metrics check

If the gateway is running with Prometheus enabled, check:

```bash
curl http://localhost:9102/metrics
```

Metrics are useful for observing latency, request counts, errors, and routing-related signals.

## What this demo proves

This demo shows that DAO_lim has a local path for:

- building the gateway and CLI,
- loading a routing config,
- inspecting a routing decision,
- exposing the reasoning behind backend selection.

## What this demo does not prove

This demo does not prove:

- production readiness,
- universal performance superiority over mature proxies,
- complete security gateway certification,
- behavior across all real AI-provider APIs,
- multi-region or multi-tenant reliability.

For measured benchmark evidence, see:

- `docs/BENCHMARKS.md`
- `docs/evidence/BENCHMARK_EVIDENCE_SNAPSHOT.md`

## Next steps

After this demo, useful next tasks are:

1. validate the quickstart on a clean machine,
2. add Docker Compose demo with two toy AI backends,
3. add curl/daoctl/admin API examples,
4. add a local failover smoke test,
5. add benchmark/failover status badges to README.
