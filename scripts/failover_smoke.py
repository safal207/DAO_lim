#!/usr/bin/env python3
"""Local DAO_lim failover smoke test.

Starts two local toy backends and a DAO process using
`configs/dao-benchmark.toml`, then verifies that traffic moves from
`fast-primary` to `fallback-secondary` after the primary starts failing.
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from e2e_benchmark import (  # noqa: E402
    DEFAULT_ADMIN_URL,
    DEFAULT_CONFIG,
    DEFAULT_DAO,
    DEFAULT_HOST,
    DEFAULT_PROXY_URL,
    BackendState,
    ensure_path_exists,
    issue_proxy_request,
    query_explain,
    spawn_dao,
    start_backend_server,
    summarize,
    terminate_process,
    wait_for_admin,
)


def candidate_by_name(explain: dict[str, Any], name: str) -> dict[str, Any] | None:
    for candidate in explain.get("candidates", []):
        if isinstance(candidate, dict) and candidate.get("name") == name:
            return candidate
    return None


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> int:
    parser = argparse.ArgumentParser(description="Run a local DAO_lim failover smoke test.")
    parser.add_argument("--dao-bin", type=Path, default=DEFAULT_DAO)
    parser.add_argument("--config", type=Path, default=DEFAULT_CONFIG)
    parser.add_argument("--host", default=DEFAULT_HOST)
    parser.add_argument("--proxy-url", default=DEFAULT_PROXY_URL)
    parser.add_argument("--admin-url", default=DEFAULT_ADMIN_URL)
    parser.add_argument("--warmup-requests", type=int, default=12)
    parser.add_argument("--steady-requests", type=int, default=20)
    parser.add_argument("--failover-requests", type=int, default=12)
    parser.add_argument("--timeout", type=float, default=3.0)
    parser.add_argument("--verbose-dao", action="store_true")
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

        for _ in range(args.warmup_requests):
            issue_proxy_request(args.proxy_url, args.host, args.timeout)

        explain_before = query_explain(args.admin_url, args.host)
        steady_samples = [
            issue_proxy_request(args.proxy_url, args.host, args.timeout)
            for _ in range(args.steady_requests)
        ]
        steady = summarize("steady-state", steady_samples)

        primary_state.set_fail_mode(True)
        time.sleep(1.5)

        failover_samples = [
            issue_proxy_request(args.proxy_url, args.host, args.timeout)
            for _ in range(args.failover_requests)
        ]
        explain_after = query_explain(args.admin_url, args.host)
        failover = summarize("failover", failover_samples)

        fast_after = candidate_by_name(explain_after, "fast-primary")
        fallback_after = candidate_by_name(explain_after, "fallback-secondary")

        require(steady["status_counts"].get("200", 0) == args.steady_requests, f"steady-state failed: {steady}")
        require(steady["backend_counts"].get("fast-primary", 0) > 0, f"fast-primary not used before failure: {steady}")
        require(failover["status_counts"].get("200", 0) == args.failover_requests, f"failover failed: {failover}")
        require(failover["backend_counts"].get("fallback-secondary", 0) > 0, f"fallback not used after failure: {failover}")
        require(explain_before.get("selected") == "fast-primary", f"unexpected before selected: {explain_before.get('selected')}")
        require(explain_after.get("selected") == "fallback-secondary", f"unexpected after selected: {explain_after.get('selected')}")
        require(fast_after is not None, "fast-primary missing from explain after failure")
        require(fallback_after is not None, "fallback-secondary missing from explain after failure")
        require(fast_after.get("circuit_open") is True, f"fast-primary circuit not open: {fast_after}")
        require(fallback_after.get("winner") is True, f"fallback-secondary not winner: {fallback_after}")

        print(json.dumps({
            "ok": True,
            "steady_state": steady,
            "failover": failover,
            "explain_before": {"selected": explain_before.get("selected")},
            "explain_after": {
                "selected": explain_after.get("selected"),
                "fast_primary": fast_after,
                "fallback_secondary": fallback_after,
            },
        }, indent=2))
        return 0
    finally:
        terminate_process(dao_process)
        primary_server.shutdown()
        secondary_server.shutdown()
        primary_server.server_close()
        secondary_server.server_close()


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:
        print(json.dumps({"ok": False, "error": str(exc)}, indent=2), file=sys.stderr)
        sys.exit(1)
