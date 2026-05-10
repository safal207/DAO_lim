# Start Here — Contributing to DAO_lim

## What is DAO_lim?

**In plain terms:** DAO_lim is an open-source **Rust gateway** you run in front of multiple AI backends (cloud APIs, local GPUs, different models). It forwards client traffic and **chooses which backend to use** using live signals such as latency, errors, load, and how well the request’s **intent** (for example `realtime` vs `batch`) matches what each backend is good at.

**Core product idea:** **explainable AI gateway routing** — decisions use **backend health** (latency, errors, load) and **request intent**, and you can **inspect why** a backend was picked via `daoctl explain` instead of guessing.

**Slightly more detail:** DAO_lim routes requests across heterogeneous backends using those signals and exposes explainable routing decisions through `daoctl`.

The one-line summary:

> Static round-robin is not enough for AI backends. DAO_lim routes by health + intent and shows why each backend was selected.

## Build and test

From the repository root:

```bash
cargo build --release
cargo test
cargo test -p dao-core
```

Optional verbose tests:

```bash
RUST_LOG=debug cargo test
```

## Explain a routing decision (`daoctl explain`)

After `cargo build --release`, you can ask the tooling how a request would be reasoned about (host, path, intent):

```bash
./target/release/daoctl explain \
  --host llm.myapp.com \
  --path /v1/chat/completions \
  --intent realtime
```

## Contributing: docs and demos vs routing core

This split matches how we review PRs.

### Docs, demos, and examples (usually lower risk)

Good first contributions tend to live here — they improve onboarding without changing core routing math:

- **Documentation** — clearer guides, typos, navigation (including this file under `docs/`).
- **Demos** — walkthroughs under `docs/demo/`, Docker-based demos, toy backends under `examples/`.
- **Example configs** — realistic snippets under `configs/` for common AI setups.

These areas are **not** the same as changing how the gateway scores or selects backends.

### Routing core (higher risk — coordinate first)

Work that changes **how requests are scored, selected, or failed over** usually touches **`crates/dao-core/`** and related proxy/control paths in **`crates/dao/`**, **`crates/daoctl/`**, admin surfaces, or WASM filters. Treat that as **routing-core / behavior** work: open an issue or discuss before large changes, and expect deeper review. See **Changes that need deeper review** below for concrete examples.

### Safe contribution areas (examples)

Here are **three** typical safe zones (there are more in the bullets above):

1. Documentation and contributor onboarding (`docs/`, README polish).
2. Demos and reproducible examples (`docs/demo/`, `examples/toy-backends/`).
3. Example configs and benchmark/write-up improvements (`configs/`, benchmark docs — narrative and harness clarity, without changing routing semantics).

Additional ideas:

- Clean-machine quickstart validation.
- `daoctl` usage examples.
- Docker Compose local demo.
- Benchmark evidence summaries.
- README badges and visual polish.
- Example configs for AI backends.
- Local toy-backend demos.
- Tests that **preserve** existing routing semantics.

## 10-minute onboarding path

1. Read the root [`README.md`](../README.md) for the product story, quickstart, architecture, and roadmap.
2. Read [`docs/GRANT_EVIDENCE.md`](GRANT_EVIDENCE.md) for reviewer-facing positioning, current evidence, and explicit non-claims.
3. Read [`docs/BENCHMARKS.md`](BENCHMARKS.md) for the benchmark harness and evidence boundaries.
4. Run **Build and test** (section above).
5. Run **`daoctl explain`** (section above).
6. Pick an issue labeled `good first issue` or `help wanted`.

## What DAO_lim does in the pipeline

DAO_lim sits between clients and AI/backend services.

```text
client request -> gate -> sense backend health -> align intent/load -> route/fallback -> explain decision
```

It is designed for teams running multiple AI backends, such as:

- fast low-latency models,
- high-quality reasoning models,
- local model servers,
- cloud provider endpoints,
- fallback pools,
- tool servers,
- heterogeneous GPU nodes.

DAO_lim's differentiator is not only proxying traffic — it is **inspectable backend selection**.

## Core concepts

- **Gate** — accepts and normalizes incoming traffic.
- **Sense** — tracks latency, errors, RPS, and backend health.
- **Align** — computes routing score using load, intent, and tempo signals.
- **Flow** — forwards traffic to the selected backend and handles proxy concerns.
- **Intent-aware routing** — matches request intent such as `realtime`, `batch`, `streaming`, or `high-quality` against backend capabilities.
- **Resonant score** — weighted score used to choose the best backend.
- **`daoctl explain`** — debugging path that shows why a backend was selected.
- **Benchmark harness** — local reproducible setup for measuring routing and failover behavior.

## Run the gateway locally

```bash
./target/release/dao --config configs/dao.toml
```

With Prometheus enabled in config, metrics may be exposed (example):

```bash
curl http://localhost:9102/metrics
```

## Changes that need deeper review

Discuss these before implementation:

- Routing score formula changes.
- Circuit breaker semantics.
- Failover behavior changes.
- Config format changes.
- Admin API compatibility changes.
- Security-sensitive proxy behavior.
- Metrics naming changes.
- Claims comparing DAO_lim to Envoy, NGINX, HAProxy, Traefik, or cloud load balancers.
- Production-readiness, compliance, or security-certification claims.

## Recommended first issues

Good starting points:

1. Verify the quickstart on a clean machine.
2. Add a 5-minute explainable routing walkthrough.
3. Add `daoctl` / admin API / metrics examples.
4. Add Docker Compose demo with two toy AI backends.
5. Add benchmark evidence snapshot for reviewers.
6. Add benchmark/failover badges to README.

## Repository map

- `crates/dao/` — main gateway binary.
- `crates/dao-core/` — routing and core decision logic.
- `crates/daoctl/` — CLI inspection tool.
- `crates/dao-admin/` — admin/inspection surface.
- `crates/dao-telemetry/` — telemetry and metrics-related code.
- `crates/dao-filters/` — filter/plugin-related code.
- `configs/` — example TOML configurations.
- `scripts/` — benchmark and helper scripts.
- `docs/` — benchmark, grant evidence, and contributor-facing docs.

## Product boundary

DAO_lim is currently best described as:

> an open-source Rust prototype/product artifact for intent-aware, explainable AI/backend routing with early benchmark evidence.

It does **not** currently claim:

- production security gateway certification,
- full replacement of Envoy, NGINX, HAProxy, or cloud load balancers,
- universal routing optimality,
- verified performance across all hardware and workloads,
- mature enterprise compliance guarantees,
- stable pre-1.0 API/config compatibility.

## Contribution principle

A strong DAO_lim contribution should preserve three things:

1. **Explainability** — users should understand why a backend was selected.
2. **Reproducibility** — demos and benchmarks should be runnable locally.
3. **No overclaiming** — performance and production claims should stay tied to evidence.

## Further reading

- [README.md](../README.md) — product story, quickstart, `daoctl` examples, architecture, roadmap.
- [docs/BENCHMARKS.md](BENCHMARKS.md) — benchmark harness and how results are interpreted.
- [docs/GRANT_EVIDENCE.md](GRANT_EVIDENCE.md) — grant/reviewer-facing evidence and non-claims.
- [configs/dao.toml](../configs/dao.toml) — example gateway configuration (see also the [`configs/`](../configs/) directory).
- [docs/demo/FIVE_MINUTE_ROUTING_DEMO.md](demo/FIVE_MINUTE_ROUTING_DEMO.md) — short explainable-routing walkthrough.
- [docs/demo/DOCKER_COMPOSE_DEMO.md](demo/DOCKER_COMPOSE_DEMO.md) — Docker Compose–based local demo.
