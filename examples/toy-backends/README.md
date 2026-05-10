# DAO_lim Toy Backends

This directory contains a tiny Python HTTP backend used by the DAO_lim Docker Compose demo.

It is intentionally simple and uses only the Python standard library.

## Behavior

The server exposes:

- `GET /health` — health check endpoint.
- Any other `GET` path — JSON response with backend name, path, request count, and artificial delay.

## Environment variables

| Variable | Default | Meaning |
|---|---:|---|
| `BACKEND_NAME` | `toy-backend` | Name returned in JSON responses. |
| `DELAY_MS` | `10` | Artificial response delay in milliseconds. |
| `FAIL_MODE` | `0` | When `1`, `true`, or `yes`, `/health` and normal requests return `503`. |
| `PORT` | `8080` | Port the toy backend listens on. |

## Local run

```bash
BACKEND_NAME=fast-primary DELAY_MS=12 PORT=8080 python examples/toy-backends/server.py
```

Then:

```bash
curl http://localhost:8080/health
curl http://localhost:8080/v1/chat/completions
```

## Docker Compose demo

From the repository root:

```bash
docker compose up --build
```

See:

```text
docs/demo/DOCKER_COMPOSE_DEMO.md
```

## Scope

This backend is for local demos and benchmark-style development only. It is not a production backend and does not implement an OpenAI-compatible API response schema.
