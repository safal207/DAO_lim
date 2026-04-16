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

If Windows build tooling is broken on your machine, see `docs/TOOLCHAIN_FIX.md`
for the exact GNU/MSVC recovery paths captured during benchmark setup.

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

## First local benchmark report

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

### Notes

- Primary backend was switched into `503` mode before the failover run.
- In this local harness, the proxy maintained `200` responses across the measured
  failover window while moving traffic to the secondary backend.
- This is a single-machine local benchmark, not yet a proxy-vs-proxy comparison.

## Benchmark report template

```markdown
## DAO end-to-end benchmark report

### Environment
- Date: YYYY-MM-DD
- Commit: `xxxxxxxx`
- OS: Windows/Linux + version
- CPU: ...
- RAM: ...
- Rust toolchain: `rustc --version`

### Benchmark setup
- DAO config: `configs/dao-benchmark.toml`
- Driver: `scripts/e2e_benchmark.py`
- Warmup requests: N
- Steady-state requests: N
- Failover requests: N

### Steady-state results
- Mean latency: X ms
- p50 latency: X ms
- p95 latency: X ms
- Status counts: `...`
- Backend distribution: `...`

### Failover results
- Mean latency: X ms
- p50 latency: X ms
- p95 latency: X ms
- Status counts: `...`
- Backend distribution: `...`

### Explain snapshot before failure
- Selected upstream: `...`
- Candidate states:
  - `fast-primary`: `winner=true/false`, `circuit_open=true/false`
  - `fallback-secondary`: `winner=true/false`, `circuit_open=true/false`

### Explain snapshot after failure
- Selected upstream: `...`
- Candidate states:
  - `fast-primary`: `winner=true/false`, `circuit_open=true/false`
  - `fallback-secondary`: `winner=true/false`, `circuit_open=true/false`

### Notes
- Primary backend was switched to `503` mode before failover run.
- This is a local reproducible benchmark harness, not yet a proxy-vs-proxy comparison.
- Next step: compare against Traefik or Nginx under the same local setup.
```

## Recommended next step

Publish one benchmark report generated from this harness that records:

- steady-state p95 latency on named hardware
- failover error window before the circuit opens
- explain output before and after failure
- commit SHA and benchmark date
