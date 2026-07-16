import json
import subprocess
import sys
from pathlib import Path

FIXTURE = Path(__file__).parent / "fixtures" / "sample.log"


def run(*args, input_path=None):
    """Invoke the CLI as a subprocess, returning CompletedProcess."""
    cmd = [sys.executable, "-m", "loglens.cli", *args]
    return subprocess.run(cmd, capture_output=True, text=True)


# --------------------------------------------------------------------------- #
# Exit codes
# --------------------------------------------------------------------------- #


def test_exit_ok():
    r = run("summary", str(FIXTURE))
    assert r.returncode == 0
    assert r.stdout.strip() != ""


def test_exit_missing_file():
    r = run("summary", "/no/such/file.log")
    assert r.returncode == 2
    assert r.stdout == ""
    assert "cannot read" in r.stderr


def test_exit_no_valid_lines(tmp_path):
    bad = tmp_path / "bad.log"
    bad.write_text("garbage\nmore garbage\n")
    r = run("summary", str(bad))
    assert r.returncode == 1
    assert r.stdout == ""
    assert "no valid log lines" in r.stderr


# --------------------------------------------------------------------------- #
# Malformed handling: count on stderr, never stdout
# --------------------------------------------------------------------------- #


def test_malformed_reported_on_stderr_only():
    r = run("summary", str(FIXTURE))
    assert r.returncode == 0
    assert "malformed" in r.stderr
    assert "malformed" not in r.stdout


def test_malformed_absent_when_clean(tmp_path):
    clean = tmp_path / "clean.log"
    clean.write_text(
        '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] '
        '"GET /a HTTP/1.1" 200 10 "-" "UA"\n'
    )
    r = run("summary", str(clean))
    assert r.returncode == 0
    assert "malformed" not in r.stderr


# --------------------------------------------------------------------------- #
# JSON output is a single valid document on stdout
# --------------------------------------------------------------------------- #


def test_summary_json():
    r = run("--format", "json", "summary", str(FIXTURE))
    assert r.returncode == 0
    data = json.loads(r.stdout)  # must not raise
    assert data["unique_ips"] == 5
    assert "error_rate" in data
    assert data["first_timestamp"].startswith("2026-07-12T06:25:24")


def test_top_json_and_ordering():
    r = run("--format", "json", "top", str(FIXTURE), "--by", "ip", "-n", "3")
    assert r.returncode == 0
    data = json.loads(r.stdout)
    assert data["by"] == "ip"
    counts = [row["count"] for row in data["results"]]
    assert counts == sorted(counts, reverse=True)
    assert len(data["results"]) == 3


def test_top_default_n_is_10():
    r = run("--format", "json", "top", str(FIXTURE), "--by", "path")
    data = json.loads(r.stdout)
    assert len(data["results"]) <= 10


def test_errors_json_most_frequent_first():
    r = run("--format", "json", "errors", str(FIXTURE))
    data = json.loads(r.stdout)
    counts = [row["count"] for row in data["results"]]
    assert counts == sorted(counts, reverse=True)
    top_err = data["results"][0]
    assert top_err["status"] == 404
    assert top_err["path"] == "/missing"


def test_errors_since_filter():
    r = run(
        "--format",
        "json",
        "errors",
        str(FIXTURE),
        "--since",
        "2026-07-12T10:00:00+00:00",
    )
    data = json.loads(r.stdout)
    keys = {(row["status"], row["path"]) for row in data["results"]}
    assert (502, "/api/login") in keys
    assert (500, "/api/login") not in keys


def test_hourly_json_has_24_hours():
    r = run("--format", "json", "hourly", str(FIXTURE))
    data = json.loads(r.stdout)
    assert len(data) == 24
    assert set(data.keys()) == {f"{h:02d}" for h in range(24)}
    assert data["06"] > 0
    assert data["00"] == 0


# --------------------------------------------------------------------------- #
# Text output smoke checks
# --------------------------------------------------------------------------- #


def test_hourly_text_histogram():
    r = run("hourly", str(FIXTURE))
    assert r.returncode == 0
    assert "#" in r.stdout
    # 24 lines, one per hour
    assert len([ln for ln in r.stdout.splitlines() if ln.strip()]) == 24


def test_summary_text():
    r = run("summary", str(FIXTURE))
    assert "Total requests:" in r.stdout
    assert "Error rate:" in r.stdout


def test_invalid_since_exit(tmp_path):
    r = run("errors", str(FIXTURE), "--since", "not-a-date")
    assert r.returncode != 0
    assert "invalid ISO 8601" in r.stderr
