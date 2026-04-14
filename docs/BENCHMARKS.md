# DAO_lim Benchmarks

This repository now includes a reproducible microbenchmark for the routing
decision path in `crates/dao-core/benches/resonant_routing.rs`.

## What it measures

- `select_upstream` latency for `4`, `8`, `16`, and `32` upstream candidates
- `explain_selection` latency for reviewer-facing decision output

These are microbenchmarks for the scoring and explain path only. They do not
replace end-to-end HTTP benchmarking against reverse proxies such as Traefik.

## Run locally

```bash
cargo bench -p dao-core --bench resonant_routing
```

Criterion will emit reports under `target/criterion/`.

## Why this matters for NLnet

- makes the routing core measurable without external infra
- gives a stable baseline before end-to-end proxy benchmarks
- keeps future regressions visible in CI by compiling the bench target

## Recommended next step

Add an end-to-end benchmark script that drives `dao` under load and compares:

- steady-state p95 latency
- failover behavior under backend errors
- explainability overhead with `daoctl explain`
