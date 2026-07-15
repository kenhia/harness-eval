"""Tests for the CLF parser."""

from __future__ import annotations

from datetime import UTC, datetime

from loglens.parser import parse_line, parse_lines

VALID = (
    '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] '
    '"GET /index.html HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"'
)


def test_parse_valid_line() -> None:
    rec = parse_line(VALID)
    assert rec is not None
    assert rec.ip == "203.0.113.7"
    assert rec.user == "-"
    assert rec.method == "GET"
    assert rec.path == "/index.html"
    assert rec.protocol == "HTTP/1.1"
    assert rec.status == 200
    assert rec.size == 5413
    assert rec.referer == "https://example.com/"
    assert rec.agent == "Mozilla/5.0"
    assert rec.time == datetime(2026, 7, 12, 6, 25, 24, tzinfo=UTC)


def test_parse_dash_size() -> None:
    line = '192.0.2.9 - - [12/Jul/2026:07:15:44 +0000] "GET /x HTTP/1.1" 404 -'
    rec = parse_line(line)
    assert rec is not None
    assert rec.size == 0


def test_parse_without_referer_and_agent() -> None:
    line = '192.0.2.9 - - [12/Jul/2026:07:15:44 +0000] "GET /x HTTP/1.1" 200 10'
    rec = parse_line(line)
    assert rec is not None
    assert rec.referer == ""
    assert rec.agent == ""


def test_parse_malformed_returns_none() -> None:
    assert parse_line("this is not a valid line") is None


def test_parse_bad_timestamp_returns_none() -> None:
    line = '192.0.2.9 - - [not a date] "GET /x HTTP/1.1" 200 10 "-" "-"'
    assert parse_line(line) is None


def test_parse_bad_request_returns_none() -> None:
    line = '192.0.2.9 - - [12/Jul/2026:07:15:44 +0000] "GET" 200 10 "-" "-"'
    assert parse_line(line) is None


def test_parse_lines_counts_malformed_and_skips_blanks() -> None:
    lines = [VALID, "", "   ", "garbage", VALID]
    records, malformed = parse_lines(lines)
    assert len(records) == 2
    assert malformed == 1
