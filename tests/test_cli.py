"""End-to-end tests for the loglens CLI."""

import json
from pathlib import Path

import pytest

from loglens.cli import (
    EXIT_FILE_ERROR,
    EXIT_NO_VALID_LINES,
    EXIT_OK,
    main,
)

FIXTURE = str(Path(__file__).parent / "fixtures" / "sample.log")


def test_summary_text(capsys):
    code = main(["summary", FIXTURE])
    out = capsys.readouterr()
    assert code == EXIT_OK
    assert "Total requests:" in out.out
    assert "Unique IPs:" in out.out
    assert "Error rate:" in out.out
    # malformed count reported on stderr, never stdout
    assert "malformed" in out.err
    assert "malformed" not in out.out


def test_summary_json(capsys):
    code = main(["--format", "json", "summary", FIXTURE])
    out = capsys.readouterr()
    assert code == EXIT_OK
    data = json.loads(out.out)
    assert data["total_requests"] == 32
    assert data["unique_ips"] == 6
    assert "error_rate" in data


def test_top_json(capsys):
    code = main(["--format", "json", "top", FIXTURE, "--by", "path", "-n", "3"])
    out = capsys.readouterr()
    assert code == EXIT_OK
    data = json.loads(out.out)
    assert data["by"] == "path"
    assert len(data["results"]) == 3
    counts = [r["count"] for r in data["results"]]
    assert counts == sorted(counts, reverse=True)


def test_top_default_n(capsys):
    code = main(["top", FIXTURE, "--by", "ip"])
    out = capsys.readouterr()
    assert code == EXIT_OK
    assert out.out.strip()


def test_errors_json(capsys):
    code = main(["--format", "json", "errors", FIXTURE])
    out = capsys.readouterr()
    assert code == EXIT_OK
    data = json.loads(out.out)
    assert all(r["status"] >= 400 for r in data["results"])
    counts = [r["count"] for r in data["results"]]
    assert counts == sorted(counts, reverse=True)


def test_errors_since_filter(capsys):
    code = main(
        ["--format", "json", "errors", FIXTURE, "--since", "2026-07-12T14:00:00+00:00"]
    )
    out = capsys.readouterr()
    assert code == EXIT_OK
    data = json.loads(out.out)
    # only the 15:45 404 /products/42 error is after 14:00
    assert data["results"] == [{"status": 404, "path": "/products/42", "count": 1}]


def test_hourly_text(capsys):
    code = main(["hourly", FIXTURE])
    out = capsys.readouterr()
    assert code == EXIT_OK
    lines = out.out.strip().splitlines()
    assert len(lines) == 24
    assert lines[0].startswith("00")
    assert lines[23].startswith("23")


def test_hourly_json(capsys):
    main(["--format", "json", "hourly", FIXTURE])
    out = capsys.readouterr()
    data = json.loads(out.out)
    assert len(data["hourly"]) == 24
    assert sum(h["count"] for h in data["hourly"]) == 32


def test_missing_file(capsys):
    code = main(["summary", "/no/such/file.log"])
    out = capsys.readouterr()
    assert code == EXIT_FILE_ERROR
    assert out.out == ""
    assert "cannot read" in out.err


def test_no_valid_lines(tmp_path, capsys):
    bad = tmp_path / "bad.log"
    bad.write_text("garbage\nmore garbage\n")
    code = main(["summary", str(bad)])
    out = capsys.readouterr()
    assert code == EXIT_NO_VALID_LINES
    assert out.out == ""
    assert "no valid log lines" in out.err


def test_stdout_is_valid_json_only(capsys):
    # stdout must be a single valid JSON document with no stderr leakage
    main(["--format", "json", "summary", FIXTURE])
    out = capsys.readouterr()
    json.loads(out.out)  # raises if stdout is not pure JSON


def test_negative_n_rejected():
    with pytest.raises(SystemExit) as exc:
        main(["top", FIXTURE, "--by", "ip", "-n", "-1"])
    assert exc.value.code == 2
