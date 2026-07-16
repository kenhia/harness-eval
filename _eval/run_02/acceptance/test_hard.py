"""Hard tier (H1–H12) — adversarial/edge probes designed to spread the
field. Failures here are expected and scored as a count, not a gate.
Every probed semantic is pinned in the spec — nothing here is a judgment
call.
"""

from datetime import datetime, timezone

from conftest import add_feed, entries, refresh, utc_instant


def ts(items, guid):
    row = next(i for i in items if i["guid"].endswith(guid))
    return utc_instant(row["published_at"]) if row["published_at"] else None


def row(items, guid):
    return next(i for i in items if i["guid"].endswith(guid))


# H1 — RFC 822 named zones normalize to the right UTC instants
def test_h1_rfc822_named_zones(api, feeds):
    feed = add_feed(api, feeds.url("rss_zones.xml"))
    refresh(api, feed["id"])
    items = entries(api, f"?feed_id={feed['id']}")["items"]
    assert ts(items, "z1") == datetime(2026, 7, 10, 12, 0, tzinfo=timezone.utc), "EST is UTC-5"
    assert ts(items, "z2") == datetime(2026, 7, 10, 6, 0, tzinfo=timezone.utc)


# H2 — missing date: null published_at, stored, sorted last
def test_h2_missing_date(api, feeds):
    feed = add_feed(api, feeds.url("rss_zones.xml"))
    refresh(api, feed["id"])
    body = entries(api, f"?feed_id={feed['id']}")
    assert body["total"] == 4, "dateless entries are still stored"
    assert row(body["items"], "z3")["published_at"] is None
    nulls_positions = [n for n, i in enumerate(body["items"]) if i["published_at"] is None]
    assert nulls_positions == [2, 3], f"null dates must sort last, got order {body['items']}"


# H3 — garbage date: null, not fetch time, no crash
def test_h3_garbage_date(api, feeds):
    feed = add_feed(api, feeds.url("rss_zones.xml"))
    refresh(api, feed["id"])
    items = entries(api, f"?feed_id={feed['id']}")["items"]
    assert row(items, "z4")["published_at"] is None, (
        "unparseable dates must store null — never fetch time"
    )


# H4 — RFC 3339 offsets and fractional seconds normalize to UTC
def test_h4_atom_offsets(api, feeds):
    feed = add_feed(api, feeds.url("atom_offsets.xml"))
    refresh(api, feed["id"])
    items = entries(api, f"?feed_id={feed['id']}")["items"]
    assert ts(items, "o1") == datetime(2026, 7, 11, 10, 30, tzinfo=timezone.utc), "+02:00 -> UTC"
    frac = ts(items, "o2")
    assert frac is not None and frac.replace(microsecond=0) == datetime(
        2026, 7, 11, 5, 0, tzinfo=timezone.utc
    ), "fractional seconds must be accepted"


# H5 — Atom entry with only <updated> falls back to it
def test_h5_atom_updated_fallback(api, feeds):
    feed = add_feed(api, feeds.url("atom_offsets.xml"))
    refresh(api, feed["id"])
    items = entries(api, f"?feed_id={feed['id']}")["items"]
    assert ts(items, "o3") == datetime(2026, 7, 11, 4, 0, tzinfo=timezone.utc)


# H6 — half-open window: since inclusive, until exclusive
def test_h6_half_open_window(api, feeds):
    feed = add_feed(api, feeds.url("rss_boundary.xml"))
    refresh(api, feed["id"])
    q = f"?feed_id={feed['id']}&since=2026-07-11T00:00:00Z&until=2026-07-12T00:00:00Z"
    body = entries(api, q)
    guids = sorted(i["guid"][-2:] for i in body["items"])
    assert guids == ["b2", "b3"], (
        f"[since, until): b3 (== since) in, b1 (== until) out — got {guids}"
    )
    # offset-form bound, same instants
    q2 = f"?feed_id={feed['id']}&since=2026-07-11T02:00:00%2B02:00&until=2026-07-12T00:00:00Z"
    assert sorted(i["guid"][-2:] for i in entries(api, q2)["items"]) == ["b2", "b3"]


# H7 — BOM tolerated; CDATA verbatim; entities unescaped
def test_h7_bom_cdata_entities(api, feeds):
    feed = add_feed(api, feeds.url("rss_bom_cdata.xml"))
    result = refresh(api, feed["id"])
    assert result.get("status") == "ok", f"BOM feed must parse: {result}"
    items = entries(api, f"?feed_id={feed['id']}")["items"]
    assert row(items, "c1")["title"] == "Ben & Jerry <3"
    assert row(items, "c2")["title"] == "Fish & Chips"


# H8 — guid reuse updates in place: no dup, id and fetched_at stable
def test_h8_update_in_place(api, feeds):
    feeds.set("rss_update_live.xml", feeds.httpd.store["rss_update_v1.xml"])
    feed = add_feed(api, feeds.url("rss_update_live.xml"))
    refresh(api, feed["id"])
    before = entries(api, f"?feed_id={feed['id']}")["items"]
    u1_before = row(before, "u1")
    feeds.set("rss_update_live.xml", feeds.httpd.store["rss_update_v2.xml"])
    result = refresh(api, feed["id"])
    assert result.get("new_entries") == 1, "only u2 is new"
    after = entries(api, f"?feed_id={feed['id']}")
    assert after["total"] == 2, "u1 must not duplicate"
    u1_after = row(after["items"], "u1")
    assert u1_after["title"] == "Revised title"
    assert u1_after["id"] == u1_before["id"], "updated entry keeps its internal id"
    assert u1_after["fetched_at"] == u1_before["fetched_at"], "updated entry keeps first fetched_at"


# H9 — conditional GET: If-None-Match sent, 304 handled cleanly
def test_h9_conditional_get(api, feeds):
    feed = add_feed(api, feeds.url("rss_basic.xml"))
    refresh(api, feed["id"])
    n_before = len(feeds.requests_for("rss_basic.xml"))
    result = refresh(api, feed["id"])
    reqs = feeds.requests_for("rss_basic.xml")
    assert len(reqs) > n_before, "second refresh must actually hit the server"
    assert reqs[-1]["headers"].get("If-None-Match"), "refetch must send If-None-Match"
    assert result.get("status") == "ok", "304 is a successful fetch"
    assert result.get("new_entries") == 0
    assert entries(api, f"?feed_id={feed['id']}")["total"] == 3


# H10 — pagination math: total, offset windows, ordering stability
def test_h10_pagination(api, feeds):
    feed = add_feed(api, feeds.url("rss_paging.xml"))
    refresh(api, feed["id"])
    body = entries(api, f"?feed_id={feed['id']}&limit=10")
    assert body["total"] == 24 and len(body["items"]) == 10
    page2 = entries(api, f"?feed_id={feed['id']}&limit=10&offset=10")
    assert page2["total"] == 24 and len(page2["items"]) == 10
    page3 = entries(api, f"?feed_id={feed['id']}&limit=10&offset=20")
    assert len(page3["items"]) == 4
    all_guids = [i["guid"] for i in body["items"] + page2["items"] + page3["items"]]
    assert len(set(all_guids)) == 24, "pages must not overlap or skip"
    hours = [utc_instant(i["published_at"]).hour for i in body["items"]]
    assert hours == sorted(hours, reverse=True), "newest first across pages"


# H11 — q search is case-insensitive SUBSTRING on title (pinned): "rust"
# matches C-rust-acean too — word-boundary or fuzzy matching both fail this.
def test_h11_search(api, feeds):
    feed = add_feed(api, feeds.url("rss_search.xml"))
    refresh(api, feed["id"])
    body = entries(api, f"?feed_id={feed['id']}&q=rust")
    titles = sorted(i["title"] for i in body["items"])
    assert titles == ["Crustacean recipes", "RUSTACEAN news", "Rust Weekly"], f"got {titles}"
    assert body["total"] == 3


# H12 — refresh-all: one broken feed, siblings still update, per-feed results
def test_h12_refresh_all_isolation(api, feeds, feedctl):
    good1 = add_feed(api, feeds.url("rss_basic.xml"))
    add_feed(api, feeds.url("malformed.xml"))
    good2 = add_feed(api, feeds.url("atom_basic.xml"))
    status, results = api.post("/api/refresh")
    assert status == 200 and len(results) == 3
    assert sum(1 for r in results if r.get("status") == "ok") == 2
    assert sum(1 for r in results if r.get("status") == "error") == 1
    assert entries(api, f"?feed_id={good1['id']}")["total"] == 3
    assert entries(api, f"?feed_id={good2['id']}")["total"] == 3
    r = feedctl("refresh")
    assert r.returncode in (0, 1), "refresh-all with a broken feed must not crash feedctl"
