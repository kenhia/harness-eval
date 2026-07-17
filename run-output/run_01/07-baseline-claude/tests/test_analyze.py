from datetime import UTC, datetime

from loglens.analyze import errors, filter_time, hourly, summarize, top
from loglens.parser import parse_lines


def make(ip="203.0.113.7", path="/", status=200, hour=6, minute=0):
    line = (
        f"{ip} - - [12/Jul/2026:{hour:02d}:{minute:02d}:00 +0000] "
        f'"GET {path} HTTP/1.1" {status} 100 "-" "-"'
    )
    return parse_lines([line]).entries[0]


def test_summarize_counts_and_bounds():
    entries = [
        make(hour=6),
        make(ip="192.0.2.9", hour=9, status=404),
        make(ip="192.0.2.9", hour=7, status=500),
        make(hour=8, status=301),
    ]
    summary = summarize(entries)
    assert summary.total == 4
    assert summary.unique_ips == 2
    assert summary.first == datetime(2026, 7, 12, 6, 0, tzinfo=UTC)
    assert summary.last == datetime(2026, 7, 12, 9, 0, tzinfo=UTC)
    assert summary.errors == 2
    assert summary.error_rate == 50.0


def test_summarize_3xx_is_not_an_error():
    assert summarize([make(status=301), make(status=200)]).error_rate == 0.0


def test_summarize_sample(sample_entries):
    summary = summarize(sample_entries)
    assert summary.total == 32
    assert summary.unique_ips == 5
    assert summary.errors == 10
    assert summary.error_rate == 31.25
    assert summary.first == datetime(2026, 7, 12, 6, 25, 24, tzinfo=UTC)
    assert summary.last == datetime(2026, 7, 12, 23, 58, 47, tzinfo=UTC)


def test_top_orders_by_count_descending(sample_entries):
    assert top(sample_entries, "path", 3) == [
        ("/api/orders", 8),
        ("/index.html", 7),
        ("/missing", 4),
    ]


def test_top_breaks_ties_by_value_ascending():
    entries = [make(path="/b"), make(path="/b"), make(path="/a"), make(path="/c")]
    assert top(entries, "path") == [("/b", 2), ("/a", 1), ("/c", 1)]


def test_top_defaults_to_ten():
    entries = [make(path=f"/p{i}") for i in range(15)]
    assert len(top(entries, "path")) == 10


def test_top_n_larger_than_data_returns_all():
    assert len(top([make(path="/a")], "path", 50)) == 1


def test_top_by_status_and_ip(sample_entries):
    assert top(sample_entries, "status", 2) == [("200", 15), ("404", 4)]
    assert top(sample_entries, "ip", 1) == [("203.0.113.7", 8)]


def test_errors_only_includes_4xx_5xx(sample_entries):
    rows = errors(sample_entries)
    assert all(status >= 400 for status, _, _ in rows)
    assert sum(count for _, _, count in rows) == 10


def test_errors_grouped_most_frequent_first(sample_entries):
    assert errors(sample_entries)[0] == (404, "/missing", 4)


def test_errors_ties_broken_by_status_then_path():
    entries = [
        make(status=500, path="/b"),
        make(status=404, path="/z"),
        make(status=404, path="/a"),
    ]
    assert errors(entries) == [(404, "/a", 1), (404, "/z", 1), (500, "/b", 1)]


def test_errors_since_and_until_are_inclusive():
    entries = [make(status=404, hour=h) for h in (6, 7, 8)]
    window = errors(
        entries,
        since=datetime(2026, 7, 12, 7, 0, tzinfo=UTC),
        until=datetime(2026, 7, 12, 8, 0, tzinfo=UTC),
    )
    assert sum(count for _, _, count in window) == 2


def test_errors_window_can_exclude_everything(sample_entries):
    assert errors(sample_entries, since=datetime(2030, 1, 1, tzinfo=UTC)) == []


def test_naive_bounds_treated_as_utc(sample_entries):
    naive = errors(sample_entries, since=datetime(2026, 7, 12, 12, 0))
    aware = errors(sample_entries, since=datetime(2026, 7, 12, 12, 0, tzinfo=UTC))
    assert naive == aware


def test_filter_time_without_bounds_is_identity(sample_entries):
    assert filter_time(sample_entries) == list(sample_entries)


def test_hourly_has_24_buckets_summing_to_total(sample_entries):
    buckets = hourly(sample_entries)
    assert len(buckets) == 24
    assert sum(buckets) == 32


def test_hourly_buckets_by_hour_of_day():
    buckets = hourly([make(hour=6), make(hour=6), make(hour=23)])
    assert buckets[6] == 2
    assert buckets[23] == 1
    assert buckets[0] == 0
