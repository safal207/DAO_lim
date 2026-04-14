# Liminal Stack Demo

This document describes a minimal end-to-end demo for the current Liminal
Stack:

```text
DAO_lim -> GardenLiminal -> LiminalDB
```

The goal is not full production deployment. The goal is to let a reviewer run a
short scenario and observe:

- LiminalDB accepting state and audit traffic
- GardenLiminal emitting lifecycle events into LiminalDB
- DAO_lim running with inspectable routing and admin surfaces

## Repositories

- DAO_lim: https://github.com/safal207/DAO_lim
- LiminalBD: https://github.com/safal207/LiminalBD
- GardenLiminal: https://github.com/safal207/GardenLiminal

## 1. Start LiminalDB

From the `LiminalBD` repository:

```bash
cargo build --release -p liminal-cli
./target/release/liminal-cli --store ./data --ws-port 8787
```

Reviewer checkpoints:

- the CLI starts successfully
- WebSocket is listening on `ws://127.0.0.1:8787`
- `:status` shows live runtime state

Useful commands inside the CLI:

```text
:status
:mirror top 10
```

## 2. Run GardenLiminal with LiminalDB audit storage

From the `GardenLiminal` repository:

```bash
cargo build --release
LIMINAL_URL=ws://127.0.0.1:8787 \
  sudo -E ./target/release/gl run -f examples/seed-busybox.yaml --store liminal
```

Reviewer checkpoints:

- the seed manifest validates and starts
- lifecycle events are emitted for process start and exit
- LiminalDB receives the event stream

If you prefer the bundled helper script:

```bash
./examples/demo-liminaldb.sh
```

## 3. Inspect the event trail in LiminalDB

Back in the `LiminalBD` CLI, inspect recent history:

```text
:mirror top 20
```

The reviewer should see recent runtime events coming from GardenLiminal.

If `websocat` is available, a direct query path is:

```bash
echo '{"cmd":"mirror.timeline","top":20}' | websocat -n1 ws://127.0.0.1:8787
```

## 4. Start DAO_lim

From the `DAO_lim` repository:

```bash
cargo build --release
./target/release/dao --config configs/dao.toml
```

Reviewer checkpoints:

- DAO starts with admin API on `127.0.0.1:9103`
- metrics are exposed on `0.0.0.0:9102`
- routing decisions can be inspected via `daoctl`

## 5. Inspect DAO routing behavior

From another terminal in the `DAO_lim` repository:

```bash
./target/release/daoctl health
./target/release/daoctl upstreams
./target/release/daoctl explain \
  --host api.example.com \
  --path /v1/chat \
  --intent realtime
```

The reviewer should see:

- DAO is reachable through the admin API
- configured upstreams and their state
- an explainable routing decision rather than opaque round-robin behavior

## What this demo proves

This short scenario demonstrates three separate layers of the stack:

- `LiminalDB` as the state and audit layer
- `GardenLiminal` as the runtime and event producer
- `DAO_lim` as the inspectable routing layer

It does not yet claim a fully integrated production deployment path. It is a
reviewer-oriented evidence demo for:

- observability
- auditability
- explainability
- stack composition

## Recommended next improvement

The next stronger version of this demo would route a real request through
DAO_lim into a GardenLiminal-managed service and persist the resulting runtime
trail in LiminalDB as one continuous trace.
