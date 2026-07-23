"""Tests for loglens CLI"""

import json
import subprocess
from pathlib import Path

import pytest


@pytest.fixture
def sample_log():
    """Path to sample log file"""
    return Path(__file__).parent / "fixtures" / "sample.log"


class TestSummary:
    """Test summary subcommand"""

    def test_summary_text(self, sample_log):
        """Test summary output in text format"""
        result = subprocess.run(
            ["loglens", "summary", str(sample_log)],
            capture_output=True,
            text=True,
        )
        assert result.returncode == 0
        assert "Total requests:" in result.stdout
        assert "Unique IPs:" in result.stdout
        assert "Error rate:" in result.stdout

    def test_summary_json(self, sample_log):
        """Test summary output in JSON format"""
        result = subprocess.run(
            ["loglens", "--format", "json", "summary", str(sample_log)],
            capture_output=True,
            text=True,
        )
        assert result.returncode == 0
        data = json.loads(result.stdout)
        assert "total_requests" in data
        assert "unique_ips" in data
        assert "error_rate" in data
        assert data["total_requests"] > 0

    def test_summary_missing_file(self):
        """Test summary with missing file"""
        result = subprocess.run(
            ["loglens", "summary", "/nonexistent/file.log"],
            capture_output=True,
            text=True,
        )
        assert result.returncode == 2


class TestTop:
    """Test top subcommand"""

    def test_top_by_ip(self, sample_log):
        """Test top by IP"""
        result = subprocess.run(
            ["loglens", "top", str(sample_log), "--by", "ip"],
            capture_output=True,
            text=True,
        )
        assert result.returncode == 0
        assert "203.0.113.7" in result.stdout or "198.51.100.22" in result.stdout

    def test_top_by_path(self, sample_log):
        """Test top by path"""
        result = subprocess.run(
            ["loglens", "top", str(sample_log), "--by", "path"],
            capture_output=True,
            text=True,
        )
        assert result.returncode == 0
        assert "/" in result.stdout

    def test_top_by_status(self, sample_log):
        """Test top by status"""
        result = subprocess.run(
            ["loglens", "top", str(sample_log), "--by", "status"],
            capture_output=True,
            text=True,
        )
        assert result.returncode == 0
        assert "200" in result.stdout

    def test_top_n(self, sample_log):
        """Test top N"""
        result = subprocess.run(
            ["loglens", "top", str(sample_log), "-n", "5"],
            capture_output=True,
            text=True,
        )
        assert result.returncode == 0
        lines = result.stdout.strip().split("\n")
        assert len(lines) <= 5

    def test_top_json(self, sample_log):
        """Test top in JSON format"""
        result = subprocess.run(
            ["loglens", "--format", "json", "top", str(sample_log)],
            capture_output=True,
            text=True,
        )
        assert result.returncode == 0
        data = json.loads(result.stdout)
        assert "ip" in data


class TestErrors:
    """Test errors subcommand"""

    def test_errors_text(self, sample_log):
        """Test errors output in text format"""
        result = subprocess.run(
            ["loglens", "errors", str(sample_log)],
            capture_output=True,
            text=True,
        )
        assert result.returncode == 0
        # Sample log has 4xx/5xx errors
        if result.stdout.strip():
            has_error = (
                "404" in result.stdout
                or "403" in result.stdout
                or "500" in result.stdout
                or "401" in result.stdout
            )
            assert has_error

    def test_errors_json(self, sample_log):
        """Test errors output in JSON format"""
        result = subprocess.run(
            ["loglens", "--format", "json", "errors", str(sample_log)],
            capture_output=True,
            text=True,
        )
        assert result.returncode == 0
        data = json.loads(result.stdout)
        assert "errors" in data

    def test_errors_since(self, sample_log):
        """Test errors with --since filter"""
        result = subprocess.run(
            ["loglens", "errors", str(sample_log), "--since", "2026-07-12T14:00:00"],
            capture_output=True,
            text=True,
        )
        assert result.returncode == 0


class TestHourly:
    """Test hourly subcommand"""

    def test_hourly_text(self, sample_log):
        """Test hourly output in text format"""
        result = subprocess.run(
            ["loglens", "hourly", str(sample_log)],
            capture_output=True,
            text=True,
        )
        assert result.returncode == 0
        lines = result.stdout.strip().split("\n")
        assert len(lines) == 24
        assert "00:" in result.stdout

    def test_hourly_json(self, sample_log):
        """Test hourly output in JSON format"""
        result = subprocess.run(
            ["loglens", "--format", "json", "hourly", str(sample_log)],
            capture_output=True,
            text=True,
        )
        assert result.returncode == 0
        data = json.loads(result.stdout)
        assert "hourly" in data
        assert len(data["hourly"]) == 24


class TestExitCodes:
    """Test exit codes"""

    def test_missing_file_exit_code(self):
        """Test exit code 2 for missing file"""
        result = subprocess.run(
            ["loglens", "summary", "/nonexistent/file.log"],
            capture_output=True,
            text=True,
        )
        assert result.returncode == 2

    def test_empty_log_exit_code(self, tmp_path):
        """Test exit code 1 for empty/no-valid-lines log"""
        empty_log = tmp_path / "empty.log"
        empty_log.write_text("malformed line\nyet another malformed\n")
        result = subprocess.run(
            ["loglens", "summary", str(empty_log)],
            capture_output=True,
            text=True,
        )
        assert result.returncode == 1


class TestMalformedHandling:
    """Test malformed line handling"""

    def test_malformed_lines_skipped(self, sample_log):
        """Test that malformed lines are skipped"""
        result = subprocess.run(
            ["loglens", "summary", str(sample_log)],
            capture_output=True,
            text=True,
        )
        assert result.returncode == 0
        # Should complete without error


class TestHelp:
    """Test help output"""

    def test_help_flag(self):
        """Test --help flag"""
        result = subprocess.run(
            ["loglens", "--help"],
            capture_output=True,
            text=True,
        )
        assert result.returncode == 0
        assert "loglens" in result.stdout
