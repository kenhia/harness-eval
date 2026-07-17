"""Tests for analysis aggregations."""

from datetime import UTC, datetime

from loglens import analyze
from loglens.parser import iter_records

SAMPLE = [
    '10.0.0.1 - - [12/Jul/2026:06:00:00 +0000] "GET /a HTTP/1.1" 200 10 "-" "UA"',
    '10.0.0.1 - - [12/Jul/2026:06:30:00 +0000] "GET /a HTTP/1.1" 200 10 "-" "UA"',
    '10.0.0.2 - - [12/Jul/2026:07:00:00 +0000] "GET /b HTTP/1.1" 404 10 "-" "UA"',
    '10.0.0.2 - - [12/Jul/2026:07:30:00 +0000] "GET /b HTTP/1.1" 500 10 "-" "UA"',
    '10.0.0.3 - - [12/Jul/2026:08:00:00 +0000] "GET /a HTTP/1.1" 404 10 "-" "UA"',
]


def records():
    return list(iter_records(SAMPLE))


def test_summary():
    data = analyze.summary(records())
    assert data["total_requests"] == 5
    assert data["unique_ips"] == 3
    assert data["error_count"] == 3
    assert data["error_rate"] == 60.0
    assert data["first_timestamp"] == datetime(2026, 7, 12, 6, 0, tzinfo=UTC)
    assert data["last_timestamp"] == datetime(2026, 7, 12, 8, 0, tzinfo=UTC)


def test_top_by_path():
    rows = analyze.top(records(), by="path")
    assert rows[0] == ("/a", 3)
    assert rows[1] == ("/b", 2)


def test_top_by_ip_tie_break_ascending():
    rows = analyze.top(records(), by="ip")
    # 10.0.0.1 and 10.0.0.2 both have 2; tie broken by value ascending.
    assert rows[0] == ("10.0.0.1", 2)
    assert rows[1] == ("10.0.0.2", 2)
    assert rows[2] == ("10.0.0.3", 1)


def test_top_limit():
    rows = analyze.top(records(), by="ip", n=1)
    assert len(rows) == 1


def test_top_by_status():
    rows = analyze.top(records(), by="status")
    assert ("404", 2) in rows
    assert ("200", 2) in rows
    assert ("500", 1) in rows


def test_errors_grouping():
    rows = analyze.errors(records())
    assert (404, "/b", 1) in rows
    assert (500, "/b", 1) in rows
    assert (404, "/a", 1) in rows
    # only errors, no 200s
    assert all(status >= 400 for status, _, _ in rows)


def test_errors_since_until():
    since = datetime(2026, 7, 12, 7, 15, tzinfo=UTC)
    rows = analyze.errors(records(), since=since)
    paths = {(s, p) for s, p, _ in rows}
    assert (500, "/b") in paths
    assert (404, "/a") in paths
    assert (404, "/b") not in paths  # at 07:00, before since


def test_errors_since_naive():
    # naive since is treated as UTC
    since = datetime(2026, 7, 12, 8, 0)
    rows = analyze.errors(records(), since=since)
    assert rows == [(404, "/a", 1)]


def test_hourly():
    buckets = analyze.hourly(records())
    assert buckets[6] == 2
    assert buckets[7] == 2
    assert buckets[8] == 1
    assert sum(buckets) == 5
