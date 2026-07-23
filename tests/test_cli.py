"""Integration tests for CLI."""

import json

import pytest
from click.testing import CliRunner

from loglens.cli import main


@pytest.fixture
def runner():
    """Return a Click CLI test runner."""
    return CliRunner()


@pytest.fixture
def sample_log(tmp_path):
    """Create a sample log file for testing."""
    logfile = tmp_path / "test.log"
    logfile.write_text(
        '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"\n'
        '198.51.100.22 - alice [12/Jul/2026:07:01:02 +0000] "POST /api/orders HTTP/1.1" 201 512 "-" "curl/8.5.0"\n'
        '192.0.2.9 - - [12/Jul/2026:08:30:44 +0000] "GET /missing HTTP/1.1" 404 153 "-" "Mozilla/5.0"\n'
        '203.0.113.7 - - [12/Jul/2026:09:45:15 +0000] "GET /api/users HTTP/1.1" 500 2048 "https://example.com/" "Mozilla/5.0"\n'
        'malformed line\n'
    )
    return str(logfile)


def test_summary_text(runner, sample_log):
    """Test summary command with text output."""
    result = runner.invoke(main, ["--format", "text", "summary", sample_log])
    assert result.exit_code == 0
    assert "Total Requests:" in result.output
    assert "Unique IPs:" in result.output
    assert "Error Rate:" in result.output


def test_summary_json(runner, sample_log):
    """Test summary command with JSON output."""
    result = runner.invoke(main, ["--format", "json", "summary", sample_log])
    assert result.exit_code == 0
    data = json.loads(result.output)
    assert "total_requests" in data
    assert "unique_ips" in data
    assert "error_rate_percent" in data


def test_top_by_ip(runner, sample_log):
    """Test top command ranking by IP."""
    result = runner.invoke(main, ["--format", "text", "top", "--by", "ip", sample_log])
    assert result.exit_code == 0
    assert "203.0.113.7" in result.output


def test_top_by_path(runner, sample_log):
    """Test top command ranking by path."""
    result = runner.invoke(
        main, ["--format", "text", "top", "--by", "path", "-n", "5", sample_log]
    )
    assert result.exit_code == 0
    assert "/index.html" in result.output or "/api" in result.output


def test_top_by_status(runner, sample_log):
    """Test top command ranking by status."""
    result = runner.invoke(
        main, ["--format", "text", "top", "--by", "status", sample_log]
    )
    assert result.exit_code == 0
    assert "200" in result.output


def test_errors_text(runner, sample_log):
    """Test errors command with text output."""
    result = runner.invoke(main, ["--format", "text", "errors", sample_log])
    assert result.exit_code == 0
    # Should show errors (404, 500)
    assert "404" in result.output or "500" in result.output


def test_errors_json(runner, sample_log):
    """Test errors command with JSON output."""
    result = runner.invoke(main, ["--format", "json", "errors", sample_log])
    assert result.exit_code == 0
    data = json.loads(result.output)
    assert "errors" in data


def test_hourly_text(runner, sample_log):
    """Test hourly command with text output."""
    result = runner.invoke(main, ["--format", "text", "hourly", sample_log])
    assert result.exit_code == 0
    # Should show hourly histogram
    assert "06:00" in result.output


def test_hourly_json(runner, sample_log):
    """Test hourly command with JSON output."""
    result = runner.invoke(main, ["--format", "json", "hourly", sample_log])
    assert result.exit_code == 0
    data = json.loads(result.output)
    assert "hourly" in data


def test_file_not_found(runner):
    """Test command with non-existent file."""
    result = runner.invoke(main, ["summary", "/nonexistent/file.log"])
    assert result.exit_code == 2
    assert "not found" in result.output.lower()


def test_empty_file(runner, tmp_path):
    """Test command with empty log file."""
    logfile = tmp_path / "empty.log"
    logfile.write_text("")

    result = runner.invoke(main, ["summary", str(logfile)])
    assert result.exit_code == 1
    assert "No valid log lines" in result.output
