"""End-to-end tests for the CLI."""

from __future__ import annotations

import json

import pytest

from loglens.cli import EXIT_FILE_ERROR, EXIT_NO_VALID_LINES, EXIT_OK, main


def run(capsys, argv):
    code = main(argv)
    captured = capsys.readouterr()
    return code, captured.out, captured.err


def test_summary_text(capsys, sample_log):
    code, out, err = run(capsys, ["summary", str(sample_log)])
    assert code == EXIT_OK
    assert "Total requests:  32" in out
    assert "Unique IPs:      8" in out
    assert "31.25%" in out
    # malformed count goes to stderr, never stdout
    assert "malformed" in err
    assert "malformed" not in out


def test_summary_json(capsys, sample_log):
    code, out, err = run(capsys, ["--format", "json", "summary", str(sample_log)])
    assert code == EXIT_OK
    data = json.loads(out)  # single valid JSON document
    assert data["total_requests"] == 32
    assert data["error_rate"] == 31.25


def test_format_after_subcommand(capsys, sample_log):
    code, out, _ = run(capsys, ["summary", str(sample_log), "--format", "json"])
    assert code == EXIT_OK
    assert json.loads(out)["unique_ips"] == 8


def test_top_json(capsys, sample_log):
    argv = ["top", str(sample_log), "--by", "ip", "-n", "2", "--format", "json"]
    code, out, _ = run(capsys, argv)
    assert code == EXIT_OK
    data = json.loads(out)
    assert data["results"][0]["value"] == "198.51.100.22"
    assert data["results"][0]["count"] == 5


def test_errors_json(capsys, sample_log):
    code, out, _ = run(capsys, ["errors", str(sample_log), "--format", "json"])
    assert code == EXIT_OK
    rows = json.loads(out)
    assert rows[0] == {"status": 404, "path": "/missing", "count": 2}


def test_hourly_json(capsys, sample_log):
    code, out, _ = run(capsys, ["hourly", str(sample_log), "--format", "json"])
    assert code == EXIT_OK
    data = json.loads(out)
    assert data["09"] == 6
    assert len(data) == 24


def test_hourly_text_histogram(capsys, sample_log):
    code, out, _ = run(capsys, ["hourly", str(sample_log)])
    assert code == EXIT_OK
    assert "#" in out
    assert "09" in out


def test_missing_file_exit_2(capsys):
    code, out, err = run(capsys, ["summary", "/no/such/file.log"])
    assert code == EXIT_FILE_ERROR
    assert out == ""
    assert "cannot read" in err


def test_no_valid_lines_exit_1(capsys, tmp_path):
    bad = tmp_path / "bad.log"
    bad.write_text("garbage line one\ngarbage line two\n")
    code, out, err = run(capsys, ["summary", str(bad)])
    assert code == EXIT_NO_VALID_LINES
    assert out == ""
    assert "no valid log lines" in err


def test_empty_file_exit_1(capsys, tmp_path):
    empty = tmp_path / "empty.log"
    empty.write_text("")
    code, _, _ = run(capsys, ["summary", str(empty)])
    assert code == EXIT_NO_VALID_LINES


def test_top_requires_by(capsys, sample_log):
    with pytest.raises(SystemExit):
        main(["top", str(sample_log)])
