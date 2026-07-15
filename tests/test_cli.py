"""CLI tests: exit codes, stream discipline, and output contracts."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

import pytest

from loglens.cli import EXIT_INPUT_ERROR, EXIT_NO_VALID_LINES, EXIT_OK, main


def run(capsys, *argv: str) -> tuple[int, str, str]:
    """Invoke the CLI in-process and capture (code, stdout, stderr)."""
    code = main(list(argv))
    captured = capsys.readouterr()
    return code, captured.out, captured.err


class TestExitCodes:
    def test_0_on_success(self, capsys, sample_log: str) -> None:
        code, out, _ = run(capsys, "summary", sample_log)
        assert code == EXIT_OK
        assert out.strip()

    def test_0_when_filter_matches_nothing(self, capsys, sample_log: str) -> None:
        """A valid file whose window matches nothing is a successful answer, not error 1."""
        code, out, _ = run(capsys, "errors", sample_log, "--since", "2030-01-01")
        assert code == EXIT_OK
        assert "No 4xx/5xx requests found." in out

    def test_1_on_empty_file(self, capsys, empty_log: str) -> None:
        code, out, err = run(capsys, "summary", empty_log)
        assert code == EXIT_NO_VALID_LINES
        assert out == "", "stdout must stay empty on exit 1"
        assert "empty" in err

    def test_1_when_no_line_is_clf(self, capsys, garbage_log: str) -> None:
        code, out, err = run(capsys, "summary", garbage_log)
        assert code == EXIT_NO_VALID_LINES
        assert out == ""
        assert "no valid log lines" in err

    def test_2_on_missing_file(self, capsys, tmp_path: Path) -> None:
        code, out, err = run(capsys, "summary", str(tmp_path / "nope.log"))
        assert code == EXIT_INPUT_ERROR
        assert out == ""
        assert "loglens:" in err

    def test_2_on_directory(self, capsys, tmp_path: Path) -> None:
        """A directory is present and readable, so it must not fall through to a crash."""
        code, _, err = run(capsys, "summary", str(tmp_path))
        assert code == EXIT_INPUT_ERROR
        assert "directory" in err

    @pytest.mark.skipif(os.geteuid() == 0, reason="root can read mode-000 files")
    def test_2_on_permission_denied(self, capsys, tmp_path: Path) -> None:
        path = tmp_path / "secret.log"
        path.write_text("x\n")
        path.chmod(0o000)
        try:
            code, _, err = run(capsys, "summary", str(path))
            assert code == EXIT_INPUT_ERROR
            assert "loglens:" in err
        finally:
            path.chmod(0o644)

    def test_2_on_usage_error(self, sample_log: str) -> None:
        with pytest.raises(SystemExit) as exc:
            main(["top", sample_log])  # --by is required
        assert exc.value.code == EXIT_INPUT_ERROR

    def test_2_on_unknown_subcommand(self) -> None:
        with pytest.raises(SystemExit) as exc:
            main(["frobnicate", "x.log"])
        assert exc.value.code == EXIT_INPUT_ERROR

    def test_2_on_bad_since_value(self, sample_log: str) -> None:
        """Rejected at the door, not as a TypeError deep in the comparison."""
        with pytest.raises(SystemExit) as exc:
            main(["errors", sample_log, "--since", "yesterday"])
        assert exc.value.code == EXIT_INPUT_ERROR

    def test_2_on_negative_n(self, sample_log: str) -> None:
        """`most_common(-5)` would return the entire list -- the opposite of the ask."""
        with pytest.raises(SystemExit) as exc:
            main(["top", sample_log, "--by", "ip", "-n", "-5"])
        assert exc.value.code == EXIT_INPUT_ERROR

    def test_2_on_inverted_window(self, sample_log: str) -> None:
        with pytest.raises(SystemExit) as exc:
            main(["errors", sample_log, "--since", "2026-07-13", "--until", "2026-07-12"])
        assert exc.value.code == EXIT_INPUT_ERROR


class TestStreamDiscipline:
    def test_malformed_count_on_stderr_never_stdout(self, capsys, sample_log: str) -> None:
        code, out, err = run(capsys, "summary", sample_log)
        assert code == EXIT_OK
        assert "malformed" in err
        assert "malformed" not in out
        assert "skipped" not in out

    def test_stdout_stays_valid_json_despite_malformed_lines(self, capsys, sample_log: str) -> None:
        """The test that actually enforces the spec's stderr rule."""
        code, out, err = run(capsys, "--format", "json", "summary", sample_log)
        assert code == EXIT_OK
        assert "malformed" in err, "the fixture must contain malformed lines for this to bite"
        json.loads(out)  # raises if the count leaked onto stdout

    def test_reports_first_offending_line(self, capsys, sample_log: str) -> None:
        _, _, err = run(capsys, "summary", sample_log)
        assert "first at line" in err

    def test_silent_when_nothing_is_malformed(self, capsys, tmp_path: Path) -> None:
        """A nightly cron must not email just because the tool ran."""
        clean = tmp_path / "clean.log"
        clean.write_text(
            '203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /a HTTP/1.1" 200 1 "-" "-"\n'
        )
        _, _, err = run(capsys, "summary", str(clean))
        assert err == ""


class TestFormatOption:
    def test_accepted_before_the_subcommand(self, capsys, sample_log: str) -> None:
        code, out, _ = run(capsys, "--format", "json", "summary", sample_log)
        assert code == EXIT_OK
        assert json.loads(out)["total_requests"] > 0

    def test_accepted_after_the_subcommand(self, capsys, sample_log: str) -> None:
        """The form everyone types, by muscle memory from `kubectl -o json`."""
        code, out, _ = run(capsys, "summary", sample_log, "--format", "json")
        assert code == EXIT_OK
        assert json.loads(out)["total_requests"] > 0

    def test_defaults_to_text(self, capsys, sample_log: str) -> None:
        _, out, _ = run(capsys, "summary", sample_log)
        assert "Total requests:" in out
        with pytest.raises(json.JSONDecodeError):
            json.loads(out)

    @pytest.mark.parametrize(
        "argv",
        [
            ["summary"],
            ["top", "--by", "ip"],
            ["errors"],
            ["hourly"],
        ],
    )
    def test_every_subcommand_emits_one_json_document(
        self, capsys, sample_log: str, argv: list[str]
    ) -> None:
        code, out, _ = run(capsys, "--format", "json", *argv, sample_log)
        assert code == EXIT_OK
        assert isinstance(json.loads(out), dict)


class TestSummaryCommand:
    def test_text_fields(self, capsys, sample_log: str) -> None:
        _, out, _ = run(capsys, "summary", sample_log)
        for label in ("Total requests:", "Unique IPs:", "First request:", "Last request:"):
            assert label in out
        assert "Error rate:" in out and "%" in out

    def test_json_shape(self, capsys, sample_log: str) -> None:
        _, out, _ = run(capsys, "--format", "json", "summary", sample_log)
        data = json.loads(out)
        assert set(data) == {"total_requests", "unique_ips", "first", "last", "error_rate"}
        assert isinstance(data["error_rate"], float)
        assert data["first"].endswith("+00:00"), "ISO8601 must carry the offset"


class TestTopCommand:
    def test_default_n_is_10(self, capsys, sample_log: str) -> None:
        _, out, _ = run(capsys, "--format", "json", "top", sample_log, "--by", "path")
        assert len(json.loads(out)["items"]) <= 10

    def test_n_respected(self, capsys, sample_log: str) -> None:
        _, out, _ = run(capsys, "--format", "json", "top", sample_log, "--by", "ip", "-n", "2")
        assert len(json.loads(out)["items"]) == 2

    def test_status_is_a_json_number(self, capsys, sample_log: str) -> None:
        _, out, _ = run(capsys, "--format", "json", "top", sample_log, "--by", "status")
        assert all(isinstance(i["value"], int) for i in json.loads(out)["items"])

    def test_by_is_required(self, sample_log: str) -> None:
        with pytest.raises(SystemExit):
            main(["top", sample_log])

    def test_rejects_unknown_dimension(self, sample_log: str) -> None:
        with pytest.raises(SystemExit):
            main(["top", sample_log, "--by", "referer"])


class TestErrorsCommand:
    def test_only_errors_listed(self, capsys, sample_log: str) -> None:
        _, out, _ = run(capsys, "--format", "json", "errors", sample_log)
        assert all(g["status"] >= 400 for g in json.loads(out)["groups"])

    def test_since_until_narrow_the_window(self, capsys, sample_log: str) -> None:
        _, wide, _ = run(capsys, "--format", "json", "errors", sample_log)
        _, narrow, _ = run(
            capsys,
            "--format",
            "json",
            "errors",
            sample_log,
            "--since",
            "2026-07-12T08:00:00Z",
            "--until",
            "2026-07-12T10:00:00Z",
        )
        assert json.loads(narrow)["total_errors"] < json.loads(wide)["total_errors"]

    def test_naive_since_does_not_crash(self, capsys, sample_log: str) -> None:
        """Comparing naive to aware raises TypeError; this is the regression guard."""
        code, out, _ = run(
            capsys, "--format", "json", "errors", sample_log, "--since", "2026-07-12"
        )
        assert code == EXIT_OK
        assert json.loads(out)["total_errors"] > 0


class TestHourlyCommand:
    def test_all_24_hours_rendered(self, capsys, sample_log: str) -> None:
        _, out, _ = run(capsys, "hourly", sample_log)
        for hour in range(24):
            assert f"{hour:02d}" in out

    def test_histogram_bars_present(self, capsys, sample_log: str) -> None:
        _, out, _ = run(capsys, "hourly", sample_log)
        assert "#" in out

    def test_json_has_24_buckets(self, capsys, sample_log: str) -> None:
        _, out, _ = run(capsys, "--format", "json", "hourly", sample_log)
        assert [h["hour"] for h in json.loads(out)["hours"]] == list(range(24))


class TestHostileInput:
    def test_hostile_log_mostly_parses(self, capsys, hostile_log: str) -> None:
        code, out, _ = run(capsys, "--format", "json", "summary", hostile_log)
        assert code == EXIT_OK
        data = json.loads(out)
        # Only the `- ` status line and the truncated final line should be rejected.
        assert data["total_requests"] == 17

    def test_ansi_escape_is_neutralized(self, capsys, hostile_log: str) -> None:
        _, out, _ = run(capsys, "top", hostile_log, "--by", "path", "-n", "50")
        assert "\x1b" not in out, "a logged escape sequence must not reach the terminal raw"
        assert "\\x1b" in out

    def test_ansi_escape_neutralized_in_json_too(self, capsys, hostile_log: str) -> None:
        _, out, _ = run(capsys, "--format", "json", "top", hostile_log, "--by", "path", "-n", "50")
        assert "\x1b" not in json.dumps(json.loads(out))

    def test_mixed_offsets_bucket_by_local_wall_clock(self, capsys, hostile_log: str) -> None:
        _, out, _ = run(capsys, "--format", "json", "hourly", hostile_log)
        hours = {h["hour"]: h["count"] for h in json.loads(out)["hours"]}
        assert hours[23] == 1, "the +0530 line is 23:00 as written"
        assert hours[6] >= 1

    def test_invalid_utf8_does_not_crash(self, capsys, hostile_log: str) -> None:
        code, _, _ = run(capsys, "top", hostile_log, "--by", "ip")
        assert code == EXIT_OK


class TestSubprocessBehavior:
    """End-to-end checks that need a real process."""

    def test_module_entry_point(self, sample_log: str) -> None:
        result = subprocess.run(
            [sys.executable, "-m", "loglens", "summary", sample_log],
            capture_output=True,
            text=True,
        )
        assert result.returncode == EXIT_OK
        assert "Total requests:" in result.stdout

    def test_broken_pipe_is_clean(self, sample_log: str) -> None:
        """`loglens hourly f.log | head -1` must not print a traceback."""
        proc = subprocess.run(
            f"{sys.executable} -m loglens hourly {sample_log} | head -1",
            shell=True,
            capture_output=True,
            text=True,
        )
        assert "BrokenPipeError" not in proc.stderr
        assert "Traceback" not in proc.stderr

    def test_version_flag(self) -> None:
        result = subprocess.run(
            [sys.executable, "-m", "loglens", "--version"], capture_output=True, text=True
        )
        assert result.returncode == 0
        assert "loglens" in result.stdout
