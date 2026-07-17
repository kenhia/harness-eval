"""Core tier — every competent run should pass all of these (C1–C12).

C1 (workspace builds, binaries exist) is enforced by the `binaries`
session fixture; every test here depends on it transitively.
"""

import json
import subprocess
import time
import urllib.error
import urllib.request

from conftest import GATE_TIMEOUT, REPO, add_feed, entries, free_port, refresh, utc_instant


# C2 — server starts, health endpoint answers
def test_c2_health(api):
    status, body = api.get("/api/health")
    assert status == 200
    assert body.get("status") == "ok"


# C3 — feed registration: 201, duplicate 409, invalid 422
def test_c3_add_duplicate_invalid(api, feeds):
    feed = add_feed(api, feeds.url("rss_basic.xml"))
    assert feed["url"] == feeds.url("rss_basic.xml")
    status, _ = api.post("/api/feeds", {"url": feeds.url("rss_basic.xml")})
    assert status == 409, "duplicate URL must 409"
    status, _ = api.post("/api/feeds", {"url": "not a url"})
    assert status == 422, "invalid URL must 422"


# C4 — refresh ingests RSS 2.0; entry fields and UTC dates
def test_c4_rss_ingest(api, feeds):
    feed = add_feed(api, feeds.url("rss_basic.xml"))
    result = refresh(api, feed["id"])
    assert result.get("status") == "ok"
    assert result.get("new_entries") == 3
    body = entries(api, f"?feed_id={feed['id']}")
    assert body["total"] == 3
    items = body["items"]
    for field in ("id", "feed_id", "guid", "title", "link", "summary", "published_at", "fetched_at"):
        assert field in items[0], f"entry object missing {field}"
    # newest first, instants normalized to UTC
    got = [utc_instant(i["published_at"]).strftime("%Y-%m-%dT%H:%M:%SZ") for i in items]
    assert got == ["2026-07-10T10:00:00Z", "2026-07-10T09:00:00Z", "2026-07-10T08:00:00Z"]
    assert items[0]["title"] == "First post"
    # feed metadata updated
    status, f = api.get(f"/api/feeds/{feed['id']}")
    assert status == 200
    assert f.get("title") == "RSS Basic"
    assert f.get("entry_count") == 3
    assert f.get("last_error") in (None, "")


# C5 — refresh ingests Atom
def test_c5_atom_ingest(api, feeds):
    feed = add_feed(api, feeds.url("atom_basic.xml"))
    result = refresh(api, feed["id"])
    assert result.get("new_entries") == 3
    items = entries(api, f"?feed_id={feed['id']}")["items"]
    assert [i["title"] for i in items] == ["Alpha", "Beta", "Gamma"]
    assert items[0]["link"].endswith("/a1")


# C6 — refetch does not duplicate
def test_c6_dedupe(api, feeds):
    feed = add_feed(api, feeds.url("rss_basic.xml"))
    refresh(api, feed["id"])
    second = refresh(api, feed["id"])
    assert second.get("new_entries") == 0
    assert entries(api, f"?feed_id={feed['id']}")["total"] == 3


# C7 — entries query: feed_id filter, limit, total
def test_c7_entries_query(api, feeds):
    f1 = add_feed(api, feeds.url("rss_basic.xml"))
    f2 = add_feed(api, feeds.url("atom_basic.xml"))
    refresh(api, f1["id"])
    refresh(api, f2["id"])
    assert entries(api)["total"] == 6
    body = entries(api, f"?feed_id={f1['id']}&limit=2")
    assert body["total"] == 3, "total must ignore limit"
    assert len(body["items"]) == 2


# C8 — delete cascades and 404s afterward
def test_c8_delete(api, feeds):
    feed = add_feed(api, feeds.url("rss_basic.xml"))
    refresh(api, feed["id"])
    status, _ = api.delete(f"/api/feeds/{feed['id']}")
    assert status == 204
    status, _ = api.get(f"/api/feeds/{feed['id']}")
    assert status == 404
    assert entries(api)["total"] == 0, "entries must be deleted with the feed"


# C9 — malformed feed: recorded error, no crash, siblings unaffected
def test_c9_failure_isolation(api, feeds):
    good = add_feed(api, feeds.url("rss_broken_sibling.xml"))
    bad = add_feed(api, feeds.url("malformed.xml"))
    status, results = api.post("/api/refresh")
    assert status == 200
    assert isinstance(results, list) and len(results) == 2
    by_ok = {r.get("status") for r in results}
    assert by_ok == {"ok", "error"}, f"expected one ok + one error: {results}"
    status, b = api.get(f"/api/feeds/{bad['id']}")
    assert b.get("last_error"), "failed fetch must record last_error"
    assert entries(api, f"?feed_id={good['id']}")["total"] == 1
    status, _ = api.get("/api/health")
    assert status == 200, "feedd must survive a malformed feed"


# C10 — feedctl drives the API; text and json formats
def test_c10_feedctl_basics(api, feeds, feedctl):
    r = feedctl("add", feeds.url("rss_basic.xml"))
    assert r.returncode == 0, r.stderr
    r = feedctl("refresh")
    assert r.returncode == 0, r.stderr
    r = feedctl("list")
    assert r.returncode == 0 and "RSS Basic" in r.stdout
    r = feedctl("entries", fmt="json")
    assert r.returncode == 0, r.stderr
    body = json.loads(r.stdout)
    assert body["total"] == 3 and len(body["items"]) == 3
    r = feedctl("entries")
    assert r.returncode == 0 and "First post" in r.stdout


# C11 — feedctl exit codes: 1 on API error, 2 on unreachable
def test_c11_feedctl_exit_codes(feedctl):
    r = feedctl("remove", "999999")
    assert r.returncode == 1, f"API error must exit 1 (got {r.returncode}); stderr: {r.stderr}"
    assert r.stderr.strip(), "error message belongs on stderr"
    r = feedctl("list", server="http://127.0.0.1:9")  # port 9 = discard; nothing listens
    assert r.returncode == 2, f"unreachable server must exit 2 (got {r.returncode})"


# C12 — the repo's own gates hold
def _gate(args):
    return subprocess.run(args, cwd=REPO, capture_output=True, text=True, timeout=GATE_TIMEOUT)


def test_c12a_cargo_test(binaries):
    r = _gate(["cargo", "test"])
    assert r.returncode == 0, f"cargo test failed:\n{r.stdout[-2000:]}\n{r.stderr[-2000:]}"


def test_c12b_fmt(binaries):
    r = _gate(["cargo", "fmt", "--check"])
    assert r.returncode == 0, f"cargo fmt --check failed:\n{r.stdout[-2000:]}"


def test_c12c_clippy(binaries):
    r = _gate(["cargo", "clippy", "--all-targets", "--", "-D", "warnings"])
    assert r.returncode == 0, f"clippy failed:\n{r.stderr[-3000:]}"


# C13 — the contender's feedgen serves files with ETag/304
def test_c13_feedgen_serves(binaries, tmp_path):
    fdir = tmp_path / "fixtures"
    fdir.mkdir()
    (fdir / "feed.xml").write_bytes(b"<rss version='2.0'><channel><title>t</title></channel></rss>")
    port = free_port()
    proc = subprocess.Popen(
        [binaries["feedgen"], "serve", "--dir", str(fdir), "--listen", f"127.0.0.1:{port}"],
        stdout=subprocess.PIPE, stderr=subprocess.PIPE,
    )
    try:
        url = f"http://127.0.0.1:{port}/feed.xml"
        deadline = time.time() + 15
        resp = None
        while time.time() < deadline:
            try:
                resp = urllib.request.urlopen(url, timeout=5)
                break
            except OSError:
                if proc.poll() is not None:
                    _, err = proc.communicate()
                    raise AssertionError(f"feedgen exited early: {err[-1500:]}")
                time.sleep(0.2)
        assert resp is not None and resp.status == 200
        etag = resp.headers.get("ETag")
        assert etag, "feedgen must send an ETag"
        req = urllib.request.Request(url, headers={"If-None-Match": etag})
        try:
            urllib.request.urlopen(req, timeout=5)
            raise AssertionError("expected 304 for matching If-None-Match")
        except urllib.error.HTTPError as e:
            assert e.code == 304
    finally:
        proc.terminate()
