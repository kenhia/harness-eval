"""Tests for analysis functions."""

from __future__ import annotations

from datetime import UTC

from loglens import analyze
from loglens.parser import parse_lines


def load(sample_log):
    return parse_lines(sample_log.read_text().splitlines()).records


def test_summary(sample_log):
    s = analyze.summary(load(sample_log))
    assert s["total_requests"] == 32
    assert s["unique_ips"] == 8
    assert s["first_timestamp"] == "2026-07-12T06:25:24+00:00"
    assert s["last_timestamp"] == "2026-07-12T15:40:10+00:00"
    assert s["error_count"] == 10
    assert s["error_rate"] == 31.25


def test_top_ip_ties_broken_by_value_ascending(sample_log):
    rows = analyze.top(load(sample_log), "ip", 2)
    assert rows[0] == {"value": "198.51.100.22", "count": 5}
    assert rows[1] == {"value": "203.0.113.7", "count": 5}


def test_top_path(sample_log):
    rows = analyze.top(load(sample_log), "path", 1)
    assert rows[0] == {"value": "/index.html", "count": 4}


def test_top_status(sample_log):
    rows = analyze.top(load(sample_log), "status", 1)
    assert rows[0] == {"value": "200", "count": 16}


def test_top_default_limit(sample_log):
    rows = analyze.top(load(sample_log), "path", 10)
    assert len(rows) == 10


def test_errors_grouped_and_ordered(sample_log):
    rows = analyze.errors(load(sample_log))
    assert rows[0] == {"status": 404, "path": "/missing", "count": 2}
    assert rows[1] == {"status": 500, "path": "/api/status", "count": 2}
    total = sum(r["count"] for r in rows)
    assert total == 10


def test_errors_since_until_filtering(sample_log):
    from datetime import datetime

    since = datetime(2026, 7, 12, 9, 0, 0, tzinfo=UTC)
    until = datetime(2026, 7, 12, 9, 59, 59, tzinfo=UTC)
    rows = analyze.errors(load(sample_log), since=since, until=until)
    total = sum(r["count"] for r in rows)
    assert total == 3  # two 500s and one 503 in hour 09


def test_hourly(sample_log):
    buckets = analyze.hourly(load(sample_log))
    assert len(buckets) == 24
    assert buckets[9] == 6
    assert buckets[7] == 4
    assert buckets[15] == 1
    assert sum(buckets) == 32
    assert buckets[0] == 0
