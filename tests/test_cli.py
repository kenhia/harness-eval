"""Tests for CLI commands."""

import json

import pytest

from loglens.cli import main


@pytest.fixture
def sample_log(tmp_path):
    """Create a sample log file for testing."""
    log_file = tmp_path / "test.log"
    lines = [
        '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1" 200 5413'
        ' "-" "Mozilla/5.0"\n',
        '198.51.100.22 - alice [12/Jul/2026:07:01:02 +0000] "POST /api/orders HTTP/1.1"'
        ' 201 512 "-" "curl/8.5.0"\n',
        '192.0.2.9 - - [12/Jul/2026:07:15:44 +0000] "GET /missing HTTP/1.1" 404 153 "-"'
        ' "Mozilla/5.0"\n',
        '203.0.113.7 - - [12/Jul/2026:08:10:15 +0000] "GET /api/users HTTP/1.1" 200 2048'
        ' "-" "Chrome/90.0"\n',
        '198.51.100.22 - - [12/Jul/2026:08:30:00 +0000] "GET /index.html HTTP/1.1" 200'
        ' 5413 "https://google.com/" "Firefox/88.0"\n',
        '192.0.2.10 - bob [12/Jul/2026:09:05:12 +0000] "POST /api/login HTTP/1.1" 401'
        ' 512 "-" "curl/8.5.0"\n',
        '203.0.113.8 - - [12/Jul/2026:09:45:33 +0000] "GET /static/js/app.js HTTP/1.1"'
        ' 200 102400 "https://example.com/" "Chrome/90.0"\n',
        '198.51.100.23 - - [12/Jul/2026:10:20:44 +0000] "GET /api/products HTTP/1.1"'
        ' 500 256 "-" "curl/7.68.0"\n',
    ]
    log_file.write_text("".join(lines))
    return str(log_file)


def test_summary_text(sample_log, capsys, monkeypatch):
    """Test summary command with text output."""
    monkeypatch.setattr("sys.argv", ["loglens", "summary", sample_log])
    result = main()
    captured = capsys.readouterr()

    assert result == 0
    assert "Total requests: 8" in captured.out
    assert "Unique client IPs: 6" in captured.out
    assert "Error rate:" in captured.out


def test_summary_json(sample_log, capsys, monkeypatch):
    """Test summary command with JSON output."""
    monkeypatch.setattr("sys.argv", ["loglens", "--format", "json", "summary", sample_log])
    result = main()
    captured = capsys.readouterr()

    assert result == 0
    data = json.loads(captured.out)
    assert data["total_requests"] == 8
    assert data["unique_client_ips"] == 6
    assert "error_rate" in data


def test_top_by_ip(sample_log, capsys, monkeypatch):
    """Test top command grouped by IP."""
    monkeypatch.setattr("sys.argv", ["loglens", "top", sample_log, "--by", "ip", "-n", "5"])
    result = main()
    captured = capsys.readouterr()

    assert result == 0
    assert "Top 5 by ip:" in captured.out
    assert "203.0.113.7" in captured.out


def test_top_by_path(sample_log, capsys, monkeypatch):
    """Test top command grouped by path."""
    monkeypatch.setattr("sys.argv", ["loglens", "top", sample_log, "--by", "path", "-n", "3"])
    result = main()
    captured = capsys.readouterr()

    assert result == 0
    assert "Top 3 by path:" in captured.out
    assert "/index.html" in captured.out


def test_top_by_status(sample_log, capsys, monkeypatch):
    """Test top command grouped by status."""
    monkeypatch.setattr("sys.argv", ["loglens", "top", sample_log, "--by", "status", "-n", "5"])
    result = main()
    captured = capsys.readouterr()

    assert result == 0
    assert "Top 5 by status:" in captured.out
    assert "200" in captured.out


def test_top_json(sample_log, capsys, monkeypatch):
    """Test top command with JSON output."""
    monkeypatch.setattr(
        "sys.argv",
        ["loglens", "--format", "json", "top", sample_log, "--by", "ip"],
    )
    result = main()
    captured = capsys.readouterr()

    assert result == 0
    data = json.loads(captured.out)
    assert isinstance(data, list)
    assert len(data) > 0
    assert "value" in data[0]
    assert "count" in data[0]


def test_errors_text(sample_log, capsys, monkeypatch):
    """Test errors command with text output."""
    monkeypatch.setattr("sys.argv", ["loglens", "errors", sample_log])
    result = main()
    captured = capsys.readouterr()

    assert result == 0
    assert "Errors" in captured.out or "404" in captured.out or "401" in captured.out


def test_errors_json(sample_log, capsys, monkeypatch):
    """Test errors command with JSON output."""
    monkeypatch.setattr("sys.argv", ["loglens", "--format", "json", "errors", sample_log])
    result = main()
    captured = capsys.readouterr()

    assert result == 0
    data = json.loads(captured.out)
    assert isinstance(data, list)
    # We have at least some errors in our sample
    assert len(data) > 0
    assert "status" in data[0]
    assert "path" in data[0]
    assert "count" in data[0]


def test_errors_with_since(sample_log, capsys, monkeypatch):
    """Test errors command with --since filter."""
    monkeypatch.setattr(
        "sys.argv",
        ["loglens", "errors", sample_log, "--since", "2026-07-12T08:00:00"],
    )
    result = main()
    _ = capsys.readouterr()

    assert result == 0
    # Should exclude the 07:15:44 error


def test_errors_with_until(sample_log, capsys, monkeypatch):
    """Test errors command with --until filter."""
    monkeypatch.setattr(
        "sys.argv",
        ["loglens", "errors", sample_log, "--until", "2026-07-12T08:00:00"],
    )
    result = main()
    _ = capsys.readouterr()

    assert result == 0


def test_hourly_text(sample_log, capsys, monkeypatch):
    """Test hourly command with text output."""
    monkeypatch.setattr("sys.argv", ["loglens", "hourly", sample_log])
    result = main()
    captured = capsys.readouterr()

    assert result == 0
    # Should have hours displayed
    for hour in range(24):
        assert f"{hour:02d}" in captured.out


def test_hourly_json(sample_log, capsys, monkeypatch):
    """Test hourly command with JSON output."""
    monkeypatch.setattr("sys.argv", ["loglens", "--format", "json", "hourly", sample_log])
    result = main()
    captured = capsys.readouterr()

    assert result == 0
    data = json.loads(captured.out)
    assert isinstance(data, dict)
    # Should have all 24 hours
    assert len(data) == 24


def test_missing_file(capsys, monkeypatch):
    """Test error handling for missing file."""
    monkeypatch.setattr("sys.argv", ["loglens", "summary", "/nonexistent/file.log"])
    result = main()
    captured = capsys.readouterr()

    assert result == 2
    assert "not found" in captured.err.lower()


def test_no_valid_lines(tmp_path, capsys, monkeypatch):
    """Test error handling when no valid log lines exist."""
    log_file = tmp_path / "test.log"
    log_file.write_text("invalid line 1\ninvalid line 2\n")

    monkeypatch.setattr("sys.argv", ["loglens", "summary", str(log_file)])
    result = main()
    captured = capsys.readouterr()

    assert result == 1
    assert "no valid log lines" in captured.err.lower()


def test_top_default_n(sample_log, capsys, monkeypatch):
    """Test that top defaults to n=10."""
    monkeypatch.setattr("sys.argv", ["loglens", "top", sample_log])
    result = main()
    captured = capsys.readouterr()

    assert result == 0
    assert "Top 10" in captured.out


def test_default_format_is_text(sample_log, capsys, monkeypatch):
    """Test that default format is text."""
    monkeypatch.setattr("sys.argv", ["loglens", "summary", sample_log])
    result = main()
    captured = capsys.readouterr()

    assert result == 0
    # Text output, not JSON
    assert "{" not in captured.out or "Total requests" in captured.out
