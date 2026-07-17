"""Tests for the CLF parser."""

from __future__ import annotations

from datetime import UTC, datetime

from loglens.parser import parse_line, parse_lines

VALID = (
    '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] '
    '"GET /index.html HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"'
)


def test_parse_valid_line():
    rec = parse_line(VALID)
    assert rec is not None
    assert rec.ip == "203.0.113.7"
    assert rec.method == "GET"
    assert rec.path == "/index.html"
    assert rec.protocol == "HTTP/1.1"
    assert rec.status == 200
    assert rec.size == 5413
    assert rec.referer == "https://example.com/"
    assert rec.agent == "Mozilla/5.0"
    assert rec.timestamp == datetime(2026, 7, 12, 6, 25, 24, tzinfo=UTC)


def test_parse_named_user():
    line = (
        '198.51.100.22 - alice [12/Jul/2026:07:01:02 +0000] '
        '"POST /api/orders HTTP/1.1" 201 512 "-" "curl/8.5.0"'
    )
    rec = parse_line(line)
    assert rec is not None
    assert rec.user == "alice"
    assert rec.method == "POST"
    assert rec.status == 201


def test_dash_size_becomes_zero():
    line = (
        '198.51.100.7 - bob [12/Jul/2026:08:12:19 +0000] '
        '"POST /login HTTP/1.1" 302 - "-" "Mozilla/5.0"'
    )
    rec = parse_line(line)
    assert rec is not None
    assert rec.size == 0


def test_malformed_lines_return_none():
    assert parse_line("this is not a valid log line at all") is None
    assert parse_line("") is None
    assert parse_line('1.2.3.4 - - [bad-date] "GET / HTTP/1.1" 200 1') is None


def test_bad_status_is_malformed():
    line = (
        '1.2.3.4 - - [12/Jul/2026:08:12:19 +0000] '
        '"GET / HTTP/1.1" 20 5 "-" "UA"'
    )
    assert parse_line(line) is None


def test_bad_request_field_is_malformed():
    line = (
        '1.2.3.4 - - [12/Jul/2026:08:12:19 +0000] '
        '"GET /only-two-parts" 200 5 "-" "UA"'
    )
    assert parse_line(line) is None


def test_parse_lines_counts_malformed():
    lines = [VALID, "garbage", "", "   ", "still garbage"]
    result = parse_lines(lines)
    assert len(result.records) == 1
    assert result.malformed == 2


def test_sample_fixture_counts(sample_log):
    result = parse_lines(sample_log.read_text().splitlines())
    assert len(result.records) == 32
    assert result.malformed == 2
