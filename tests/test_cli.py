"""End-to-end tests for the CLI."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from loglens.cli import EXIT_FILE_ERROR, EXIT_NO_VALID_LINES, EXIT_OK, main


def test_summary_text(sample_log: Path, capsys: pytest.CaptureFixture[str]) -> None:
    code = main(["summary", str(sample_log)])
    out = capsys.readouterr()
    assert code == EXIT_OK
    assert "Total requests: 32" in out.out
    assert "Unique client IPs: 4" in out.out
    # Malformed count goes to stderr, never stdout.
    assert "malformed" in out.err
    assert "malformed" not in out.out


def test_summary_json(sample_log: Path, capsys: pytest.CaptureFixture[str]) -> None:
    code = main(["--format", "json", "summary", str(sample_log)])
    out = capsys.readouterr()
    assert code == EXIT_OK
    data = json.loads(out.out)
    assert data["total_requests"] == 32
    assert data["unique_ips"] == 4


def test_top_json(sample_log: Path, capsys: pytest.CaptureFixture[str]) -> None:
    code = main(["--format", "json", "top", str(sample_log), "--by", "path", "-n", "3"])
    out = capsys.readouterr()
    assert code == EXIT_OK
    data = json.loads(out.out)
    assert len(data) == 3
    assert data[0]["value"] == "/index.html"


def test_errors_json(sample_log: Path, capsys: pytest.CaptureFixture[str]) -> None:
    code = main(["--format", "json", "errors", str(sample_log)])
    out = capsys.readouterr()
    assert code == EXIT_OK
    data = json.loads(out.out)
    assert data[0]["path"] == "/missing"
    assert data[0]["count"] == 3


def test_hourly_json(sample_log: Path, capsys: pytest.CaptureFixture[str]) -> None:
    code = main(["--format", "json", "hourly", str(sample_log)])
    out = capsys.readouterr()
    assert code == EXIT_OK
    data = json.loads(out.out)
    assert len(data) == 24
    assert data["06"] == 1


def test_hourly_text_histogram(sample_log: Path, capsys: pytest.CaptureFixture[str]) -> None:
    code = main(["hourly", str(sample_log)])
    out = capsys.readouterr()
    assert code == EXIT_OK
    assert "#" in out.out
    assert out.out.strip().splitlines()[0].startswith("00")


def test_missing_file(capsys: pytest.CaptureFixture[str]) -> None:
    code = main(["summary", "/no/such/file.log"])
    out = capsys.readouterr()
    assert code == EXIT_FILE_ERROR
    assert out.out == ""
    assert "cannot read" in out.err


def test_no_valid_lines(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    empty = tmp_path / "bad.log"
    empty.write_text("garbage\nmore garbage\n")
    code = main(["summary", str(empty)])
    out = capsys.readouterr()
    assert code == EXIT_NO_VALID_LINES
    assert out.out == ""
    assert "no valid log lines" in out.err


def test_json_stdout_is_single_document(
    sample_log: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    main(["--format", "json", "summary", str(sample_log)])
    out = capsys.readouterr()
    # Parsing the entire stdout as one JSON document must succeed.
    json.loads(out.out)
