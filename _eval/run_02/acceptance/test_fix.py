"""Fix-round addendum (run_02.1) — F1–F3. SEALED.

Runs ONLY when FIX_ROUND=1 (run-acceptance.sh --fix); the frozen run_02
suite is unaffected by this file's presence. Frozen before the first fix
run, same rules as the main suite.
"""

import os

import pytest

from conftest import add_feed, entries, refresh

pytestmark = pytest.mark.skipif(
    not os.environ.get("FIX_ROUND"),
    reason="fix-round addendum; set FIX_ROUND=1 (run-acceptance.sh --fix)",
)


# F1 — a different truncation also errors (no special-casing the samples)
def test_f1_alternate_truncation_errors(api, feeds):
    feed = add_feed(api, feeds.url("rss_truncated_alt.xml"))
    status, result = api.post(f"/api/feeds/{feed['id']}/refresh")
    assert status == 200, f"refresh endpoint itself must not fail: {result}"
    assert result.get("status") == "error", (
        f"truncated XML must be a recorded fetch failure: {result}"
    )
    _, f = api.get(f"/api/feeds/{feed['id']}")
    assert f.get("last_error"), "last_error must describe the parse failure"
    assert entries(api, f"?feed_id={feed['id']}")["total"] == 0


# F2 — well-formed feed with zero items still refreshes ok
def test_f2_empty_valid_feed_is_ok(api, feeds):
    feed = add_feed(api, feeds.url("rss_empty_valid.xml"))
    result = refresh(api, feed["id"])
    assert result.get("status") == "ok", (
        f"zero items in well-formed XML is not an error: {result}"
    )
    assert result.get("new_entries") == 0
    _, f = api.get(f"/api/feeds/{feed['id']}")
    assert f.get("last_error") in (None, ""), (
        f"empty-but-valid feed must not record an error: {f}"
    )


# F3 — last_error clears after a subsequent successful fetch
def test_f3_error_clears_on_recovery(api, feeds):
    feeds.set("fix_flip.xml", feeds.httpd.store["rss_truncated_alt.xml"])
    feed = add_feed(api, feeds.url("fix_flip.xml"))
    status, result = api.post(f"/api/feeds/{feed['id']}/refresh")
    assert result.get("status") == "error", f"setup: expected error first: {result}"
    feeds.set("fix_flip.xml", feeds.httpd.store["rss_basic.xml"])
    result = refresh(api, feed["id"])
    assert result.get("status") == "ok"
    assert result.get("new_entries") == 3
    _, f = api.get(f"/api/feeds/{feed['id']}")
    assert f.get("last_error") in (None, ""), (
        f"recovered feed must clear last_error: {f}"
    )
