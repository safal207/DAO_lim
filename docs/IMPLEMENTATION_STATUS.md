# DAO_lim Implementation Status

This document is a reviewer-facing status snapshot for the current DAO_lim
codebase. Its purpose is to separate implemented functionality from partial
surfaces and roadmap work.

## Status summary

| Area | Status | Notes |
|---|---|---|
| HTTP proxy core | Implemented | `dao` starts, loads config, and exposes admin and metrics surfaces |
| Hot config reload | Implemented | In-memory config update path exists |
| Explainable routing | Implemented | `daoctl explain` and explain data types are present |
| Prometheus metrics | Implemented | Metrics exporter and upstream gauges/histograms are wired |
| Circuit breaker logic | Implemented | Config, upstream state handling, and tests are present |
| OpenTelemetry wiring | Partial | OTLP configuration and tracing layer exist, but cross-stack tracing is not yet a published evidence path |
| WebSocket proxying | Partial | Present in README as in-progress; not yet positioned as complete reviewer evidence |
| gRPC proxy support | Planned | Listed as next milestone, not current capability |
| WASM filter runtime | Partial | Crate and ABI surface exist, but instantiation still returns placeholder error |
| WASM plugin marketplace | Planned | Not implemented |
| End-to-end proxy benchmarks | Planned | Current repository only publishes routing-core microbenchmarks |

## What is implemented today

- routing decision logic and scoring live in `crates/dao-core`
- admin API and `daoctl` inspection surface are available
- Prometheus metrics and optional OTLP configuration are present
- circuit breaker state is represented in config and upstream health flow
- microbenchmarks exist for `select_upstream` and explain-path overhead

## Important caveats

### WASM filters

`crates/dao-filters` is present, but filter instantiation is still a
placeholder that returns `WASM filters not yet implemented`.

That means the honest positioning today is:

- WASM extension surface is under construction
- not "production-ready WASM plugin support"

### OpenTelemetry

OTLP configuration and tracing code exist, but the repository does not yet
publish a reviewer-facing end-to-end tracing demo across the full stack.

The honest positioning today is:

- DAO_lim contains tracing plumbing
- unified cross-stack observability remains a next milestone

### Benchmarks

The current benchmark package is useful, but narrow. It measures routing-core
decision cost rather than full reverse-proxy performance under network load.

## Recommended wording

Use wording like:

- "implements explainable routing and measurable routing-core benchmarks"
- "includes circuit breaker logic, Prometheus metrics, and admin inspection"
- "contains early WASM extension scaffolding"

Avoid wording like:

- "complete WASM plugin system"
- "published end-to-end proxy benchmark suite"
- "full cross-stack tracing evidence"

## Recommended next deliverables

1. publish an end-to-end benchmark against live HTTP backends
2. publish a failover scenario with circuit breaker evidence
3. publish one real OTLP tracing walkthrough
4. either implement a minimal working WASM filter or downgrade remaining claims further
