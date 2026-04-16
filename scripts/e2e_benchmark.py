#!/usr/bin/env python3
"""
Run a small end-to-end DAO benchmark with local toy backends.

What it measures:
- steady-state proxy latency through DAO
- selected backend distribution under healthy conditions
- failover behavior after the primary backend starts failing
- admin explain snapshot before and after the failure burst

This script intentionally uses only the Python standard library so reviewers
can run it without extra dependencies.
"""

from __future__ import annotations

import argparse
import json
import os
import signal
import socket
import statistics
import subprocess
import sys
import threading
import time
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any


DEFAULT_CONFIG = Path("configs/dao-benchmark.toml")
DEFAULT_DAO = Path("target/release/dao.exe" if os.name == "nt" else "target/release/dao")
DEFAULT_HOST = "bench.dao.local"
DEFAULT_PROXY_URL = "http://127.0.0.1:19080/v1/chat"
DEFAULT_ADMIN_URL = "http://127.0.0.1:19103"


class BackendState:
    def __init__(self, name: str, delay_ms: float) -> None:
        self.name = name
        self.delay_ms = delay_ms
        self.fail_mode = False
        self.request_count = 0
        self.lock = threading.Lock()

    def set_fail_mode(self, enabled: bool) -> None:
        with self.lock:
            self.fail_mode = enabled

    def snapshot(self) -> dict[str, Any]:
        with self.lock:
            return {
                "name": self.name,
                "delay_ms": self.delay_ms,
                "fail_mode": self.fail_mode,
                "request_count": self.request_count,
            }


def make_handler(state: BackendState):
    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
            with state.lock:
                state.request_count += 1
                fail_mode = state.fail_mode
                delay_ms = state.delay_ms
                request_count = state.request_count

            if self.path == "/health":
                status = 503 if fail_mode else 200
                payload = json.dumps(
                    {
                        "status": "fail" if fail_mode else "ok",
                        "backend": state.name,
                    }
                ).encode("utf-8")
                self.send_response(status)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)
                return

            time.sleep(delay_ms / 1000.0)

            if fail_mode:
                payload = json.dumps(
                    {
                        "backend": state.name,
                        "status": "error",
                        "request_count": request_count,
                    }
                ).encode("utf-8")
                self.send_response(503)
            else:
                payload = json.dumps(
                    {
                        "backend": state.name,
                        "status": "ok",
                        "request_count": request_count,
                    }
                ).encode("utf-8")
                self.send_response(200)

            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

        def log_message(self, format: str, *args: object) -> None:
            return

    return Handler


@dataclass
class RequestSample:
    latency_ms: float
    status: int
    backend: str


def percentile(values: list[float], pct: float) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    index = round((len(ordered) - 1) * pct)
    return ordered[index]


def fetch_json(url: str, headers: dict[str, str] | None = None, timeout: float = 2.0) -> tuple[int, Any]:
    request = urllib.request.Request(url, headers=headers or {}, method="GET")
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            body = response.read().decode("utf-8")
            return response.getcode(), json.loads(body)
    except urllib.error.HTTPError as error:
        body = error.read().decode("utf-8")
        try:
            parsed = json.loads(body)
        except json.JSONDecodeError:
            parsed = {"raw": body}
        return error.code, parsed


def issue_proxy_request(url: str, host: str, timeout: float) -> RequestSample:
    started = time.perf_counter()
    status, payload = fetch_json(url, headers={"Host": host}, timeout=timeout)
    latency_ms = (time.perf_counter() - started) * 1000.0
    backend = payload.get("backend", "unknown") if isinstance(payload, dict) else "unknown"
    return RequestSample(latency_ms=latency_ms, status=status, backend=backend)


def wait_for_admin(admin_url: str, timeout: float = 10.0) -> None:
    deadline = time.time() + timeout
    health_url = urllib.parse.urljoin(admin_url + "/", "admin/health")
    while time.time() < deadline:
        try:
            status, payload = fetch_json(health_url, timeout=1.0)
            if status == 200 and isinstance(payload, dict) and payload.get("status") == "ok":
                return
        except OSError:
            pass
        time.sleep(0.2)
    raise RuntimeError(f"DAO admin API did not become ready at {health_url}")


def start_backend_server(port: int, state: BackendState) -> tuple[ThreadingHTTPServer, threading.Thread]:
    server = ThreadingHTTPServer(("127.0.0.1", port), make_handler(state))
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    return server, thread


def ensure_path_exists(path: Path, label: str) -> None:
    if not path.exists():
        raise FileNotFoundError(f"{label} not found: {path}")


def spawn_dao(dao_path: Path, config_path: Path, verbose: bool) -> subprocess.Popen[str]:
    command = [str(dao_path), "--config", str(config_path)]
    if verbose:
        command.append("--verbose")

    creationflags = 0
    if os.name == "nt":
        creationflags = subprocess.CREATE_NEW_PROCESS_GROUP

    return subprocess.Popen(
        command,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        text=True,
        creationflags=creationflags,
    )


def terminate_process(process: subprocess.Popen[str]) -> None:
    if process.poll() is not None:
        return
    try:
        if os.name == "nt":
            process.send_signal(signal.CTRL_BREAK_EVENT)
        else:
            process.send_signal(signal.SIGINT)
        process.wait(timeout=5)
    except Exception:
        process.kill()
        process.wait(timeout=5)


def count_by_backend(samples: list[RequestSample]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for sample in samples:
        counts[sample.backend] = counts.get(sample.backend, 0) + 1
    return counts


def summarize(label: str, samples: list[RequestSample]) -> dict[str, Any]:
    latencies = [sample.latency_ms for sample in samples]
    statuses = {}
    for sample in samples:
        statuses[str(sample.status)] = statuses.get(str(sample.status), 0) + 1
    return {
        "label": label,
        "requests": len(samples),
        "status_counts": statuses,
        "backend_counts": count_by_backend(samples),
        "mean_ms": round(statistics.mean(latencies), 2) if latencies else 0.0,
        "p50_ms": round(percentile(latencies, 0.50), 2),
        "p95_ms": round(percentile(latencies, 0.95), 2),
    }


def print_summary(summary: dict[str, Any]) -> None:
    print(f"\n[{summary['label']}]")
    print(f"requests:      {summary['requests']}")
    print(f"status counts: {summary['status_counts']}")
    print(f"backend use:   {summary['backend_counts']}")
    print(f"mean latency:  {summary['mean_ms']} ms")
    print(f"p50 latency:   {summary['p50_ms']} ms")
    print(f"p95 latency:   {summary['p95_ms']} ms")


def query_explain(admin_url: str, host: str) -> dict[str, Any]:
    query = urllib.parse.urlencode(
        {
            "host": host,
            "path": "/v1/chat",
            "method": "GET",
            "intent": "realtime",
        }
    )
    explain_url = urllib.parse.urljoin(admin_url + "/", f"admin/explain?{query}")
    _, payload = fetch_json(explain_url, timeout=2.0)
    if not isinstance(payload, dict):
        raise RuntimeError("Unexpected /admin/explain payload")
    return payload


def main() -> int:
    parser = argparse.ArgumentParser(description="Run an end-to-end DAO benchmark with local toy backends.")
    parser.add_argument("--dao-bin", type=Path, default=DEFAULT_DAO, help="Path to the dao binary.")
    parser.add_argument("--config", type=Path, default=DEFAULT_CONFIG, help="Path to benchmark TOML config.")
    parser.add_argument("--host", default=DEFAULT_HOST, help="Host header used for routing.")
    parser.add_argument("--proxy-url", default=DEFAULT_PROXY_URL, help="DAO proxy URL.")
    parser.add_argument("--admin-url", default=DEFAULT_ADMIN_URL, help="DAO admin API base URL.")
    parser.add_argument("--steady-requests", type=int, default=120, help="Number of healthy steady-state requests.")
    parser.add_argument("--failover-requests", type=int, default=30, help="Number of requests after primary failure.")
    parser.add_argument("--warmup-requests", type=int, default=20, help="Number of warmup requests.")
    parser.add_argument("--timeout", type=float, default=3.0, help="Per-request timeout in seconds.")
    parser.add_argument("--verbose-dao", action="store_true", help="Start DAO with --verbose.")
    args = parser.parse_args()

    ensure_path_exists(args.dao_bin, "DAO binary")
    ensure_path_exists(args.config, "Benchmark config")

    primary_state = BackendState(name="fast-primary", delay_ms=12.0)
    secondary_state = BackendState(name="fallback-secondary", delay_ms=45.0)
    primary_server, _ = start_backend_server(18081, primary_state)
    secondary_server, _ = start_backend_server(18082, secondary_state)
    dao_process = spawn_dao(args.dao_bin, args.config, args.verbose_dao)

    try:
        wait_for_admin(args.admin_url)
        print("DAO is up; warming up route state...")

        for _ in range(args.warmup_requests):
            issue_proxy_request(args.proxy_url, args.host, args.timeout)

        explain_before = query_explain(args.admin_url, args.host)

        steady_samples = [
            issue_proxy_request(args.proxy_url, args.host, args.timeout)
            for _ in range(args.steady_requests)
        ]
        steady_summary = summarize("steady-state", steady_samples)
        print_summary(steady_summary)

        print("\nTriggering primary backend failure to observe failover...")
        primary_state.set_fail_mode(True)
        time.sleep(1.5)

        failover_samples = [
            issue_proxy_request(args.proxy_url, args.host, args.timeout)
            for _ in range(args.failover_requests)
        ]
        explain_after = query_explain(args.admin_url, args.host)
        failover_summary = summarize("failover", failover_samples)
        print_summary(failover_summary)

        print("\n[explain before failure]")
        print(json.dumps(
            {
                "selected": explain_before.get("selected"),
                "candidates": [
                    {
                        "name": candidate.get("name"),
                        "winner": candidate.get("winner"),
                        "circuit_open": candidate.get("circuit_open"),
                    }
                    for candidate in explain_before.get("candidates", [])
                ],
            },
            indent=2,
        ))

        print("\n[explain after failure]")
        print(json.dumps(
            {
                "selected": explain_after.get("selected"),
                "candidates": [
                    {
                        "name": candidate.get("name"),
                        "winner": candidate.get("winner"),
                        "circuit_open": candidate.get("circuit_open"),
                    }
                    for candidate in explain_after.get("candidates", [])
                ],
            },
            indent=2,
        ))

        print("\n[backend counters]")
        print(json.dumps(
            {
                "fast-primary": primary_state.snapshot(),
                "fallback-secondary": secondary_state.snapshot(),
            },
            indent=2,
        ))

        return 0
    finally:
        terminate_process(dao_process)
        primary_server.shutdown()
        secondary_server.shutdown()
        primary_server.server_close()
        secondary_server.server_close()


if __name__ == "__main__":
    sys.exit(main())
