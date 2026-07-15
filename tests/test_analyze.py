from datetime import UTC, datetime

from loglens import analyze
from loglens.parser import LogRecord


def rec(ip="1.1.1.1", path="/", status=200, hour=6):
    return LogRecord(
        ip=ip,
        user="",
        timestamp=datetime(2026, 7, 12, hour, 0, 0, tzinfo=UTC),
        method="GET",
        path=path,
        protocol="HTTP/1.1",
        status=status,
        size=100,
        referer="",
        user_agent="",
    )


def test_summarize_basic():
    records = [
        rec(ip="a", status=200, hour=6),
        rec(ip="b", status=404, hour=7),
        rec(ip="a", status=500, hour=8),
        rec(ip="c", status=302, hour=9),
    ]
    stats = analyze.summarize(records)
    assert stats["total_requests"] == 4
    assert stats["unique_ips"] == 3
    assert stats["first_timestamp"].hour == 6
    assert stats["last_timestamp"].hour == 9
    assert stats["error_rate"] == 50.0


def test_summarize_no_errors():
    records = [rec(status=200), rec(status=304)]
    assert analyze.summarize(records)["error_rate"] == 0.0


def test_top_by_ip_desc_with_tiebreak():
    records = [rec(ip="b"), rec(ip="b"), rec(ip="a"), rec(ip="a"), rec(ip="c")]
    result = analyze.top(records, by="ip", n=10)
    # a and b tie at 2 -> a before b (value asc); c last
    assert result == [("a", 2), ("b", 2), ("c", 1)]


def test_top_by_status():
    records = [rec(status=200), rec(status=200), rec(status=404)]
    assert analyze.top(records, by="status", n=10) == [(200, 2), (404, 1)]


def test_top_respects_n():
    records = [rec(path="/a"), rec(path="/b"), rec(path="/c")]
    assert len(analyze.top(records, by="path", n=2)) == 2


def test_errors_grouping_and_order():
    records = [
        rec(status=404, path="/x"),
        rec(status=404, path="/x"),
        rec(status=500, path="/y"),
        rec(status=200, path="/ok"),
    ]
    result = analyze.errors(records)
    assert result == [((404, "/x"), 2), ((500, "/y"), 1)]


def test_errors_time_window():
    records = [
        rec(status=500, path="/a", hour=6),
        rec(status=500, path="/a", hour=8),
        rec(status=500, path="/a", hour=10),
    ]
    since = datetime(2026, 7, 12, 7, 0, 0, tzinfo=UTC)
    until = datetime(2026, 7, 12, 10, 0, 0, tzinfo=UTC)
    # inclusive since, exclusive until -> only the hour=8 record
    assert analyze.errors(records, since=since, until=until) == [((500, "/a"), 1)]


def test_hourly_buckets():
    records = [rec(hour=6), rec(hour=6), rec(hour=13)]
    buckets = analyze.hourly(records)
    assert len(buckets) == 24
    assert buckets[6] == 2
    assert buckets[13] == 1
    assert buckets[0] == 0
