# DAO_lim Landing Page Copy

## Hero

### Your AI traffic should not be routed by blind round-robin rules

DAO_lim is an intent-aware reverse proxy for AI infrastructure.
It routes requests based on latency, error rate, and semantic intent so teams
can stop guessing why traffic went to the wrong backend.

**Primary promise**

- route AI traffic with explainable decisions
- reduce silent failure and overloaded backend drift
- keep the control surface open, self-hosted, and inspectable

**CTA**

- Run the stack demo
- Inspect routing with `daoctl explain`

## Problem

Most proxy layers were not designed for AI traffic.

They do not understand the difference between:

- low-latency chat
- long-running batch work
- streaming responses
- degraded but still alive model pools

The result is predictable:

- expensive backends get hit at the wrong time
- overloaded nodes keep receiving traffic
- failover becomes opaque
- debugging turns into log archaeology

## Agitation

If you are operating multiple LLM or inference backends, the cost of bad
routing compounds fast:

- higher p95 latency
- avoidable 429 and 5xx bursts
- wasted GPU capacity
- no human-readable reason for a routing decision

When your routing layer is opaque, every backend incident becomes harder to
triage and slower to fix.

## Solution

DAO_lim gives you an AI-focused control layer with explicit routing logic.

It combines:

- intent-aware selection
- resonant load balancing
- live admin and telemetry surfaces
- explainable CLI inspection via `daoctl`

Instead of asking "which upstream won?", you can ask "why did this upstream
win?" and get a concrete answer.

## What you get

### 1. Intent-aware routing

Send `realtime`, `batch`, or `streaming` traffic to the backend that best fits
the request rather than whichever node happens to be next in line.

### 2. Explainable decisions

Use `daoctl explain` to inspect the reasoning path behind a routing result.

### 3. Open operational surface

Use the admin API, hot reload path, and metrics output to operate the router as
infrastructure instead of a black box.

### 4. Extensible architecture

Build on a Rust workspace with telemetry and WASM-oriented extension paths.

## Who this is for

- teams running several LLM or inference backends
- builders of self-hosted AI gateways
- infra engineers who want routing they can inspect and justify
- research stacks that need open control over model traffic

## Proof and credibility

- open-source Rust workspace
- admin API and `daoctl` control surface
- benchmark scaffold in `docs/BENCHMARKS.md`
- stack walkthrough in `docs/STACK_DEMO.md`
- NLnet-facing materials in `grants/`

## Call to action

If your AI infrastructure depends on multiple backends, static routing is
already a tax on reliability.

Use DAO_lim if you want a routing layer that is:

- explainable
- inspectable
- self-hosted
- designed for AI traffic rather than retrofitted to it

Start with:

- `docs/STACK_DEMO.md`
- `docs/BENCHMARKS.md`
- `grants/README.md`

## FAQ

### Is this a general-purpose reverse proxy?

It can cover common proxy scenarios, but the pitch is not "yet another proxy."
The value is AI-oriented routing logic and inspection.

### Does it already replace every mainstream proxy feature?

No. The strongest current value is explainable routing and open control, not
feature-parity marketing.

### Why not just use Nginx or Traefik?

Because those tools were not built around semantic intent and explicit routing
explanations for AI backends.
