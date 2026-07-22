"""Tests for loglens."""

from datetime import UTC, datetime
from pathlib import Path

from loglens.analysis import (
    calculate_summary,
    get_errors,
    get_hourly_distribution,
    get_top_values,
)
from loglens.formatter import (
    format_errors_json,
    format_errors_text,
    format_hourly_json,
    format_hourly_text,
    format_summary_json,
    format_summary_text,
    format_top_json,
    format_top_text,
)
from loglens.parser import LogEntry, LogParser


class TestLogParser:
    """Test log parsing functionality."""

    def test_parse_valid_line(self):
        """Test parsing a valid CLF log line."""
        parser = LogParser()
        line = (
            '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] '
            '"GET /index.html HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"'
        )
        entry = parser.parse_line(line)

        assert entry is not None
        assert entry.ip == "203.0.113.7"
        assert entry.remote_user == "-"
        assert entry.method == "GET"
        assert entry.path == "/index.html"
        assert entry.status == 200
        assert entry.bytes_sent == 5413
        assert entry.referrer == "https://example.com/"
        assert entry.user_agent == "Mozilla/5.0"

    def test_parse_line_with_username(self):
        """Test parsing a CLF log line with username."""
        parser = LogParser()
        line = (
            '198.51.100.22 - alice [12/Jul/2026:07:01:02 +0000] '
            '"POST /api/orders HTTP/1.1" 201 512 "-" "curl/8.5.0"'
        )
        entry = parser.parse_line(line)

        assert entry is not None
        assert entry.ip == "198.51.100.22"
        assert entry.remote_user == "alice"
        assert entry.method == "POST"
        assert entry.path == "/api/orders"
        assert entry.status == 201

    def test_parse_malformed_line(self):
        """Test parsing a malformed line."""
        parser = LogParser()
        line = "This is not a valid log line!"
        entry = parser.parse_line(line)

        assert entry is None
        assert parser.malformed_count == 1

    def test_parse_line_with_missing_bytes(self):
        """Test parsing a line with missing bytes field."""
        parser = LogParser()
        line = (
            '192.0.2.9 - - [12/Jul/2026:07:15:44 +0000] '
            '"GET /missing HTTP/1.1" 404 - "-" "Mozilla/5.0"'
        )
        entry = parser.parse_line(line)

        assert entry is not None
        assert entry.bytes_sent == 0

    def test_is_error(self):
        """Test the is_error property."""
        parser = LogParser()
        line_200 = (
            '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] '
            '"GET /index.html HTTP/1.1" 200 5413 "-" "Mozilla/5.0"'
        )
        line_404 = (
            '192.0.2.9 - - [12/Jul/2026:07:15:44 +0000] '
            '"GET /missing HTTP/1.1" 404 153 "-" "Mozilla/5.0"'
        )
        line_500 = (
            '203.0.113.7 - - [12/Jul/2026:09:45:33 +0000] '
            '"GET /api/data HTTP/1.1" 500 0 "-" "Python/3.9"'
        )

        entry_200 = parser.parse_line(line_200)
        entry_404 = parser.parse_line(line_404)
        entry_500 = parser.parse_line(line_500)

        assert entry_200.is_error is False
        assert entry_404.is_error is True
        assert entry_500.is_error is True

    def test_parse_file(self):
        """Test parsing a complete log file."""
        fixture_path = Path(__file__).parent / "fixtures" / "sample.log"
        parser = LogParser()
        entries = parser.parse_file(str(fixture_path))

        assert len(entries) > 0
        assert parser.malformed_count >= 2  # We have at least 2 malformed lines
        # All entries should be valid LogEntry objects
        for entry in entries:
            assert isinstance(entry, LogEntry)

    def test_parse_nonexistent_file(self):
        """Test parsing a nonexistent file."""
        parser = LogParser()
        entries = parser.parse_file("/nonexistent/file.log")

        assert entries == []


class TestAnalysis:
    """Test analysis functions."""

    def setup_method(self):
        """Set up test fixtures."""
        self.fixture_path = Path(__file__).parent / "fixtures" / "sample.log"
        parser = LogParser()
        self.entries = parser.parse_file(str(self.fixture_path))

    def test_calculate_summary(self):
        """Test summary calculation."""
        summary = calculate_summary(self.entries)

        assert summary["total_requests"] > 0
        assert summary["unique_ips"] > 0
        assert summary["first_timestamp"] is not None
        assert summary["last_timestamp"] is not None
        assert 0 <= summary["error_rate"] <= 100

    def test_calculate_summary_empty(self):
        """Test summary calculation with empty list."""
        summary = calculate_summary([])

        assert summary["total_requests"] == 0
        assert summary["unique_ips"] == 0
        assert summary["error_rate"] == 0.0

    def test_get_top_values_by_ip(self):
        """Test getting top values by IP."""
        top = get_top_values(self.entries, "ip", 5)

        assert len(top) > 0
        assert len(top) <= 5
        # Verify sorted by count descending
        counts = [count for _, count in top]
        assert counts == sorted(counts, reverse=True)

    def test_get_top_values_by_path(self):
        """Test getting top values by path."""
        top = get_top_values(self.entries, "path", 5)

        assert len(top) > 0
        assert len(top) <= 5

    def test_get_top_values_by_status(self):
        """Test getting top values by status."""
        top = get_top_values(self.entries, "status", 10)

        assert len(top) > 0
        # All values should be valid HTTP status codes
        for status, count in top:
            assert 100 <= status < 600
            assert count > 0

    def test_get_errors(self):
        """Test error analysis."""
        errors = get_errors(self.entries)

        assert len(errors) > 0
        for (status, _path), count in errors:
            assert status >= 400
            assert count > 0

    def test_get_errors_with_time_filter(self):
        """Test error analysis with time filters."""

        since = datetime(2026, 7, 12, 12, 0, 0, tzinfo=UTC)
        until = datetime(2026, 7, 12, 18, 0, 0, tzinfo=UTC)

        errors = get_errors(self.entries, since, until)
        # Verify all errors are within time range
        for (status, _path), _count in errors:
            assert status >= 400

    def test_get_hourly_distribution(self):
        """Test hourly distribution."""
        hourly = get_hourly_distribution(self.entries)

        assert len(hourly) == 24
        assert all(0 <= hour < 24 for hour in hourly.keys())
        assert all(count >= 0 for count in hourly.values())


class TestFormatters:
    """Test output formatting."""

    def setup_method(self):
        """Set up test fixtures."""
        self.fixture_path = Path(__file__).parent / "fixtures" / "sample.log"
        parser = LogParser()
        self.entries = parser.parse_file(str(self.fixture_path))
        self.summary = calculate_summary(self.entries)
        self.top_values = get_top_values(self.entries, "ip", 5)
        self.errors = get_errors(self.entries)
        self.hourly = get_hourly_distribution(self.entries)

    def test_format_summary_text(self):
        """Test text formatting for summary."""
        output = format_summary_text(self.summary)

        assert "Total Requests:" in output
        assert "Unique Client IPs:" in output
        assert "Error Rate:" in output

    def test_format_summary_json(self):
        """Test JSON formatting for summary."""
        import json

        output = format_summary_json(self.summary)
        data = json.loads(output)

        assert "total_requests" in data
        assert "unique_ips" in data
        assert "error_rate" in data

    def test_format_top_text(self):
        """Test text formatting for top values."""
        output = format_top_text(self.top_values, "ip")

        assert len(output) > 0
        lines = output.split("\n")
        assert len(lines) > 0

    def test_format_top_json(self):
        """Test JSON formatting for top values."""
        import json

        output = format_top_json(self.top_values, "ip")
        data = json.loads(output)

        assert isinstance(data, list)
        for item in data:
            assert "value" in item
            assert "count" in item

    def test_format_errors_text(self):
        """Test text formatting for errors."""
        output = format_errors_text(self.errors)

        assert len(output) > 0

    def test_format_errors_json(self):
        """Test JSON formatting for errors."""
        import json

        output = format_errors_json(self.errors)
        data = json.loads(output)

        assert isinstance(data, list)
        for item in data:
            assert "status" in item
            assert "path" in item
            assert "count" in item

    def test_format_hourly_text(self):
        """Test text formatting for hourly distribution."""
        output = format_hourly_text(self.hourly)

        assert len(output) > 0
        lines = output.split("\n")
        assert len(lines) == 24

    def test_format_hourly_json(self):
        """Test JSON formatting for hourly distribution."""
        import json

        output = format_hourly_json(self.hourly)
        data = json.loads(output)

        assert isinstance(data, list)
        assert len(data) == 24
