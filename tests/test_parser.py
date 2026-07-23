"""Tests for log parser."""

from loglens.parser import LogEntry, parse_log_file, parse_timestamp


def test_parse_timestamp_valid():
    """Test parsing a valid CLF timestamp."""
    ts = parse_timestamp("12/Jul/2026:06:25:24 +0000")
    assert ts is not None
    assert ts.year == 2026
    assert ts.month == 7
    assert ts.day == 12
    assert ts.hour == 6
    assert ts.minute == 25
    assert ts.second == 24


def test_parse_timestamp_invalid():
    """Test parsing an invalid timestamp."""
    ts = parse_timestamp("invalid")
    assert ts is None


def test_parse_log_file_valid(tmp_path):
    """Test parsing a valid log file."""
    logfile = tmp_path / "test.log"
    logfile.write_text(
        '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"\n'
        '198.51.100.22 - alice [12/Jul/2026:07:01:02 +0000] "POST /api/orders HTTP/1.1" 201 512 "-" "curl/8.5.0"\n'
    )

    entries, malformed = parse_log_file(str(logfile))
    assert len(entries) == 2
    assert malformed == 0
    assert entries[0].ip == "203.0.113.7"
    assert entries[0].status == 200
    assert entries[1].status == 201


def test_parse_log_file_with_malformed(tmp_path):
    """Test parsing a log file with malformed lines."""
    logfile = tmp_path / "test.log"
    logfile.write_text(
        '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"\n'
        'invalid line\n'
        '198.51.100.22 - alice [12/Jul/2026:07:01:02 +0000] "POST /api/orders HTTP/1.1" 201 512 "-" "curl/8.5.0"\n'
    )

    entries, malformed = parse_log_file(str(logfile))
    assert len(entries) == 2
    assert malformed == 1


def test_parse_log_file_not_found():
    """Test parsing a file that doesn't exist."""
    entries, malformed = parse_log_file("/nonexistent/file.log")
    assert entries == []
    assert malformed == -1


def test_parse_log_file_empty(tmp_path):
    """Test parsing an empty log file."""
    logfile = tmp_path / "empty.log"
    logfile.write_text("")

    entries, malformed = parse_log_file(str(logfile))
    assert entries == []
    assert malformed == 0


def test_log_entry_is_error():
    """Test LogEntry.is_error property."""
    from datetime import datetime, timezone

    # 2xx success
    entry = LogEntry(
        ip="203.0.113.7",
        timestamp=datetime(2026, 7, 12, 6, 25, 24, tzinfo=timezone.utc),
        method="GET",
        path="/index.html",
        status=200,
        bytes_sent=5413,
        referer="https://example.com/",
        user_agent="Mozilla/5.0",
    )
    assert not entry.is_error

    # 4xx error
    entry.status = 404
    assert entry.is_error

    # 5xx error
    entry.status = 500
    assert entry.is_error


def test_log_entry_hour():
    """Test LogEntry.hour property."""
    from datetime import datetime, timezone

    entry = LogEntry(
        ip="203.0.113.7",
        timestamp=datetime(2026, 7, 12, 14, 30, 0, tzinfo=timezone.utc),
        method="GET",
        path="/",
        status=200,
        bytes_sent=100,
        referer="-",
        user_agent="-",
    )
    assert entry.hour == 14
