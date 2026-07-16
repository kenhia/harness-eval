"""Acceptance harness for run_02 (feedhub). SEALED — never shown to agents.

Run against one contender repo:

    FEEDHUB_REPO=~/src/ai-agents/harness-eval-runs/run_02/05-baseline \
        uv run --with pytest pytest _eval/run_02/acceptance -v

Black-box: builds the repo's release binaries, serves the sealed fixture
corpus from an in-process HTTP server (with ETag/304 support and a request
log), and drives feedd/feedctl as subprocesses. The contender's own
feedgen is exercised separately in test_core.
"""

import hashlib
import http.server
import json
import os
import socket
import subprocess
import threading
import time
import urllib.error
import urllib.request
from pathlib import Path

import pytest

from fixtures import CORPUS

REPO = Path(os.environ.get("FEEDHUB_REPO", "")).expanduser()
BUILD_TIMEOUT = 900
GATE_TIMEOUT = 900


def free_port() -> int:
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


# ---------------------------------------------------------------- fixtures server
class FeedHandler(http.server.BaseHTTPRequestHandler):
    server_version = "AcceptanceFixtures/1.0"

    def do_GET(self):  # noqa: N802
        name = self.path.lstrip("/")
        store = self.server.store  # type: ignore[attr-defined]
        # Header names lowercased: HTTP headers are case-insensitive and
        # clients differ (S1 — a case-sensitive lookup failed 06 unfairly).
        self.server.request_log.append(  # type: ignore[attr-defined]
            {"path": self.path, "headers": {k.lower(): v for k, v in self.headers.items()}}
        )
        if name not in store:
            self.send_response(404)
            self.end_headers()
            return
        body = store[name]
        etag = '"' + hashlib.md5(body).hexdigest() + '"'
        if self.headers.get("If-None-Match") == etag:
            self.send_response(304)
            self.send_header("ETag", etag)
            self.end_headers()
            return
        self.send_response(200)
        self.send_header("Content-Type", "application/xml; charset=utf-8")
        self.send_header("ETag", etag)
        self.send_header("Last-Modified", "Fri, 10 Jul 2026 00:00:00 GMT")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *_):  # quiet
        pass


class FixtureServer:
    def __init__(self):
        self.port = free_port()
        self.httpd = http.server.ThreadingHTTPServer(("127.0.0.1", self.port), FeedHandler)
        self.httpd.store = dict(CORPUS)  # type: ignore[attr-defined]
        self.httpd.request_log = []  # type: ignore[attr-defined]
        self.thread = threading.Thread(target=self.httpd.serve_forever, daemon=True)
        self.thread.start()

    def url(self, name: str) -> str:
        return f"http://127.0.0.1:{self.port}/{name}"

    def set(self, name: str, body: bytes | None) -> None:
        if body is None:
            self.httpd.store.pop(name, None)  # type: ignore[attr-defined]
        else:
            self.httpd.store[name] = body  # type: ignore[attr-defined]

    def requests_for(self, name: str) -> list[dict]:
        return [r for r in self.httpd.request_log if r["path"].lstrip("/") == name]  # type: ignore[attr-defined]

    def stop(self):
        self.httpd.shutdown()


# ---------------------------------------------------------------- build
@pytest.fixture(scope="session")
def binaries() -> dict[str, Path]:
    assert REPO.is_dir(), f"FEEDHUB_REPO not set or not a directory: {REPO!r}"
    proc = subprocess.run(
        ["cargo", "build", "--release"],
        cwd=REPO, capture_output=True, text=True, timeout=BUILD_TIMEOUT,
    )
    assert proc.returncode == 0, f"cargo build --release failed:\n{proc.stderr[-4000:]}"
    bins = {}
    for name in ("feedd", "feedctl", "feedgen"):
        p = REPO / "target" / "release" / name
        assert p.is_file(), f"missing binary after build: {p}"
        bins[name] = p
    return bins


# ---------------------------------------------------------------- feedd + client
class Api:
    def __init__(self, base: str):
        self.base = base

    def request(self, method: str, path: str, body: dict | None = None):
        req = urllib.request.Request(self.base + path, method=method)
        data = None
        if body is not None:
            data = json.dumps(body).encode()
            req.add_header("Content-Type", "application/json")
        try:
            with urllib.request.urlopen(req, data=data, timeout=10) as resp:
                raw = resp.read()
                return resp.status, json.loads(raw) if raw.strip() else None
        except urllib.error.HTTPError as e:
            raw = e.read()
            try:
                parsed = json.loads(raw) if raw.strip() else None
            except json.JSONDecodeError:
                parsed = {"_raw": raw.decode(errors="replace")}
            return e.code, parsed

    def get(self, path):
        return self.request("GET", path)

    def post(self, path, body=None):
        return self.request("POST", path, body or {})

    def delete(self, path):
        return self.request("DELETE", path)


class Feedd:
    def __init__(self, binaries, tmp: Path):
        self.port = free_port()
        self.db = tmp / "feedhub.db"
        self.proc = subprocess.Popen(
            [binaries["feedd"], "--db", str(self.db),
             "--listen", f"127.0.0.1:{self.port}", "--poll-interval", "0"],
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, cwd=tmp,
        )
        self.api = Api(f"http://127.0.0.1:{self.port}")
        deadline = time.time() + 20
        last = None
        while time.time() < deadline:
            if self.proc.poll() is not None:
                out, err = self.proc.communicate()
                raise AssertionError(
                    f"feedd exited early (rc={self.proc.returncode})\n"
                    f"stdout: {out[-2000:]}\nstderr: {err[-2000:]}"
                )
            try:
                status, _ = self.api.get("/api/health")
                if status == 200:
                    return
                last = status
            except OSError as e:
                last = e
            time.sleep(0.2)
        raise AssertionError(f"feedd never became healthy on :{self.port} (last: {last})")

    def stop(self):
        self.proc.terminate()
        try:
            self.proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.proc.kill()


@pytest.fixture(scope="session")
def feeds():
    server = FixtureServer()
    yield server
    server.stop()


@pytest.fixture
def feedd(binaries, feeds, tmp_path):
    d = Feedd(binaries, tmp_path)
    yield d
    d.stop()


@pytest.fixture
def api(feedd):
    return feedd.api


@pytest.fixture
def feedctl(binaries, feedd):
    def run(*args: str, fmt: str | None = None, server: str | None = None):
        cmd = [str(binaries["feedctl"]), "--server", server or feedd.api.base]
        if fmt:
            cmd += ["--format", fmt]
        cmd += list(args)
        return subprocess.run(cmd, capture_output=True, text=True, timeout=30)

    return run


# ---------------------------------------------------------------- helpers
def add_feed(api: Api, url: str) -> dict:
    status, feed = api.post("/api/feeds", {"url": url})
    assert status == 201, f"POST /api/feeds -> {status}: {feed}"
    assert "id" in feed, f"feed object missing id: {feed}"
    return feed


def refresh(api: Api, feed_id) -> dict:
    status, result = api.post(f"/api/feeds/{feed_id}/refresh")
    assert status == 200, f"refresh -> {status}: {result}"
    return result


def entries(api: Api, query: str = "") -> dict:
    status, body = api.get(f"/api/entries{query}")
    assert status == 200, f"GET /api/entries{query} -> {status}: {body}"
    assert isinstance(body, dict) and "total" in body and "items" in body, (
        f"entries response must be {{total, items}}: {body}"
    )
    return body


def utc_instant(value):
    """Parse an RFC 3339 timestamp and require an explicit UTC-equivalent instant."""
    from datetime import datetime, timezone

    assert value, f"expected a timestamp, got {value!r}"
    dt = datetime.fromisoformat(value.replace("Z", "+00:00"))
    assert dt.tzinfo is not None, f"timestamp lacks offset: {value}"
    return dt.astimezone(timezone.utc)
