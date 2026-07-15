"""Parser tests.

Weighted toward the cases that would otherwise fail *silently*: a line that parses
"successfully" into the wrong fields is worse than one that raises.
"""

from __future__ import annotations

import locale
import time
from datetime import UTC, datetime, timedelta

import pytest

from loglens.parse import LogEntry, ParseError, parse_line, parse_timestamp, split_request

BASIC = (
    '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1" 200 5413 '
    '"https://example.com/" "Mozilla/5.0"'
)


def parsed(line: str) -> LogEntry:
    result = parse_line(line, 1)
    assert isinstance(result, ParseError) is False, f"unexpectedly malformed: {result}"
    assert isinstance(result, LogEntry)
    return result


class TestBasicParsing:
    def test_all_fields(self) -> None:
        e = parsed(BASIC)
        assert e.ip == "203.0.113.7"
        assert e.ident is None
        assert e.authuser is None
        assert e.time == datetime(2026, 7, 12, 6, 25, 24, tzinfo=UTC)
        assert e.method == "GET"
        assert e.path == "/index.html"
        assert e.protocol == "HTTP/1.1"
        assert e.status == 200
        assert e.size == 5413
        assert e.referer == "https://example.com/"
        assert e.user_agent == "Mozilla/5.0"

    def test_authuser_captured(self) -> None:
        line = (
            "198.51.100.22 - alice [12/Jul/2026:07:01:02 +0000] "
            '"POST /api/orders HTTP/1.1" 201 512 "-" "curl/8.5.0"'
        )
        e = parsed(line)
        assert e.authuser == "alice"
        assert e.method == "POST"

    def test_common_log_format_without_referer_and_agent(self) -> None:
        e = parsed('203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /a HTTP/1.1" 200 5413')
        assert e.path == "/a"
        assert e.referer is None
        assert e.user_agent is None

    def test_status_is_int_not_str(self) -> None:
        # Guards the JSON contract: consumers doing d["value"] == 404 must not get False.
        assert parsed(BASIC).status == 200
        assert isinstance(parsed(BASIC).status, int)

    def test_is_error_boundary(self) -> None:
        assert parsed(BASIC.replace(" 200 ", " 399 ")).is_error is False
        assert parsed(BASIC.replace(" 200 ", " 400 ")).is_error is True
        assert parsed(BASIC.replace(" 200 ", " 500 ")).is_error is True


class TestEscapedQuotes:
    """`"([^"]*)"` would shift every later field left -- wrong status, wrong path."""

    def test_escaped_quote_in_user_agent(self) -> None:
        line = (
            '203.0.113.10 - - [12/Jul/2026:06:00:00 +0000] "GET /a HTTP/1.1" 200 100 '
            r'"-" "Mozilla/5.0 (says \"hi\")"'
        )
        e = parsed(line)
        assert e.status == 200, "fields shifted -- the escaped quote broke tokenization"
        assert e.path == "/a"
        assert e.user_agent == 'Mozilla/5.0 (says "hi")'

    def test_trailing_backslash_before_closing_quote(self) -> None:
        line = (
            '203.0.113.11 - - [12/Jul/2026:06:00:01 +0000] "GET /b HTTP/1.1" 200 100 '
            r'"-" "weird\\"'
        )
        e = parsed(line)
        assert e.status == 200, "the trailing backslash consumed the closing quote"
        assert e.size == 100
        assert e.user_agent == "weird\\"

    def test_escaped_quote_in_request(self) -> None:
        line = (
            '203.0.113.12 - - [12/Jul/2026:06:00:02 +0000] "GET /c\\"d HTTP/1.1" 200 100 '
            '"-" "curl/8.5.0"'
        )
        e = parsed(line)
        assert e.status == 200
        assert e.path == '/c"d'


class TestBytesPlaceholder:
    """`int("-")` would make every 304 malformed and inflate the error rate."""

    def test_dash_bytes_is_none_not_malformed(self) -> None:
        line = '203.0.113.13 - - [12/Jul/2026:06:00:03 +0000] "GET /e HTTP/1.1" 304 - "-" "M"'
        e = parsed(line)
        assert e.size is None
        assert e.status == 304

    def test_zero_bytes_is_zero(self) -> None:
        line = '203.0.113.13 - - [12/Jul/2026:06:00:03 +0000] "GET /e HTTP/1.1" 204 0 "-" "M"'
        assert parsed(line).size == 0


class TestRequestLine:
    """Junk request-lines are valid records: they are the traffic worth investigating."""

    @pytest.mark.parametrize(
        ("request_field", "expected_path"),
        [
            ("-", "-"),
            (r"\x16\x03\x01\x00\xa8\x01", "\x16\x03\x01\x00\xa8\x01"),
            ("GET /f g HTTP/1.1", "GET /f g HTTP/1.1"),
            ("GET /h", "GET /h"),
        ],
    )
    def test_unparseable_request_still_yields_an_entry(
        self, request_field: str, expected_path: str
    ) -> None:
        line = f'203.0.113.14 - - [12/Jul/2026:06:00:04 +0000] "{request_field}" 400 0 "-" "-"'
        e = parsed(line)
        assert e.method is None
        assert e.path == expected_path
        assert e.status == 400
        assert e.is_error is True

    def test_options_asterisk_parses_normally(self) -> None:
        line = '203.0.113.18 - - [12/Jul/2026:06:00:08 +0000] "OPTIONS * HTTP/1.0" 200 0 "-" "-"'
        e = parsed(line)
        assert e.method == "OPTIONS"
        assert e.path == "*"

    def test_absolute_form_uri(self) -> None:
        line = (
            "203.0.113.19 - - [12/Jul/2026:06:00:09 +0000] "
            '"GET http://example.com/i HTTP/1.1" 200 100 "-" "-"'
        )
        assert parsed(line).path == "http://example.com/i"

    def test_query_string_stripped(self) -> None:
        # RFC 3986: a path excludes the query. Also caps top --by path cardinality.
        line = (
            "203.0.113.21 - - [12/Jul/2026:06:00:12 +0000] "
            '"GET /search?q=hello&p=2 HTTP/1.1" 200 100 "-" "-"'
        )
        assert parsed(line).path == "/search"

    def test_split_request_directly(self) -> None:
        assert split_request("GET /a?b=1 HTTP/1.1") == ("GET", "/a", "HTTP/1.1")
        assert split_request("-") == (None, "-", None)


class TestTimestamps:
    def test_offset_preserved(self) -> None:
        e = parsed(BASIC.replace("+0000", "-0700"))
        assert e.time.utcoffset() == timedelta(hours=-7)
        assert e.time.hour == 6, "wall-clock hour must be as written in the log"

    def test_half_hour_offset(self) -> None:
        e = parsed(BASIC.replace("+0000", "+0530"))
        assert e.time.utcoffset() == timedelta(hours=5, minutes=30)

    def test_impossible_date_is_malformed(self) -> None:
        result = parse_line(BASIC.replace("12/Jul", "31/Feb"), 1)
        assert isinstance(result, ParseError)

    def test_parse_timestamp_rejects_junk(self) -> None:
        assert parse_timestamp("not a timestamp") is None
        assert parse_timestamp("12/Xxx/2026:06:25:24 +0000") is None

    def test_month_parsing_is_locale_independent(self) -> None:
        """strptime's %b reads LC_TIME: under de_DE it wants 'Okt', not 'Oct'."""
        try:
            locale.setlocale(locale.LC_TIME, "de_DE.UTF-8")
        except locale.Error:
            pytest.skip("de_DE.UTF-8 locale not available on this host")
        try:
            time.strptime  # noqa: B018 - locale is now active
            e = parsed(BASIC.replace("Jul", "Oct"))
            assert e.time.month == 10
        finally:
            locale.setlocale(locale.LC_TIME, "C")


class TestMalformed:
    @pytest.mark.parametrize(
        "line",
        [
            "this line is not a log line at all",
            '[12/Jul/2026:11:09:00 +0000] "GET /broken HTTP/1.1" 200',
            "",
            "   ",
            # status is not a 3-digit int
            '203.0.113.24 - - [12/Jul/2026:06:00:15 +0000] "GET /n HTTP/1.1" - 0 "-" "-"',
            # truncated mid-line, the log-rotation race
            '203.0.113.27 - - [12/Jul/2026:06:00:18 +0000] "GET /q HTTP/1',
        ],
    )
    def test_rejected(self, line: str) -> None:
        assert isinstance(parse_line(line, 7), ParseError)

    def test_error_carries_diagnosis(self) -> None:
        result = parse_line("total garbage", 42)
        assert isinstance(result, ParseError)
        assert result.lineno == 42
        assert result.reason
        assert result.excerpt == "total garbage"

    def test_excerpt_truncates_long_lines(self) -> None:
        result = parse_line("x" * 500, 1)
        assert isinstance(result, ParseError)
        assert len(result.excerpt) == 80

    def test_crlf_line_parses(self) -> None:
        assert parsed(BASIC + "\r\n").status == 200


class TestReDoS:
    def test_huge_user_agent_is_linear(self) -> None:
        """A crafted UA must not hang the tool you opened *because* you're under attack."""
        line = (
            '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /a HTTP/1.1" 200 100 "-" "'
            + 'A"' * 50000
        )
        start = time.perf_counter()
        parse_line(line, 1)
        assert time.perf_counter() - start < 1.0
