"""Tests for loglens CLI and modules."""

import tempfile
from datetime import datetime, timezone
from pathlib import Path

import pytest

from loglens.parser import parse_log_line, parse_log_file, LogEntry
from loglens.commands import summary, top, errors, hourly
from loglens.commands import (
    format_text_summary,
    format_text_top,
    format_text_errors,
    format_text_hourly,
)


class TestLogParsing:
    """Test CLF log parsing."""

    def test_parse_valid_log_line(self):
        """Parse a valid CLF log line."""
        line = (
            '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html '
            'HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"'
        )
        entry = parse_log_line(line)

        assert entry is not None
        assert entry.ip == "203.0.113.7"
        assert entry.user is None
        assert entry.method == "GET"
        assert entry.path == "/index.html"
        assert entry.status == 200
        assert entry.bytes_sent == 5413

    def test_parse_log_line_with_user(self):
        """Parse log line with authenticated user."""
        line = (
            '198.51.100.22 - alice [12/Jul/2026:07:01:02 +0000] "POST '
            '/api/orders HTTP/1.1" 201 512 "-" "curl/8.5.0"'
        )
        entry = parse_log_line(line)

        assert entry is not None
        assert entry.ip == "198.51.100.22"
        assert entry.user == "alice"
        assert entry.method == "POST"
        assert entry.status == 201

    def test_parse_malformed_line(self):
        """Malformed line returns None."""
        line = "THIS IS A MALFORMED LOG LINE"
        entry = parse_log_line(line)
        assert entry is None

    def test_parse_log_file(self):
        """Parse a complete log file."""
        with tempfile.NamedTemporaryFile(mode="w", suffix=".log", delete=False) as f:
            f.write(
                '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html '
                'HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"\n'
            )
            f.write(
                '192.0.2.9 - - [12/Jul/2026:07:15:44 +0000] "GET /missing '
                'HTTP/1.1" 404 153 "-" "Mozilla/5.0"\n'
            )
            f.write("MALFORMED LINE\n")
            f.flush()

            entries, malformed = parse_log_file(f.name)

            assert len(entries) == 2
            assert malformed == 1

    def test_parse_nonexistent_file(self):
        """Nonexistent file raises FileNotFoundError."""
        with pytest.raises(FileNotFoundError):
            parse_log_file("/nonexistent/file.log")


class TestSummaryCommand:
    """Test summary command."""

    def test_summary_basic(self):
        """Generate summary from entries."""
        entries = [
            LogEntry(
                ip="203.0.113.7",
                ident="-",
                user=None,
                timestamp=datetime(2026, 7, 12, 6, 25, 24, tzinfo=timezone.utc),
                method="GET",
                path="/index.html",
                protocol="HTTP/1.1",
                status=200,
                bytes_sent=5413,
                referrer="https://example.com/",
                user_agent="Mozilla/5.0",
            ),
            LogEntry(
                ip="203.0.113.7",
                ident="-",
                user=None,
                timestamp=datetime(2026, 7, 12, 7, 15, 44, tzinfo=timezone.utc),
                method="GET",
                path="/missing",
                protocol="HTTP/1.1",
                status=404,
                bytes_sent=153,
                referrer="-",
                user_agent="Mozilla/5.0",
            ),
        ]

        result = summary(entries)

        assert result["total_requests"] == 2
        assert result["unique_ips"] == 1
        assert result["error_rate"] == 50.0

    def test_summary_empty(self):
        """Empty entries return empty dict."""
        result = summary([])
        assert result == {}


class TestTopCommand:
    """Test top command."""

    def test_top_by_ip(self):
        """Top IPs by request count."""
        entries = [
            LogEntry("203.0.113.7", "-", None, datetime.now(), "GET", "/a",
                     "HTTP/1.1", 200, 0, None, None),
            LogEntry("203.0.113.7", "-", None, datetime.now(), "GET", "/b",
                     "HTTP/1.1", 200, 0, None, None),
            LogEntry("192.0.2.9", "-", None, datetime.now(), "GET", "/c",
                     "HTTP/1.1", 200, 0, None, None),
        ]

        result = top(entries, by="ip", n=10)

        assert len(result) == 2
        assert result[0]["ip"] == "203.0.113.7"
        assert result[0]["count"] == 2
        assert result[1]["ip"] == "192.0.2.9"
        assert result[1]["count"] == 1

    def test_top_by_path(self):
        """Top paths by request count."""
        entries = [
            LogEntry("203.0.113.7", "-", None, datetime.now(), "GET", "/api",
                     "HTTP/1.1", 200, 0, None, None),
            LogEntry("203.0.113.7", "-", None, datetime.now(), "GET", "/api",
                     "HTTP/1.1", 200, 0, None, None),
            LogEntry("192.0.2.9", "-", None, datetime.now(), "GET", "/home",
                     "HTTP/1.1", 200, 0, None, None),
        ]

        result = top(entries, by="path", n=10)

        assert len(result) == 2
        assert result[0]["path"] == "/api"
        assert result[0]["count"] == 2

    def test_top_by_status(self):
        """Top status codes by frequency."""
        entries = [
            LogEntry("203.0.113.7", "-", None, datetime.now(), "GET", "/a",
                     "HTTP/1.1", 200, 0, None, None),
            LogEntry("203.0.113.7", "-", None, datetime.now(), "GET", "/b",
                     "HTTP/1.1", 200, 0, None, None),
            LogEntry("192.0.2.9", "-", None, datetime.now(), "GET", "/c",
                     "HTTP/1.1", 404, 0, None, None),
        ]

        result = top(entries, by="status", n=10)

        assert len(result) == 2
        assert result[0]["status"] == 200
        assert result[0]["count"] == 2

    def test_top_limit(self):
        """Respect n parameter."""
        entries = [
            LogEntry(f"192.0.2.{i}", "-", None, datetime.now(), "GET", "/",
                     "HTTP/1.1", 200, 0, None, None)
            for i in range(15)
        ]

        result = top(entries, by="ip", n=5)

        assert len(result) == 5

    def test_top_ties_broken_by_value(self):
        """Ties in count broken by value ascending."""
        entries = [
            LogEntry("203.0.113.7", "-", None, datetime.now(), "GET", "/a",
                     "HTTP/1.1", 200, 0, None, None),
            LogEntry("192.0.2.9", "-", None, datetime.now(), "GET", "/b",
                     "HTTP/1.1", 200, 0, None, None),
        ]

        result = top(entries, by="ip", n=10)

        # Both have count 1, should be sorted by IP value
        assert result[0]["ip"] == "192.0.2.9"
        assert result[1]["ip"] == "203.0.113.7"


class TestErrorsCommand:
    """Test errors command."""

    def test_errors_basic(self):
        """Filter and group errors by status and path."""
        entries = [
            LogEntry("203.0.113.7", "-", None, datetime.now(), "GET",
                     "/missing", "HTTP/1.1", 404, 0, None, None),
            LogEntry("203.0.113.7", "-", None, datetime.now(), "GET",
                     "/missing", "HTTP/1.1", 404, 0, None, None),
            LogEntry("192.0.2.9", "-", None, datetime.now(), "GET", "/error",
                     "HTTP/1.1", 500, 0, None, None),
            LogEntry("203.0.113.7", "-", None, datetime.now(), "GET", "/index",
                     "HTTP/1.1", 200, 0, None, None),
        ]

        result = errors(entries)

        assert len(result) == 2
        assert result[0]["status"] == 404
        assert result[0]["path"] == "/missing"
        assert result[0]["count"] == 2

    def test_errors_with_date_filter(self):
        """Filter errors by date range."""
        dt1 = datetime(2026, 7, 12, 6, 0, 0, tzinfo=timezone.utc)
        dt2 = datetime(2026, 7, 12, 8, 0, 0, tzinfo=timezone.utc)

        entries = [
            LogEntry("203.0.113.7", "-", None, dt1, "GET", "/missing",
                     "HTTP/1.1", 404, 0, None, None),
            LogEntry("203.0.113.7", "-", None, dt2, "GET", "/error",
                     "HTTP/1.1", 500, 0, None, None),
        ]

        since = datetime(2026, 7, 12, 7, 0, 0, tzinfo=timezone.utc)
        result = errors(entries, since=since)

        assert len(result) == 1
        assert result[0]["status"] == 500


class TestHourlyCommand:
    """Test hourly command."""

    def test_hourly_histogram(self):
        """Generate hourly request counts."""
        entries = [
            LogEntry("203.0.113.7", "-", None,
                     datetime(2026, 7, 12, 6, 0, 0), "GET", "/", "HTTP/1.1",
                     200, 0, None, None),
            LogEntry("203.0.113.7", "-", None,
                     datetime(2026, 7, 12, 6, 30, 0), "GET", "/", "HTTP/1.1",
                     200, 0, None, None),
            LogEntry("192.0.2.9", "-", None, datetime(2026, 7, 12, 7, 0, 0),
                     "GET", "/", "HTTP/1.1", 200, 0, None, None),
        ]

        result = hourly(entries)

        assert len(result) == 24
        assert result[6]["hour"] == 6
        assert result[6]["count"] == 2
        assert result[7]["hour"] == 7
        assert result[7]["count"] == 1


class TestFormatting:
    """Test output formatting."""

    def test_format_text_summary(self):
        """Format summary as text."""
        data = {
            "total_requests": 100,
            "unique_ips": 5,
            "first_timestamp": "2026-07-12T06:25:24+00:00",
            "last_timestamp": "2026-07-12T13:40:22+00:00",
            "error_rate": 25.5,
        }

        output = format_text_summary(data)

        assert "Total Requests: 100" in output
        assert "Unique IPs: 5" in output
        assert "Error Rate: 25.5%" in output

    def test_format_text_top(self):
        """Format top data as text."""
        data = [
            {"ip": "203.0.113.7", "count": 10},
            {"ip": "192.0.2.9", "count": 5},
        ]

        output = format_text_top(data, "ip")

        assert "Top ip:" in output
        assert "203.0.113.7: 10" in output
        assert "192.0.2.9: 5" in output

    def test_format_text_errors(self):
        """Format errors as text."""
        data = [
            {"status": 404, "path": "/missing", "count": 3},
            {"status": 500, "path": "/error", "count": 1},
        ]

        output = format_text_errors(data)

        assert "404 /missing: 3" in output
        assert "500 /error: 1" in output

    def test_format_text_hourly(self):
        """Format hourly data as histogram."""
        data = [{"hour": h, "count": h % 3} for h in range(24)]

        output = format_text_hourly(data)

        assert "Requests per Hour:" in output
        assert "00:" in output


class TestSampleLogFile:
    """Test with actual sample.log fixture."""

    @pytest.fixture
    def sample_log_path(self):
        """Return path to sample.log."""
        return Path(__file__).parent / "fixtures" / "sample.log"

    def test_parse_sample_log(self, sample_log_path):
        """Parse sample log file."""
        entries, malformed = parse_log_file(str(sample_log_path))

        assert len(entries) > 30
        assert malformed >= 2  # At least 2 malformed lines

    def test_summary_sample_log(self, sample_log_path):
        """Generate summary from sample log."""
        entries, _ = parse_log_file(str(sample_log_path))
        result = summary(entries)

        assert result["total_requests"] > 30
        assert result["unique_ips"] > 1
        assert 0 <= result["error_rate"] <= 100

    def test_top_sample_log(self, sample_log_path):
        """Get top values from sample log."""
        entries, _ = parse_log_file(str(sample_log_path))

        top_ips = top(entries, by="ip", n=5)
        assert len(top_ips) > 0
        assert all("ip" in item for item in top_ips)
        assert all("count" in item for item in top_ips)

    def test_errors_sample_log(self, sample_log_path):
        """Get errors from sample log."""
        entries, _ = parse_log_file(str(sample_log_path))
        result = errors(entries)

        assert all(item["status"] >= 400 for item in result)

    def test_hourly_sample_log(self, sample_log_path):
        """Get hourly distribution from sample log."""
        entries, _ = parse_log_file(str(sample_log_path))
        result = hourly(entries)

        assert len(result) == 24
        assert sum(item["count"] for item in result) == len(entries)
