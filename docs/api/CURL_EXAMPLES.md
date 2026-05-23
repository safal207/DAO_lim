# Local API and CLI examples

These examples are for local development and demos. They assume you have built
DAO_lim from the repository root and are using the sample configuration in
[`configs/dao.toml`](../../configs/dao.toml). For the broader project overview,
see the [README](../../README.md). For performance context, see
[`docs/BENCHMARKS.md`](../BENCHMARKS.md).

## Start DAO locally

```bash
cargo build --release
./target/release/dao --config configs/dao.toml
```

By default the sample config exposes:

- proxy traffic on `127.0.0.1:8080`
- the admin API used by `daoctl` on `127.0.0.1:9103`
- Prometheus metrics on `127.0.0.1:9102`

## Check daemon health

```bash
./target/release/daoctl health
```

Use `--server` when you need to point the CLI at a non-default admin API:

```bash
./target/release/daoctl --server http://127.0.0.1:9103 health
```

## Inspect upstream state

```bash
./target/release/daoctl upstreams
```

This asks the local admin API for the current upstream view configured for the
running DAO process.

## Explain a routing decision

```bash
./target/release/daoctl explain \
  --host llm.myapp.com \
  --path /v1/chat/completions \
  --intent realtime
```

The same explain command can emit raw JSON for scripts and debugging:

```bash
./target/release/daoctl explain \
  --host llm.myapp.com \
  --path /v1/chat/completions \
  --intent realtime \
  --json | jq .
```

## Query Prometheus metrics

```bash
curl http://127.0.0.1:9102/metrics
```

This endpoint is intended for local Prometheus/Grafana integration during demos
and development. Production deployments should choose bind addresses, firewall
rules, and scrape configuration deliberately rather than copying these examples
unchanged.
