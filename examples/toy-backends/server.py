#!/usr/bin/env python3
"""Small toy backend for the DAO_lim Docker Compose demo.

Environment variables:
- BACKEND_NAME: name returned in responses.
- DELAY_MS: artificial response delay in milliseconds.
- FAIL_MODE: when set to 1/true/yes, /health and normal requests return 503.
- PORT: bind port, default 8080.
"""

from __future__ import annotations

import json
import os
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


BACKEND_NAME = os.getenv("BACKEND_NAME", "toy-backend")
DELAY_MS = float(os.getenv("DELAY_MS", "10"))
FAIL_MODE = os.getenv("FAIL_MODE", "0").lower() in {"1", "true", "yes"}
PORT = int(os.getenv("PORT", "8080"))


class Handler(BaseHTTPRequestHandler):
    request_count = 0

    def _write_json(self, status: int, payload: dict) -> None:
        body = json.dumps(payload).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self) -> None:  # noqa: N802
        type(self).request_count += 1

        if self.path == "/health":
            self._write_json(
                503 if FAIL_MODE else 200,
                {
                    "status": "fail" if FAIL_MODE else "ok",
                    "backend": BACKEND_NAME,
                },
            )
            return

        time.sleep(DELAY_MS / 1000.0)

        self._write_json(
            503 if FAIL_MODE else 200,
            {
                "status": "error" if FAIL_MODE else "ok",
                "backend": BACKEND_NAME,
                "path": self.path,
                "request_count": type(self).request_count,
                "delay_ms": DELAY_MS,
            },
        )

    def log_message(self, format: str, *args: object) -> None:
        return


def main() -> None:
    server = ThreadingHTTPServer(("0.0.0.0", PORT), Handler)
    print(
        f"toy backend {BACKEND_NAME} listening on 0.0.0.0:{PORT} "
        f"delay_ms={DELAY_MS} fail_mode={FAIL_MODE}",
        flush=True,
    )
    server.serve_forever()


if __name__ == "__main__":
    main()
