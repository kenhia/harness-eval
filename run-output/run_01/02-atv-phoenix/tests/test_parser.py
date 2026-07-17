from datetime import UTC, datetime
from pathlib import Path

from loglens.parser import LogRecord, parse_line, parse_lines

FIXTURE = Path(__file__).parent / "fixtures" / "sample.log"


def test_parse_valid_line():
    line = (
        '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] '
        '"GET /index.html HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"'
    )
    rec = parse_line(line)
    assert isinstance(rec, LogRecord)
    assert rec.host == "203.0.113.7"
    assert rec.user == "-"
    assert rec.method == "GET"
    assert rec.path == "/index.html"
    assert rec.protocol == "HTTP/1.1"
    assert rec.status == 200
    assert rec.size == 5413
    assert rec.referer == "https://example.com/"
    assert rec.user_agent == "Mozilla/5.0"
    assert rec.timestamp == datetime(2026, 7, 12, 6, 25, 24, tzinfo=UTC)
    assert rec.is_error is False


def test_parse_line_with_user_and_post():
    line = (
        '198.51.100.22 - alice [12/Jul/2026:07:01:02 +0000] '
        '"POST /api/orders HTTP/1.1" 201 512 "-" "curl/8.5.0"'
    )
    rec = parse_line(line)
    assert rec is not None
    assert rec.user == "alice"
    assert rec.method == "POST"
    assert rec.status == 201


def test_size_dash_becomes_zero():
    line = (
        '203.0.113.7 - - [12/Jul/2026:07:34:12 +0000] '
        '"GET /favicon.ico HTTP/1.1" 304 - "-" "Mozilla/5.0"'
    )
    rec = parse_line(line)
    assert rec is not None
    assert rec.size == 0


def test_is_error_for_4xx_and_5xx():
    base = '192.0.2.9 - - [12/Jul/2026:07:15:44 +0000] "GET /x HTTP/1.1" {code} 10 "-" "UA"'
    assert parse_line(base.format(code=404)).is_error is True
    assert parse_line(base.format(code=500)).is_error is True
    assert parse_line(base.format(code=200)).is_error is False
    assert parse_line(base.format(code=302)).is_error is False


def test_malformed_lines_return_none():
    assert parse_line("this is not a valid log line") is None
    assert parse_line("") is None
    assert parse_line("   ") is None
    # bad status (not 3 digits)
    assert (
        parse_line(
            '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET / HTTP/1.1" 20 5413 "-" "UA"'
        )
        is None
    )
    # bad timestamp
    assert (
        parse_line(
            '203.0.113.7 - - [not-a-date] "GET / HTTP/1.1" 200 5413 "-" "UA"'
        )
        is None
    )


def test_parse_lines_counts_malformed_and_ignores_blank():
    lines = [
        '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET / HTTP/1.1" 200 5413 "-" "UA"',
        "garbage line one",
        "",
        "   ",
        "garbage line two",
    ]
    records, malformed = parse_lines(lines)
    assert len(records) == 1
    assert malformed == 2  # blank lines are not counted


def test_parse_fixture_file():
    records, malformed = parse_lines(FIXTURE.read_text().splitlines())
    assert malformed >= 2
    assert len(records) >= 28
    statuses = {r.status for r in records}
    # spec requires 2xx/3xx/4xx/5xx coverage
    assert any(200 <= s < 300 for s in statuses)
    assert any(300 <= s < 400 for s in statuses)
    assert any(400 <= s < 500 for s in statuses)
    assert any(500 <= s < 600 for s in statuses)
