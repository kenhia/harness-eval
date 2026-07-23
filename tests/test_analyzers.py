"""Tests for analyzers."""

from datetime import datetime, timezone

from loglens.analyzers.errors import ErrorsAnalyzer
from loglens.analyzers.hourly import HourlyAnalyzer
from loglens.analyzers.summary import SummaryAnalyzer
from loglens.analyzers.top import TopAnalyzer
from loglens.parser import LogEntry


def create_entry(
    ip="203.0.113.7",
    timestamp_str=None,
    method="GET",
    path="/",
    status=200,
    bytes_sent=100,
):
    """Helper to create a LogEntry."""
    if timestamp_str is None:
        ts = datetime(2026, 7, 12, 6, 25, 24, tzinfo=timezone.utc)
    else:
        ts = datetime.fromisoformat(timestamp_str)

    return LogEntry(
        ip=ip,
        timestamp=ts,
        method=method,
        path=path,
        status=status,
        bytes_sent=bytes_sent,
        referer="-",
        user_agent="-",
    )


def test_summary_analyzer_basic():
    """Test SummaryAnalyzer with basic entries."""
    analyzer = SummaryAnalyzer()

    analyzer.process(create_entry(ip="203.0.113.7", status=200))
    analyzer.process(create_entry(ip="203.0.113.7", status=200))
    analyzer.process(create_entry(ip="198.51.100.22", status=404))

    data = analyzer.to_dict()
    assert data["total_requests"] == 3
    assert data["unique_ips"] == 2
    assert data["error_rate_percent"] == 33.33


def test_summary_analyzer_no_errors():
    """Test SummaryAnalyzer with all successful requests."""
    analyzer = SummaryAnalyzer()

    analyzer.process(create_entry(status=200))
    analyzer.process(create_entry(status=201))
    analyzer.process(create_entry(status=301))

    data = analyzer.to_dict()
    assert data["error_rate_percent"] == 0.0


def test_top_analyzer_ip():
    """Test TopAnalyzer ranking by IP."""
    analyzer = TopAnalyzer(by="ip")

    analyzer.process(create_entry(ip="203.0.113.7"))
    analyzer.process(create_entry(ip="203.0.113.7"))
    analyzer.process(create_entry(ip="198.51.100.22"))

    top = analyzer.get_top(10)
    assert len(top) == 2
    assert top[0] == ("203.0.113.7", 2)
    assert top[1] == ("198.51.100.22", 1)


def test_top_analyzer_path():
    """Test TopAnalyzer ranking by path."""
    analyzer = TopAnalyzer(by="path")

    analyzer.process(create_entry(path="/api/users"))
    analyzer.process(create_entry(path="/api/users"))
    analyzer.process(create_entry(path="/index.html"))

    top = analyzer.get_top(10)
    assert len(top) == 2
    assert top[0][0] == "/api/users"
    assert top[0][1] == 2


def test_top_analyzer_status():
    """Test TopAnalyzer ranking by status."""
    analyzer = TopAnalyzer(by="status")

    analyzer.process(create_entry(status=200))
    analyzer.process(create_entry(status=200))
    analyzer.process(create_entry(status=404))

    top = analyzer.get_top(10)
    assert len(top) == 2
    assert top[0] == ("200", 2)


def test_top_analyzer_tie_breaking():
    """Test TopAnalyzer tie-breaking with ascending value."""
    analyzer = TopAnalyzer(by="ip")

    analyzer.process(create_entry(ip="192.0.2.2"))
    analyzer.process(create_entry(ip="192.0.2.1"))

    top = analyzer.get_top(10)
    # Both have count=1, so tie-break by value ascending
    assert top[0][0] == "192.0.2.1"
    assert top[1][0] == "192.0.2.2"


def test_errors_analyzer_basic():
    """Test ErrorsAnalyzer grouping errors."""
    analyzer = ErrorsAnalyzer()

    analyzer.process(create_entry(path="/api/users", status=404))
    analyzer.process(create_entry(path="/api/users", status=404))
    analyzer.process(create_entry(path="/api/data", status=500))
    analyzer.process(create_entry(path="/", status=200))  # Not an error

    errors = analyzer.get_sorted()
    assert len(errors) == 2
    assert errors[0][0] == (404, "/api/users")
    assert errors[0][1] == 2
    assert errors[1][0] == (500, "/api/data")
    assert errors[1][1] == 1


def test_errors_analyzer_filtering():
    """Test ErrorsAnalyzer with date range filtering."""
    since = datetime(2026, 7, 12, 12, 0, 0, tzinfo=timezone.utc)
    until = datetime(2026, 7, 12, 18, 0, 0, tzinfo=timezone.utc)
    analyzer = ErrorsAnalyzer(since=since, until=until)

    analyzer.process(
        create_entry(timestamp_str="2026-07-12T10:00:00+00:00", status=404)
    )  # Before range
    analyzer.process(
        create_entry(timestamp_str="2026-07-12T14:00:00+00:00", status=404)
    )  # In range
    analyzer.process(
        create_entry(timestamp_str="2026-07-12T20:00:00+00:00", status=404)
    )  # After range

    errors = analyzer.get_sorted()
    assert len(errors) == 1


def test_hourly_analyzer_distribution():
    """Test HourlyAnalyzer hourly distribution."""
    analyzer = HourlyAnalyzer()

    analyzer.process(create_entry(timestamp_str="2026-07-12T06:00:00+00:00"))
    analyzer.process(create_entry(timestamp_str="2026-07-12T06:30:00+00:00"))
    analyzer.process(create_entry(timestamp_str="2026-07-12T14:00:00+00:00"))

    assert analyzer.hourly_counts[6] == 2
    assert analyzer.hourly_counts[14] == 1
    assert analyzer.hourly_counts[0] == 0


def test_hourly_analyzer_histogram():
    """Test HourlyAnalyzer histogram generation."""
    analyzer = HourlyAnalyzer()

    for hour in [6, 6, 14, 14, 14]:
        analyzer.process(create_entry(timestamp_str=f"2026-07-12T{hour:02d}:00:00+00:00"))

    histogram = analyzer.to_histogram()
    assert "06:00" in histogram
    assert "14:00" in histogram
    assert histogram.count("█") > 0  # Should have bars
