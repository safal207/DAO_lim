# DAO_lim Benchmark Evidence Snapshot

**Status:** reviewer-facing evidence summary  
**Source of truth:** `docs/BENCHMARKS.md`  
**Scope:** local reproducible benchmark evidence for routing, explainability, and failover behavior.

This document summarizes the current benchmark evidence in DAO_lim for reviewers, pilot users, and contributors. It explains what the current benchmark does prove, what it does not prove, and how to reproduce the evidence locally.

## Executive summary

DAO_lim currently has two benchmark layers:

1. **Routing microbenchmark** — measures the core routing decision path.
2. **Local end-to-end benchmark** — runs a live `dao` process with two toy HTTP backends and measures steady-state routing plus failover behavior.

The strongest current evidence is not a universal performance claim. It is a reproducible local demonstration that DAO_lim can:

- route through a live proxy process,
- maintain successful responses during a measured local failover window,
- move traffic from `fast-primary` to `fallback-secondary` after induced `503` behavior,
- expose explain snapshots before and after the failure.

## What is measured

### 1. Routing microbenchmark

Source:

```text
crates/dao-core/benches/resonant_routing.rs
```

Run command:

```bash
cargo bench -p dao-core --bench resonant_routing
```

It measures:

- `select_upstream` latency for `4`, `8`, `16`, and `32` upstream candidates,
- `explain_selection` latency for reviewer-facing decision output.

This benchmark is focused on decision-path cost only. It does not measure full HTTP proxy behavior.

### 2. Local end-to-end benchmark

Sources:

```text
scripts/e2e_benchmark.py
configs/dao-benchmark.toml
```

Run command:

```bash
cargo build --release -p dao -p daoctl
python scripts/e2e_benchmark.py
```

The local e2e benchmark:

- starts two toy HTTP backends on `127.0.0.1:18081` and `127.0.0.1:18082`,
- starts `dao` with `configs/dao-benchmark.toml`,
- sends warmup traffic,
- measures steady-state latency through `127.0.0.1:19080`,
- switches the primary backend into failure mode,
- measures backend distribution and explain output during failover.

## First tracked local benchmark report

### Environment

- Date: `2026-04-16`
- Commit: `df68cf108200e107aea625d2d68fa2b2ad57f3ca`
- OS: `Windows 10 Home`, version `2009`
- CPU: `AMD Ryzen 7 5700U with Radeon Graphics`
- RAM: `16 GB`
- Rust toolchain: `rustc 1.93.0 (254b59607 2026-01-19)`

### Benchmark setup

- DAO config: `configs/dao-benchmark.toml`
- Driver: `scripts/e2e_benchmark.py`
- Warmup requests: `20`
- Steady-state requests: `120`
- Failover requests: `30`

### Steady-state results

- Mean latency: `22.13 ms`
- p50 latency: `18.29 ms`
- p95 latency: `40.91 ms`
- Status counts: `{'200': 120}`
- Backend distribution: `{'fast-primary': 120}`

### Failover results

- Mean latency: `58.03 ms`
- p50 latency: `51.89 ms`
- p95 latency: `75.23 ms`
- Status counts: `{'200': 30}`
- Backend distribution: `{'fallback-secondary': 30}`

### Explain snapshot before failure

- Selected upstream: `fast-primary`
- Candidate states:
  - `fast-primary`: `winner=true`, `circuit_open=false`
  - `fallback-secondary`: `winner=false`, `circuit_open=false`

### Explain snapshot after failure

- Selected upstream: `fallback-secondary`
- Candidate states:
  - `fallback-secondary`: `winner=true`, `circuit_open=false`
  - `fast-primary`: `winner=false`, `circuit_open=true`

## What this evidence supports

The current benchmark evidence supports these narrow claims:

- DAO_lim has a reproducible local benchmark path.
- The routing decision path can be benchmarked independently.
- A live local `dao` process can be benchmarked through the proxy path.
- The benchmark can simulate a primary backend failure.
- In the tracked local run, traffic moved from `fast-primary` to `fallback-secondary` after induced failure.
- Explain output showed the selected backend and circuit state before and after failure.

## What this evidence does not prove

This evidence does **not** prove:

- production-grade reliability,
- universal performance across hardware,
- superiority over Envoy, NGINX, HAProxy, Traefik, or cloud load balancers,
- production security gateway certification,
- mature multi-tenant isolation,
- complete LLM gateway policy enforcement,
- zero-downtime behavior across all failure modes,
- performance under real provider APIs or GPU clusters.

The current benchmark is a local reproducible harness, not a production benchmark report.

## How reviewers should interpret it

The right reviewer interpretation is:

> DAO_lim has early measurable evidence for explainable routing and local failover behavior, with clear reproduction commands and caveats.

The wrong interpretation is:

> DAO_lim has proven universal performance superiority over mature production proxies.

## Reproduction commands

Microbenchmark:

```bash
cargo bench -p dao-core --bench resonant_routing
```

Local e2e benchmark:

```bash
cargo build --release -p dao -p daoctl
python scripts/e2e_benchmark.py
```

Shorter local e2e run:

```bash
python scripts/e2e_benchmark.py --steady-requests 60 --failover-requests 20
```

## Recommended next benchmark improvements

High-value next steps:

1. Add a Linux benchmark run with host specs and commit SHA.
2. Add a macOS benchmark run with host specs and commit SHA.
3. Track benchmark output as a generated artifact.
4. Add a local failover smoke test to CI or a manual validation workflow.
5. Compare against static round-robin under the same toy-backend harness.
6. Later, compare against Traefik or NGINX under a carefully documented setup.
7. Add OpenTelemetry spans for routing decisions and benchmark trace export.

## Evidence principle

DAO_lim benchmark language should stay tied to measured artifacts.

Good:

> In the tracked local benchmark, DAO_lim routed steady-state traffic to `fast-primary` and moved failover traffic to `fallback-secondary` after induced `503` behavior.

Avoid:

> DAO_lim is faster than all existing proxies.

Good:

> Current evidence is local, reproducible, and reviewer-friendly.

Avoid:

> Current evidence proves production readiness.
