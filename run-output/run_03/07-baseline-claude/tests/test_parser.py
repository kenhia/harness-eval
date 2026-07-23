"""Tests for log parsing."""

from datetime import datetime

import pytest

from loglens.parser import load_log_file, parse_line


def test_parse_valid_line():
    """Test parsing a valid CLF line."""
    line = (
        '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1"'
        ' 200 5413 "https://example.com/" "Mozilla/5.0"'
    )
    entry = parse_line(line)

    assert entry is not None
    assert entry.ip == "203.0.113.7"
    assert entry.ident == "-"
    assert entry.user == "-"
    assert entry.method == "GET"
    assert entry.path == "/index.html"
    assert entry.protocol == "HTTP/1.1"
    assert entry.status == 200
    assert entry.size == 5413
    assert entry.referer == "https://example.com/"
    assert entry.user_agent == "Mozilla/5.0"
    assert entry.timestamp == datetime(2026, 7, 12, 6, 25, 24)


def test_parse_line_with_user():
    """Test parsing a line with authenticated user."""
    line = (
        '198.51.100.22 - alice [12/Jul/2026:07:01:02 +0000] "POST /api/orders HTTP/1.1"'
        ' 201 512 "-" "curl/8.5.0"'
    )
    entry = parse_line(line)

    assert entry is not None
    assert entry.user == "alice"
    assert entry.status == 201
    assert entry.referer == "-"


def test_parse_line_with_dash_size():
    """Test parsing a line with dash for size."""
    line = (
        '192.0.2.9 - - [12/Jul/2026:07:15:44 +0000] "GET /missing HTTP/1.1" 404 -'
        ' "-" "Mozilla/5.0"'
    )
    entry = parse_line(line)

    assert entry is not None
    assert entry.status == 404
    assert entry.size == 0


def test_parse_invalid_line():
    """Test that invalid lines return None."""
    line = "this is not a valid log line"
    assert parse_line(line) is None


def test_parse_malformed_status():
    """Test that lines with invalid status are skipped."""
    line = (
        '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1"'
        ' ABC 5413 "-" "Mozilla/5.0"'
    )
    assert parse_line(line) is None


def test_load_log_file(tmp_path):
    """Test loading a log file."""
    log_file = tmp_path / "test.log"
    log_file.write_text(
        '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1"'
        ' 200 5413 "-" "Mozilla/5.0"\n'
        '198.51.100.22 - alice [12/Jul/2026:07:01:02 +0000] "POST /api/orders HTTP/1.1"'
        ' 201 512 "-" "curl/8.5.0"\n'
        "invalid line\n"
    )

    entries, malformed_count = load_log_file(str(log_file))

    assert len(entries) == 2
    assert malformed_count == 1
    assert entries[0].ip == "203.0.113.7"
    assert entries[1].ip == "198.51.100.22"


def test_load_log_file_not_found():
    """Test that FileNotFoundError is raised for missing file."""
    with pytest.raises(FileNotFoundError):
        load_log_file("/nonexistent/file.log")


def test_log_entry_hour():
    """Test getting the hour from a log entry."""
    line = (
        '203.0.113.7 - - [12/Jul/2026:14:30:24 +0000] "GET /index.html HTTP/1.1"'
        ' 200 5413 "-" "Mozilla/5.0"'
    )
    entry = parse_line(line)

    assert entry.hour == 14


def test_status_code_parsing():
    """Test parsing various status codes."""
    for status_code in [200, 201, 301, 302, 400, 401, 403, 404, 500, 502, 503]:
        line = (
            '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1"'
            f' {status_code} 5413 "-" "Mozilla/5.0"'
        )
        entry = parse_line(line)
        assert entry is not None
        assert entry.status == status_code
