"""Tests for the aggregation logic."""

from __future__ import annotations

from datetime import datetime
from pathlib import Path

from loglens import analyze
from loglens.parser import parse_lines


def _records(path: Path):
    with open(path, encoding="utf-8") as fh:
        records, _ = parse_lines(fh)
    return records


def test_summary(sample_log: Path) -> None:
    records = _records(sample_log)
    data = analyze.summary(records)
    assert data["total_requests"] == 32
    assert data["unique_ips"] == 4
    assert data["first_timestamp"] == "2026-07-12T06:25:24+00:00"
    assert data["last_timestamp"] == "2026-07-12T18:01:49+00:00"
    # 12 error responses out of 32 requests.
    assert data["error_rate"] == round(12 / 32 * 100, 2)


def test_top_path(sample_log: Path) -> None:
    records = _records(sample_log)
    top = analyze.top(records, "path", n=3)
    assert top[0]["value"] == "/index.html"
    assert top[0]["count"] == 8
    counts = [row["count"] for row in top]
    assert counts == sorted(counts, reverse=True)


def test_top_tie_break_ascending() -> None:
    from loglens.parser import parse_line

    base = '{ip} - - [12/Jul/2026:06:00:00 +0000] "GET /a HTTP/1.1" 200 1 "-" "-"'
    records = [parse_line(base.format(ip=ip)) for ip in ["10.0.0.2", "10.0.0.1"]]
    records = [r for r in records if r is not None]
    top = analyze.top(records, "ip", n=2)
    # Both have count 1, so ties break by value ascending.
    assert [row["value"] for row in top] == ["10.0.0.1", "10.0.0.2"]


def test_top_respects_n(sample_log: Path) -> None:
    records = _records(sample_log)
    assert len(analyze.top(records, "status", n=2)) == 2


def test_errors_grouping(sample_log: Path) -> None:
    records = _records(sample_log)
    errs = analyze.errors(records)
    assert all(400 <= row["status"] <= 599 for row in errs)
    # 404 /missing appears 3 times and should be the most frequent.
    assert errs[0]["status"] == 404
    assert errs[0]["path"] == "/missing"
    assert errs[0]["count"] == 3
    total = sum(row["count"] for row in errs)
    assert total == 12


def test_errors_since_until(sample_log: Path) -> None:
    records = _records(sample_log)
    since = datetime.fromisoformat("2026-07-12T14:00:00+00:00")
    until = datetime.fromisoformat("2026-07-12T18:00:00+00:00")
    errs = analyze.errors(records, since=since, until=until)
    for row in errs:
        assert row["count"] >= 1
    total = sum(row["count"] for row in errs)
    # Errors in [14:00, 18:00]: 404 favicon, 500 products, 401 admin, 404 missing.
    assert total == 4


def test_hourly(sample_log: Path) -> None:
    records = _records(sample_log)
    buckets = analyze.hourly(records)
    assert len(buckets) == 24
    assert sum(buckets) == 32
    assert buckets[6] == 1
    assert buckets[0] == 0
