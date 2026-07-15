import json
from pathlib import Path

import pytest

from loglens.cli import main

FIXTURE = str(Path(__file__).parent / "fixtures" / "sample.log")


def test_summary_text(capsys):
    code = main(["summary", FIXTURE])
    out = capsys.readouterr()
    assert code == 0
    assert "Total requests:" in out.out
    assert "Error rate:" in out.out
    # malformed count goes to stderr, never stdout
    assert "malformed" in out.err
    assert "malformed" not in out.out


def test_summary_json_single_document(capsys):
    code = main(["--format", "json", "summary", FIXTURE])
    out = capsys.readouterr()
    assert code == 0
    doc = json.loads(out.out)  # raises if not a single valid JSON document
    assert doc["total_requests"] > 0
    assert set(doc) == {
        "total_requests",
        "unique_ips",
        "first_timestamp",
        "last_timestamp",
        "error_rate",
    }


def test_format_after_subcommand(capsys):
    code = main(["summary", FIXTURE, "--format", "json"])
    out = capsys.readouterr()
    assert code == 0
    json.loads(out.out)


def test_top_by_ip_limit(capsys):
    code = main(["top", FIXTURE, "--by", "ip", "-n", "2"])
    out = capsys.readouterr()
    assert code == 0
    assert len(out.out.strip().splitlines()) == 2


def test_top_json(capsys):
    code = main(["--format", "json", "top", FIXTURE, "--by", "status"])
    out = capsys.readouterr()
    assert code == 0
    doc = json.loads(out.out)
    assert doc["by"] == "status"
    assert doc["results"][0]["count"] >= doc["results"][-1]["count"]


def test_errors_filtered(capsys):
    code = main(
        [
            "errors",
            FIXTURE,
            "--since",
            "2026-07-12T09:00:00+00:00",
            "--until",
            "2026-07-12T10:00:00+00:00",
        ]
    )
    out = capsys.readouterr()
    assert code == 0
    # 403 /admin and /admin/users fall in this window
    assert "403" in out.out


def test_hourly_histogram(capsys):
    code = main(["hourly", FIXTURE])
    out = capsys.readouterr()
    assert code == 0
    lines = out.out.strip().splitlines()
    assert len(lines) == 24
    assert lines[0].startswith("00")
    assert lines[23].startswith("23")


def test_missing_file_exit_2(capsys):
    code = main(["summary", "/no/such/file.log"])
    out = capsys.readouterr()
    assert code == 2
    assert out.out == ""
    assert "cannot read" in out.err


def test_no_valid_lines_exit_1(tmp_path, capsys):
    bad = tmp_path / "bad.log"
    bad.write_text("garbage\nmore garbage\n")
    code = main(["summary", str(bad)])
    out = capsys.readouterr()
    assert code == 1
    assert out.out == ""
    assert "no valid log lines" in out.err
    assert "malformed" in out.err


def test_top_requires_by():
    with pytest.raises(SystemExit):
        main(["top", FIXTURE])


def test_invalid_since_exit_2(capsys):
    code = main(["errors", FIXTURE, "--since", "not-a-date"])
    out = capsys.readouterr()
    assert code == 2
    assert out.out == ""
    assert "invalid ISO 8601 timestamp" in out.err
