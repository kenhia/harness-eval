from datetime import UTC, datetime
from pathlib import Path

from loglens.analysis import errors, hourly, summarize, top
from loglens.parser import parse_lines

FIXTURE = Path(__file__).parent / "fixtures" / "sample.log"


def load():
    records, _ = parse_lines(FIXTURE.read_text().splitlines())
    return records


def test_summarize():
    records = load()
    s = summarize(records)
    assert s.total_requests == len(records)
    assert s.unique_ips == 5  # 203.0.113.7/.50/.99, 198.51.100.22, 192.0.2.9
    assert s.first_timestamp == datetime(2026, 7, 12, 6, 25, 24, tzinfo=UTC)
    assert s.last_timestamp == datetime(2026, 7, 12, 11, 17, 22, tzinfo=UTC)
    errs = sum(1 for r in records if r.is_error)
    assert abs(s.error_rate - errs / len(records) * 100) < 1e-9


def test_summarize_empty():
    s = summarize([])
    assert s.total_requests == 0
    assert s.unique_ips == 0
    assert s.first_timestamp is None
    assert s.last_timestamp is None
    assert s.error_rate == 0.0


def test_top_ip():
    result = top(load(), by="ip", n=3)
    assert len(result) == 3
    counts = [c for _, c in result]
    assert counts == sorted(counts, reverse=True)


def test_top_status_default_n():
    result = top(load(), by="status")
    # descending by count
    counts = [c for _, c in result]
    assert counts == sorted(counts, reverse=True)


def test_top_tie_break_value_ascending():
    # Build synthetic records with tied counts to check tie-breaking.
    line = '{ip} - - [12/Jul/2026:06:25:24 +0000] "GET /p HTTP/1.1" 200 1 "-" "UA"'
    lines = [line.format(ip="10.0.0.2"), line.format(ip="10.0.0.1")]
    records, _ = parse_lines(lines)
    result = top(records, by="ip", n=10)
    # both count 1, tie broken by value ascending
    assert result == [("10.0.0.1", 1), ("10.0.0.2", 1)]


def test_top_invalid_by():
    try:
        top(load(), by="bogus")
    except ValueError:
        pass
    else:
        raise AssertionError("expected ValueError")


def test_errors_grouping():
    result = errors(load())
    # every entry must be a 4xx/5xx status
    for status, _path, count in result:
        assert 400 <= status <= 599
        assert count >= 1
    counts = [c for _, _, c in result]
    assert counts == sorted(counts, reverse=True)
    # /missing 404 appears 4 times and should be the most frequent error
    assert result[0] == (404, "/missing", 4)


def test_errors_time_window():
    since = datetime(2026, 7, 12, 10, 0, 0, tzinfo=UTC)
    result = errors(load(), since=since)
    # only errors at/after 10:00 remain: 502 /api/login and 403 /admin
    keys = {(s, p) for s, p, _ in result}
    assert (502, "/api/login") in keys
    assert (403, "/admin") in keys
    assert (500, "/api/login") not in keys


def test_hourly():
    buckets = hourly(load())
    assert len(buckets) == 24
    assert sum(buckets) == len(load())
    # fixture has traffic in hours 6..11
    for h in range(6, 12):
        assert buckets[h] > 0
    assert buckets[0] == 0
