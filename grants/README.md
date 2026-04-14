# DAO_lim Grant Readiness

## Positioning

DAO_lim is the routing layer of the Liminal Stack: an intent-aware reverse proxy
for AI backends with explainable routing decisions, live backend scoring, and
reusable control surfaces.

## Why it fits NGI Zero Commons Fund

- open infrastructure component rather than a closed product
- reusable independently from the rest of the stack
- improves transparency at the routing layer
- supports self-hosted and interoperable deployments

## Grant-facing strengths visible in the repository

- Rust workspace with multiple crates
- `license = "MIT OR Apache-2.0"` in [Cargo.toml](../Cargo.toml)
- routing narrative already present in [README.md](../README.md)
- intent-aware routing, telemetry, and `daoctl` explain flow documented

## Readiness notes

- The repository now includes the missing MIT license text so the dual-license
  claim matches the repository contents.
- The README is technically strong, but a short fund-facing summary would still
  help reviewers who are not deep in AI infra routing.

## Recommended next fixes before submission

- add a short architecture diagram for reviewer onboarding
- add a one-page milestone list tied to the NLnet application
