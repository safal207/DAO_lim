# DAO_lim Benchmarks

This repository now includes a reproducible microbenchmark for the routing
decision path in `crates/dao-core/benches/resonant_routing.rs`.

It also now includes a small local end-to-end benchmark driver in
`scripts/e2e_benchmark.py` backed by `configs/dao-benchmark.toml`.

## What it measures

- `select_upstream` latency for `4`, `8`, `16`, and `32` upstream candidates
- `explain_selection` latency for reviewer-facing decision output

These are microbenchmarks for the scoring and explain path only. They do not
replace end-to-end HTTP benchmarking against reverse proxies such as Traefik.

The end-to-end benchmark adds:

- steady-state proxy latency through a live `dao` process
- backend distribution under healthy conditions
- failover behavior after the primary backend starts returning `503`
- admin explain snapshots before and after the induced failure burst

## Run locally

```bash
cargo bench -p dao-core --bench resonant_routing
```

Criterion will emit reports under `target/criterion/`.

## Run end-to-end benchmark locally

Build the binaries first:

```bash
cargo build --release -p dao -p daoctl
```

Then run:

```bash
python scripts/e2e_benchmark.py
```

The script will:

- start two local toy HTTP backends on `127.0.0.1:18081` and `127.0.0.1:18082`
- start `dao` with `configs/dao-benchmark.toml`
- send warmup traffic and measure steady-state latency through `127.0.0.1:19080`
- switch the primary backend into failure mode
- measure how traffic and explain output move during failover

You can tune request counts if you want faster runs during development:

```bash
python scripts/e2e_benchmark.py --steady-requests 60 --failover-requests 20
```

Example output fields:

- `mean latency`, `p50 latency`, `p95 latency`
- `status counts`
- `backend use`
- `selected` and `circuit_open` fields from `/admin/explain`

## Why this matters for NLnet

- makes the routing core measurable without external infra
- gives a stable baseline before end-to-end proxy benchmarks
- adds a reviewer-friendly live failover scenario on top of the microbench
- keeps future regressions visible in CI by compiling the bench target

## Evidence boundaries

- this is still a local reproducible harness, not a production benchmark report
- it does not yet compare against Traefik, Nginx, or Envoy
- it is best used to generate a first public benchmark report with host specs,
  OS, Rust toolchain version, and commit SHA

## Recommended next step

Publish one benchmark report generated from this harness that records:

- steady-state p95 latency on named hardware
- failover error window before the circuit opens
- explain output before and after failure
- commit SHA and benchmark date
