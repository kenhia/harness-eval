import json

import pytest

from loglens.cli import main

VALID = '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1" 200 5413 "-" "-"'


def run(capsys, *argv):
    """Invoke the CLI, returning (exit_code, stdout, stderr)."""
    code = main(list(argv))
    captured = capsys.readouterr()
    return code, captured.out, captured.err


# --- exit codes ---------------------------------------------------------


def test_success_exits_zero(capsys, sample_log):
    code, out, _ = run(capsys, "summary", sample_log)
    assert code == 0
    assert "Total requests:  32" in out


def test_missing_file_exits_two(capsys, tmp_path):
    code, out, err = run(capsys, "summary", str(tmp_path / "nope.log"))
    assert code == 2
    assert out == ""
    assert "cannot read" in err


def test_unreadable_file_exits_two(capsys, tmp_path):
    log = tmp_path / "denied.log"
    log.write_text(VALID + "\n")
    log.chmod(0o000)
    try:
        code, out, err = run(capsys, "summary", str(log))
    finally:
        log.chmod(0o644)
    assert code == 2
    assert out == ""
    assert "cannot read" in err


def test_directory_as_logfile_exits_two(capsys, tmp_path):
    code, _, err = run(capsys, "summary", str(tmp_path))
    assert code == 2
    assert "cannot read" in err


def test_file_with_no_valid_lines_exits_one(capsys, tmp_path):
    log = tmp_path / "junk.log"
    log.write_text("garbage\nmore garbage\n")
    code, out, err = run(capsys, "summary", str(log))
    assert code == 1
    assert out == ""
    assert "no valid log lines" in err


def test_empty_file_exits_one(capsys, tmp_path):
    log = tmp_path / "empty.log"
    log.write_text("")
    code, _, err = run(capsys, "summary", str(log))
    assert code == 1
    assert "no valid log lines" in err


# --- malformed reporting ------------------------------------------------


def test_malformed_count_goes_to_stderr_not_stdout(capsys, sample_log):
    _, out, err = run(capsys, "summary", sample_log)
    assert "skipped 2 malformed lines" in err
    assert "malformed" not in out


def test_malformed_count_absent_when_file_is_clean(capsys, tmp_path):
    log = tmp_path / "clean.log"
    log.write_text(VALID + "\n")
    _, _, err = run(capsys, "summary", str(log))
    assert err == ""


def test_malformed_message_singular(capsys, tmp_path):
    log = tmp_path / "one_bad.log"
    log.write_text(f"{VALID}\nbroken\n")
    _, _, err = run(capsys, "summary", str(log))
    assert "skipped 1 malformed line" in err
    assert "lines" not in err


def test_stdout_stays_valid_json_despite_malformed_lines(capsys, sample_log):
    _, out, err = run(capsys, "--format", "json", "summary", sample_log)
    assert "malformed" in err
    json.loads(out)


# --- output shape -------------------------------------------------------


def test_all_subcommands_emit_a_single_json_document(capsys, sample_log):
    for argv in (
        ["summary", sample_log],
        ["top", sample_log, "--by", "ip"],
        ["errors", sample_log],
        ["hourly", sample_log],
    ):
        _, out, _ = run(capsys, "--format", *["json"], *argv)
        assert json.loads(out) is not None


def test_format_accepted_after_subcommand(capsys, sample_log):
    _, out, _ = run(capsys, "summary", sample_log, "--format", "json")
    assert json.loads(out)["total_requests"] == 32


def test_text_is_the_default_format(capsys, sample_log):
    _, out, _ = run(capsys, "summary", sample_log)
    with pytest.raises(json.JSONDecodeError):
        json.loads(out)


def test_summary_json_fields(capsys, sample_log):
    _, out, _ = run(capsys, "--format", "json", "summary", sample_log)
    payload = json.loads(out)
    assert payload == {
        "total_requests": 32,
        "unique_ips": 5,
        "first_timestamp": "2026-07-12T06:25:24+00:00",
        "last_timestamp": "2026-07-12T23:58:47+00:00",
        "errors": 10,
        "error_rate": 31.25,
    }


def test_top_respects_n(capsys, sample_log):
    _, out, _ = run(capsys, "--format", "json", "top", sample_log, "--by", "path", "-n", "2")
    assert len(json.loads(out)["results"]) == 2


def test_top_defaults_to_ten(capsys, sample_log):
    _, out, _ = run(capsys, "--format", "json", "top", sample_log, "--by", "ip")
    payload = json.loads(out)
    assert len(payload["results"]) == 5  # fewer than 10 distinct IPs exist
    assert payload["by"] == "ip"


def test_top_text_header_fits_the_column(capsys, sample_log):
    _, out, _ = run(capsys, "top", sample_log, "--by", "status", "-n", "2")
    header, first = out.splitlines()[:2]
    assert header.startswith("STATUS")
    assert first.startswith("200")


def test_top_requires_by(capsys, sample_log):
    with pytest.raises(SystemExit) as exc:
        main(["top", sample_log])
    assert exc.value.code == 2


def test_top_rejects_unknown_by(capsys, sample_log):
    with pytest.raises(SystemExit) as exc:
        main(["top", sample_log, "--by", "referer"])
    assert exc.value.code == 2


def test_top_rejects_non_positive_n(capsys, sample_log):
    with pytest.raises(SystemExit) as exc:
        main(["top", sample_log, "--by", "ip", "-n", "0"])
    assert exc.value.code == 2


def test_errors_json_sorted_by_count(capsys, sample_log):
    _, out, _ = run(capsys, "--format", "json", "errors", sample_log)
    results = json.loads(out)["results"]
    counts = [row["count"] for row in results]
    assert counts == sorted(counts, reverse=True)
    assert results[0] == {"status": 404, "path": "/missing", "count": 4}


def test_errors_since_until_narrow_results(capsys, sample_log):
    _, full, _ = run(capsys, "--format", "json", "errors", sample_log)
    _, windowed, _ = run(
        capsys,
        "--format",
        "json",
        "errors",
        sample_log,
        "--since",
        "2026-07-12T12:00:00",
        "--until",
        "2026-07-12T23:59:59",
    )
    total = sum(r["count"] for r in json.loads(full)["results"])
    subset = sum(r["count"] for r in json.loads(windowed)["results"])
    assert 0 < subset < total


def test_errors_rejects_bad_iso8601(capsys, sample_log):
    with pytest.raises(SystemExit) as exc:
        main(["errors", sample_log, "--since", "yesterday"])
    assert exc.value.code == 2


def test_errors_text_when_no_errors_in_window(capsys, sample_log):
    _, out, _ = run(capsys, "errors", sample_log, "--since", "2030-01-01T00:00:00")
    assert "No error responses found." in out


def test_errors_json_empty_when_no_errors_in_window(capsys, sample_log):
    _, out, _ = run(capsys, "--format", "json", "errors", sample_log, "--since", "2030-01-01")
    assert json.loads(out) == {"results": []}


def test_hourly_renders_24_rows_with_bars(capsys, sample_log):
    _, out, _ = run(capsys, "hourly", sample_log)
    lines = out.splitlines()
    assert len(lines) == 24
    assert lines[0].startswith("00")
    assert lines[23].startswith("23")
    assert "#" in out


def test_hourly_has_no_trailing_whitespace(capsys, sample_log):
    _, out, _ = run(capsys, "hourly", sample_log)
    assert all(line == line.rstrip() for line in out.splitlines())


def test_hourly_json_covers_every_hour(capsys, sample_log):
    _, out, _ = run(capsys, "--format", "json", "hourly", sample_log)
    hours = json.loads(out)["hourly"]
    assert [row["hour"] for row in hours] == list(range(24))
    assert sum(row["count"] for row in hours) == 32


def test_no_subcommand_exits_two(capsys):
    with pytest.raises(SystemExit) as exc:
        main([])
    assert exc.value.code == 2
