# Docker Compose Demo — DAO_lim with Two Toy AI Backends

**Goal:** run DAO_lim locally with two toy backends and inspect routing behavior through a live gateway.

This is a local developer-experience demo. It is not production deployment guidance and does not make production performance claims.

## What this demo starts

```text
client
  |
  v
DAO_lim gateway :19080
  |
  +--> fast-primary        :8080 inside Docker network
  |
  +--> fallback-secondary  :8080 inside Docker network
```

Exposed local ports:

| Service | Local port | Purpose |
|---|---:|---|
| DAO proxy | `19080` | Send demo traffic through the gateway. |
| DAO admin API | `19103` | Inspect health/explain state. |
| DAO metrics | `19102` | Prometheus-style metrics. |

Routing config:

```text
configs/dao-docker-demo.toml
```

Toy backend source:

```text
examples/toy-backends/server.py
```

## Prerequisites

- Docker with Compose support.
- Repository cloned locally.

```bash
git clone https://github.com/safal207/DAO_lim.git
cd DAO_lim
```

## Step 1 — Start the demo

```bash
docker compose up --build
```

The first run may take time because the `dao` service builds the Rust binary inside the container.

## Step 2 — Send traffic through DAO_lim

In another terminal:

```bash
curl -H "Host: bench.dao.local" http://localhost:19080/v1/chat/completions
```

Expected shape:

```json
{
  "status": "ok",
  "backend": "fast-primary",
  "path": "/v1/chat/completions",
  "request_count": 1,
  "delay_ms": 12.0
}
```

The exact backend can vary as routing state evolves, but the demo is configured so `fast-primary` is the lower-latency realtime backend.

## Step 3 — Inspect gateway health

```bash
curl http://localhost:19103/admin/health
```

Expected shape:

```json
{
  "status": "ok"
}
```

## Step 4 — Inspect routing explanation

```bash
curl "http://localhost:19103/admin/explain?host=bench.dao.local&path=/v1/chat/completions&method=GET&intent=realtime"
```

This should return a JSON explanation of the selected upstream and candidate state.

Useful fields to look for:

- selected upstream,
- candidate names,
- winner flags,
- circuit state,
- score-related fields if present.

## Step 5 — Check metrics

```bash
curl http://localhost:19102/metrics
```

This exposes local Prometheus-style metrics for the running demo.

## Step 6 — Stop the demo

```bash
docker compose down
```

## Optional — Simulate primary failure

The included Compose file starts both backends in healthy mode.

For a simple manual failure test, edit `docker-compose.yml` and change the `fast-primary` environment:

```yaml
FAIL_MODE: "1"
```

Then restart:

```bash
docker compose up --build
```

Send requests again:

```bash
curl -H "Host: bench.dao.local" http://localhost:19080/v1/chat/completions
```

Then inspect explain output:

```bash
curl "http://localhost:19103/admin/explain?host=bench.dao.local&path=/v1/chat/completions&method=GET&intent=realtime"
```

In a failure-oriented run, the expected direction is that traffic should move away from `fast-primary` toward `fallback-secondary`, and explain/circuit state should make that inspectable.

For a more controlled failover benchmark, use:

```bash
cargo build --release -p dao -p daoctl
python scripts/e2e_benchmark.py
```

## Demo boundaries

This demo proves that contributors can locally start:

- DAO_lim,
- two toy backends,
- proxy traffic through the gateway,
- admin health inspection,
- explain inspection,
- metrics inspection.

This demo does not prove:

- production readiness,
- production security posture,
- multi-tenant isolation,
- superiority over Envoy, NGINX, HAProxy, Traefik, or cloud load balancers,
- behavior against real OpenAI-compatible providers,
- universal failover behavior across all workloads.

For measured benchmark evidence, see:

- `docs/BENCHMARKS.md`
- `docs/evidence/BENCHMARK_EVIDENCE_SNAPSHOT.md`
