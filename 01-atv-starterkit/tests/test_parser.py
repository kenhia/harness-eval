from datetime import UTC, datetime

from loglens.parser import parse_line, parse_lines

CANON = (
    '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] '
    '"GET /index.html HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"'
)
NAMED_USER = (
    '198.51.100.22 - alice [12/Jul/2026:07:01:02 +0000] '
    '"POST /api/orders HTTP/1.1" 201 512 "-" "curl/8.5.0"'
)


def test_parse_canonical_line():
    rec = parse_line(CANON)
    assert rec is not None
    assert rec.ip == "203.0.113.7"
    assert rec.user == ""
    assert rec.method == "GET"
    assert rec.path == "/index.html"
    assert rec.protocol == "HTTP/1.1"
    assert rec.status == 200
    assert rec.size == 5413
    assert rec.referer == "https://example.com/"
    assert rec.user_agent == "Mozilla/5.0"


def test_timestamp_is_timezone_aware():
    rec = parse_line(CANON)
    assert rec.timestamp == datetime(2026, 7, 12, 6, 25, 24, tzinfo=UTC)
    assert rec.timestamp.tzinfo is not None


def test_named_user_and_dash_size():
    line = NAMED_USER.replace(" 512 ", " - ")
    rec = parse_line(line)
    assert rec is not None
    assert rec.user == "alice"
    assert rec.size == 0


def test_blank_line_returns_none():
    assert parse_line("") is None
    assert parse_line("   ") is None


def test_malformed_lines_return_none():
    assert parse_line("this is not a valid log line") is None
    # non-numeric status
    assert parse_line(
        '1.2.3.4 - - [12/Jul/2026:06:25:24 +0000] "GET / HTTP/1.1" abc 10 "-" "-"'
    ) is None
    # bad date
    assert parse_line(
        '1.2.3.4 - - [not-a-date] "GET / HTTP/1.1" 200 10 "-" "-"'
    ) is None


def test_parse_lines_counts_malformed_and_skips_blank():
    lines = [CANON, "", "garbage line", NAMED_USER, "   "]
    records, malformed = parse_lines(lines)
    assert len(records) == 2
    assert malformed == 1
