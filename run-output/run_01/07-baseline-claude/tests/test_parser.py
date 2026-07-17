from datetime import UTC, datetime, timedelta, timezone

import pytest

from loglens.parser import parse_line, parse_lines

VALID = (
    '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1" 200 5413 '
    '"https://example.com/" "Mozilla/5.0"'
)


def test_parses_all_fields():
    entry = parse_line(VALID)
    assert entry is not None
    assert entry.ip == "203.0.113.7"
    assert entry.user is None
    assert entry.time == datetime(2026, 7, 12, 6, 25, 24, tzinfo=UTC)
    assert entry.method == "GET"
    assert entry.path == "/index.html"
    assert entry.protocol == "HTTP/1.1"
    assert entry.status == 200
    assert entry.size == 5413
    assert entry.referer == "https://example.com/"
    assert entry.agent == "Mozilla/5.0"


def test_parses_authenticated_user():
    line = (
        '198.51.100.22 - alice [12/Jul/2026:07:01:02 +0000] "POST /api/orders HTTP/1.1" '
        '201 512 "-" "curl/8.5.0"'
    )
    entry = parse_line(line)
    assert entry is not None
    assert entry.user == "alice"
    assert entry.referer is None


def test_preserves_non_utc_offset():
    line = VALID.replace("+0000", "-0700")
    entry = parse_line(line)
    assert entry is not None
    assert entry.time.utcoffset() == timedelta(hours=-7)
    assert entry.time == datetime(2026, 7, 12, 6, 25, 24, tzinfo=timezone(timedelta(hours=-7)))


def test_dash_size_becomes_zero():
    entry = parse_line(VALID.replace(" 200 5413 ", " 304 - "))
    assert entry is not None
    assert entry.size == 0


def test_quotes_inside_user_agent():
    line = VALID.replace('"Mozilla/5.0"', r'"Mozilla/5.0 (says \"hi\")"')
    entry = parse_line(line)
    assert entry is not None
    assert entry.agent == r"Mozilla/5.0 (says \"hi\")"


def test_surrounding_whitespace_tolerated():
    assert parse_line(f"  {VALID}  \n") is not None


@pytest.mark.parametrize("status", [200, 301, 404, 500])
def test_is_error_only_for_4xx_5xx(status):
    entry = parse_line(VALID.replace(" 200 ", f" {status} "))
    assert entry is not None
    assert entry.is_error == (status >= 400)


@pytest.mark.parametrize(
    ("label", "line"),
    [
        ("empty-ish junk", "this is not a log line"),
        (
            "missing quotes on request",
            '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] GET / 200 1 "-" "-"',
        ),
        ("bad month", VALID.replace("Jul", "Zzz")),
        ("bad day", VALID.replace("12/Jul", "99/Jul")),
        ("non-numeric status", VALID.replace(" 200 ", " abc ")),
        (
            "truncated",
            '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1" 200',
        ),
        ("request missing protocol", VALID.replace("GET /index.html HTTP/1.1", "GET /index.html")),
        ("no timestamp brackets", VALID.replace("[", "").replace("]", "")),
    ],
)
def test_malformed_lines_rejected(label, line):
    assert parse_line(line) is None, label


def test_parse_lines_counts_malformed_and_ignores_blanks():
    result = parse_lines([VALID, "", "   ", "garbage", VALID, "\n"])
    assert len(result.entries) == 2
    assert result.malformed == 1


def test_sample_fixture_shape(sample_entries):
    assert len(sample_entries) == 32
    assert len({e.ip for e in sample_entries}) == 5
    statuses = {e.status // 100 for e in sample_entries}
    assert statuses == {2, 3, 4, 5}
