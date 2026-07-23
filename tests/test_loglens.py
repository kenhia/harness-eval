import json
from pathlib import Path

import pytest
from click.testing import CliRunner

from loglens.cli import cli
from loglens.parser import LogParser


@pytest.fixture
def runner():
    """Provide a Click CLI test runner."""
    return CliRunner()


def extract_json_from_output(output: str) -> dict | list:
    """Extract JSON from output that may contain warnings."""
    lines = output.strip().split('\n')
    for line in lines:
        if line.strip().startswith('[') or line.strip().startswith('{'):
            return json.loads(line)
    raise ValueError(f"No JSON found in output: {output}")


@pytest.fixture
def sample_log():
    """Path to sample log file."""
    return str(Path(__file__).parent / "fixtures" / "sample.log")


class TestParser:
    """Tests for the log parser."""

    def test_parse_valid_line(self):
        """Test parsing a valid CLF line."""
        line = (
            '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] '
            '"GET /index.html HTTP/1.1" 200 5413 '
            '"https://example.com/" "Mozilla/5.0"'
        )
        entry = LogParser.parse_line(line)
        assert entry is not None
        assert entry.ip == "203.0.113.7"
        assert entry.path == "/index.html"
        assert entry.status == 200
        assert entry.size == 5413
        assert entry.method == "GET"

    def test_parse_malformed_line(self):
        """Test that malformed lines return None."""
        line = "This is not a valid log line"
        entry = LogParser.parse_line(line)
        assert entry is None

    def test_parse_file(self, sample_log):
        """Test parsing a complete log file."""
        entries, malformed_count = LogParser.parse_file(sample_log)
        assert len(entries) > 0
        assert malformed_count == 2  # Two malformed lines in sample.log

    def test_log_entry_is_error(self):
        """Test error status detection."""
        from datetime import datetime

        from loglens.parser import LogEntry

        entry_ok = LogEntry(
            ip="192.168.1.1",
            timestamp=datetime.now(),
            method="GET",
            path="/",
            status=200,
            size=100,
            referrer="",
            user_agent="",
        )
        assert not entry_ok.is_error

        entry_4xx = LogEntry(
            ip="192.168.1.1",
            timestamp=datetime.now(),
            method="GET",
            path="/",
            status=404,
            size=100,
            referrer="",
            user_agent="",
        )
        assert entry_4xx.is_error

        entry_5xx = LogEntry(
            ip="192.168.1.1",
            timestamp=datetime.now(),
            method="GET",
            path="/",
            status=500,
            size=100,
            referrer="",
            user_agent="",
        )
        assert entry_5xx.is_error


class TestSummaryCommand:
    """Tests for the summary subcommand."""

    def test_summary_text_format(self, runner, sample_log):
        """Test summary command with text output."""
        result = runner.invoke(cli, ["summary", sample_log, "--format", "text"])
        assert result.exit_code == 0
        assert "Total Requests:" in result.output
        assert "Unique IPs:" in result.output
        assert "Error Rate:" in result.output

    def test_summary_json_format(self, runner, sample_log):
        """Test summary command with JSON output."""
        result = runner.invoke(cli, ["summary", sample_log, "--format", "json"])
        assert result.exit_code == 0
        data = extract_json_from_output(result.output)
        assert "total_requests" in data
        assert "unique_ips" in data
        assert "first_timestamp" in data
        assert "last_timestamp" in data
        assert "error_rate" in data

    def test_summary_missing_file(self, runner):
        """Test summary command with missing file."""
        result = runner.invoke(cli, ["summary", "/nonexistent/file.log"])
        assert result.exit_code == 2
        assert "Cannot read file" in result.output

    def test_summary_no_valid_lines(self, runner):
        """Test summary command with no valid log lines."""
        with runner.isolated_filesystem():
            with open("empty.log", "w") as f:
                f.write("malformed line 1\n")
                f.write("malformed line 2\n")
            result = runner.invoke(cli, ["summary", "empty.log"])
            assert result.exit_code == 1


class TestTopCommand:
    """Tests for the top subcommand."""

    def test_top_by_ip_text(self, runner, sample_log):
        """Test top command by IP with text output."""
        result = runner.invoke(cli, ["top", sample_log, "--by", "ip", "-n", "5"])
        assert result.exit_code == 0
        assert result.output.count("\n") > 0

    def test_top_by_ip_json(self, runner, sample_log):
        """Test top command by IP with JSON output."""
        result = runner.invoke(cli, ["top", sample_log, "--by", "ip", "-n", "3", "--format", "json"])
        assert result.exit_code == 0
        data = extract_json_from_output(result.output)
        assert isinstance(data, list)
        assert len(data) <= 3
        assert all("value" in item and "count" in item for item in data)

    def test_top_by_path(self, runner, sample_log):
        """Test top command by path."""
        result = runner.invoke(cli, ["top", sample_log, "--by", "path", "-n", "5"])
        assert result.exit_code == 0

    def test_top_by_status(self, runner, sample_log):
        """Test top command by status."""
        result = runner.invoke(cli, ["top", sample_log, "--by", "status", "-n", "5"])
        assert result.exit_code == 0

    def test_top_default_limit(self, runner, sample_log):
        """Test top command with default limit (10)."""
        result = runner.invoke(cli, ["top", sample_log])
        assert result.exit_code == 0

    def test_top_missing_file(self, runner):
        """Test top command with missing file."""
        result = runner.invoke(cli, ["top", "/nonexistent/file.log"])
        assert result.exit_code == 2


class TestErrorsCommand:
    """Tests for the errors subcommand."""

    def test_errors_text_format(self, runner, sample_log):
        """Test errors command with text output."""
        result = runner.invoke(cli, ["errors", sample_log, "--format", "text"])
        assert result.exit_code == 0

    def test_errors_json_format(self, runner, sample_log):
        """Test errors command with JSON output."""
        result = runner.invoke(cli, ["errors", sample_log, "--format", "json"])
        assert result.exit_code == 0
        data = extract_json_from_output(result.output)
        assert isinstance(data, list)
        for item in data:
            assert "status" in item
            assert "path" in item
            assert "count" in item
            assert item["status"] >= 400

    def test_errors_with_since_filter(self, runner, sample_log):
        """Test errors command with --since filter."""
        result = runner.invoke(
            cli, ["errors", sample_log, "--since", "2026-07-12T10:00:00"]
        )
        assert result.exit_code == 0

    def test_errors_with_until_filter(self, runner, sample_log):
        """Test errors command with --until filter."""
        result = runner.invoke(
            cli, ["errors", sample_log, "--until", "2026-07-12T15:00:00"]
        )
        assert result.exit_code == 0

    def test_errors_with_both_filters(self, runner, sample_log):
        """Test errors command with both --since and --until filters."""
        result = runner.invoke(
            cli,
            [
                "errors",
                sample_log,
                "--since",
                "2026-07-12T10:00:00",
                "--until",
                "2026-07-12T20:00:00",
            ],
        )
        assert result.exit_code == 0

    def test_errors_invalid_since_format(self, runner, sample_log):
        """Test errors command with invalid --since format."""
        result = runner.invoke(cli, ["errors", sample_log, "--since", "invalid-date"])
        assert result.exit_code == 2

    def test_errors_missing_file(self, runner):
        """Test errors command with missing file."""
        result = runner.invoke(cli, ["errors", "/nonexistent/file.log"])
        assert result.exit_code == 2


class TestHourlyCommand:
    """Tests for the hourly subcommand."""

    def test_hourly_text_format(self, runner, sample_log):
        """Test hourly command with text output."""
        result = runner.invoke(cli, ["hourly", sample_log, "--format", "text"])
        assert result.exit_code == 0
        # Should have 24 hours of output (or at least some)
        assert ":00" in result.output

    def test_hourly_json_format(self, runner, sample_log):
        """Test hourly command with JSON output."""
        result = runner.invoke(cli, ["hourly", sample_log, "--format", "json"])
        assert result.exit_code == 0
        data = extract_json_from_output(result.output)
        assert isinstance(data, list)
        assert len(data) == 24
        for item in data:
            assert "hour" in item
            assert "count" in item
            assert 0 <= item["hour"] < 24

    def test_hourly_missing_file(self, runner):
        """Test hourly command with missing file."""
        result = runner.invoke(cli, ["hourly", "/nonexistent/file.log"])
        assert result.exit_code == 2


class TestMalformedLineHandling:
    """Tests for handling of malformed lines."""

    def test_malformed_lines_warning(self, runner, sample_log):
        """Test that malformed lines produce a warning on stderr."""
        result = runner.invoke(cli, ["summary", sample_log])
        assert result.exit_code == 0
        assert "malformed" in result.output.lower() or "warning" in result.output.lower()

    def test_no_valid_lines_error(self, runner):
        """Test exit code 1 when no valid lines are found."""
        with runner.isolated_filesystem():
            with open("invalid.log", "w") as f:
                f.write("This is not a valid log line\n")
                f.write("Neither is this\n")
            result = runner.invoke(cli, ["summary", "invalid.log"])
            assert result.exit_code == 1


class TestExitCodes:
    """Tests for correct exit codes."""

    def test_success_exit_code(self, runner, sample_log):
        """Test exit code 0 for successful execution."""
        result = runner.invoke(cli, ["summary", sample_log])
        assert result.exit_code == 0

    def test_no_valid_lines_exit_code(self, runner):
        """Test exit code 1 when no valid lines."""
        with runner.isolated_filesystem():
            with open("invalid.log", "w") as f:
                f.write("invalid\n")
            result = runner.invoke(cli, ["summary", "invalid.log"])
            assert result.exit_code == 1

    def test_missing_file_exit_code(self, runner):
        """Test exit code 2 when file is missing."""
        result = runner.invoke(cli, ["summary", "/nonexistent/file.log"])
        assert result.exit_code == 2
