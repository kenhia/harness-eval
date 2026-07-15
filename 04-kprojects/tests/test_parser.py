"""Tests for the CLF parser."""

from datetime import UTC, datetime

from loglens.parser import parse_line, parse_lines

VALID = (
    '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] '
    '"GET /index.html HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"'
)


def test_parse_valid_line():
    record = parse_line(VALID)
    assert record is not None
    assert record.host == "203.0.113.7"
    assert record.method == "GET"
    assert record.path == "/index.html"
    assert record.protocol == "HTTP/1.1"
    assert record.status == 200
    assert record.size == 5413
    assert record.referer == "https://example.com/"
    assert record.useragent == "Mozilla/5.0"
    assert record.timestamp == datetime(2026, 7, 12, 6, 25, 24, tzinfo=UTC)


def test_parse_authuser():
    line = (
        '198.51.100.22 - alice [12/Jul/2026:07:01:02 +0000] '
        '"POST /api/orders HTTP/1.1" 201 512 "-" "curl/8.5.0"'
    )
    record = parse_line(line)
    assert record is not None
    assert record.authuser == "alice"
    assert record.method == "POST"
    assert record.status == 201


def test_parse_dash_size():
    line = (
        '192.0.2.55 - - [12/Jul/2026:11:00:47 +0000] '
        '"POST /login HTTP/1.1" 302 - "-" "Mozilla/5.0"'
    )
    record = parse_line(line)
    assert record is not None
    assert record.size is None


def test_malformed_returns_none():
    assert parse_line("this is not a valid log line") is None
    assert parse_line("") is None
    assert parse_line("malformed [12/Jul/2026 GET nonsense line") is None


def test_bad_timestamp_returns_none():
    line = (
        '203.0.113.7 - - [99/Xxx/2026:06:25:24 +0000] '
        '"GET /x HTTP/1.1" 200 10 "-" "UA"'
    )
    assert parse_line(line) is None


def test_empty_request_returns_none():
    line = '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "" 400 10 "-" "UA"'
    assert parse_line(line) is None


def test_parse_lines_counts_malformed():
    lines = [VALID, "garbage line", "", VALID, "another bad one"]
    result = parse_lines(lines)
    assert len(result.records) == 2
    assert result.malformed == 2
