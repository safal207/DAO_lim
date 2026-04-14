# DAO_lim Grant Landing

## DAO_lim

### Explainable routing for AI infrastructure

DAO_lim is the routing layer of the Liminal Stack.  
It is an open-source reverse proxy for AI and service backends that makes
routing decisions inspectable instead of opaque.

For grant reviewers, the core value is simple:

- better control over AI traffic
- explainable backend selection
- open infrastructure that can be reused independently

## Why this matters

Modern AI deployments often route traffic through generic proxies that were not
designed for:

- semantic request intent
- uneven model latency
- bursty backend degradation
- human-readable routing explanations

That creates a trust problem.

Infrastructure teams cannot easily answer:

- why did this request go to that backend?
- why did latency spike for this class of traffic?
- why did a degraded backend keep receiving requests?

DAO_lim addresses that gap directly.

## What DAO_lim contributes

DAO_lim gives the stack an explicit routing and control layer with:

- intent-aware routing
- resonant load balancing
- admin and telemetry surfaces
- inspectable decisions via `daoctl explain`

The point is not only to route traffic.
The point is to make routing behavior visible and accountable.

## Why it fits a grant

DAO_lim is relevant as a digital commons building block because it is:

- open-source infrastructure
- reusable outside the Liminal Stack
- designed for self-hosted deployment
- useful to downstream projects that need transparent AI traffic control

It is not an end-user app.
It is infrastructure that improves trust at the routing layer.

## Current reviewer evidence

- [Stack demo](STACK_DEMO.md)
- [Benchmarks](BENCHMARKS.md)
- [NLnet materials](../grants/README.md)

## What reviewers should remember

If the runtime and database become more trustworthy, but the routing layer is
still opaque, the stack remains incomplete.

DAO_lim fills that gap by making backend choice:

- explicit
- inspectable
- measurable

## Best next milestone

The strongest next milestone is end-to-end routing evidence against real
backends, with:

- benchmarked behavior
- visible failover logic
- traceable decision output
