# Start Here — Contributing to DAO_lim

DAO_lim is an open-source Rust gateway for AI infrastructure. It routes requests across heterogeneous backends using backend health, latency, error-rate, load, and request-intent signals, while exposing explainable routing decisions through `daoctl`.

The short version:

> Static round-robin is not enough for AI backends. DAO_lim routes by health + intent and shows why each backend was selected.

## 10-minute onboarding path

1. Read the root `README.md` for the product story, quickstart, architecture, and roadmap.
2. Read `docs/GRANT_EVIDENCE.md` for reviewer-facing positioning, current evidence, and explicit non-claims.
3. Read `docs/BENCHMARKS.md` to understand the current benchmark harness and evidence boundaries.
4. Build and test the workspace locally:

```bash
cargo build --release
cargo test
cargo test -p dao-core
```

5. Try the routing explanation path:

```bash
./target/release/daoctl explain \
  --host llm.myapp.com \
  --path /v1/chat/completions \
  --intent realtime
```

6. Pick an issue labeled `good first issue` or `help wanted`.

## What DAO_lim is

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

DAO_lim's differentiator is not only proxying traffic. Its differentiator is **inspectable backend selection**.

## Core concepts

- **Gate** — accepts and normalizes incoming traffic.
- **Sense** — tracks latency, errors, RPS, and backend health.
- **Align** — computes routing score using load, intent, and tempo signals.
- **Flow** — forwards traffic to the selected backend and handles proxy concerns.
- **Intent-aware routing** — matches request intent such as `realtime`, `batch`, `streaming`, or `high-quality` against backend capabilities.
- **Resonant score** — weighted score used to choose the best backend.
- **`daoctl explain`** — debugging path that shows why a backend was selected.
- **Benchmark harness** — local reproducible setup for measuring routing and failover behavior.

## Local validation

Build the binaries:

```bash
cargo build --release
```

Run tests:

```bash
cargo test
cargo test -p dao-core
```

Run with debug logs when needed:

```bash
RUST_LOG=debug cargo test
```

Run the gateway locally:

```bash
./target/release/dao --config configs/dao.toml
```

Inspect routing:

```bash
./target/release/daoctl explain \
  --host llm.myapp.com \
  --path /v1/chat/completions \
  --intent realtime
```

Check metrics if the gateway is running with Prometheus enabled:

```bash
curl http://localhost:9102/metrics
```

## Safe contribution zones

These are good places for new contributors:

Start with **documentation, demos, and example configs** under `docs/`, `examples/`, and `configs/`. That work is usually easy to review and does not change live routing behavior. **Routing score, policy, and core selection logic** live in `crates/dao-core/` (and related proxy paths) — treat those as a separate, higher-stakes area; use the list in **Changes that need deeper review** as a guide.

- Documentation improvements.
- Clean-machine quickstart validation.
- `daoctl` usage examples.
- Docker Compose local demo.
- Benchmark evidence summaries.
- README badges and visual polish.
- Example configs for AI backends.
- Local toy-backend demos.
- Tests that preserve existing routing semantics.

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
