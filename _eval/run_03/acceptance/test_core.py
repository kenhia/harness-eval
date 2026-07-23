"""Core tier — mechanical port of run_01's A1–A12. Every competent run
should pass all of these. Calibration target: the seven graded run_01
trees (all adjudicated 12/12) must pass this tier.
"""

import re
import subprocess

from conftest import CLI_TIMEOUT, REPO, clean_env, has_number, has_rate, text_positions

import fixtures as fx


# A1 — install & entry point
def test_a1_entrypoint(loglens):
    r = loglens("--help")
    assert r.rc == 0, f"loglens --help rc={r.rc}: {r.err[-500:]}"


# A2 — summary (text): headline values present
def test_a2_summary_text(loglens, logfile):
    r = loglens("summary", str(logfile))
    assert r.rc == 0, r.err[-500:]
    assert str(fx.TOTAL_VALID) in r.out, f"total {fx.TOTAL_VALID} not in summary:\n{r.out}"
    assert str(fx.UNIQUE_IPS) in r.out, f"unique-IP count {fx.UNIQUE_IPS} not in summary:\n{r.out}"
    # error rate in any common form: 20.0, 20%, 20.00%, 0.20
    pct = fx.ERROR_RATE_PCT
    forms = [f"{pct:.0f}", f"{pct:.1f}", f"{pct:.2f}", f"{pct/100:.2f}"]
    assert any(f in r.out for f in forms), f"error rate ~{pct} not found:\n{r.out}"


# A3 — summary --format json: single parseable doc carrying the A2 values
def test_a3_summary_json(loglens_json, logfile):
    r = loglens_json("summary", str(logfile))
    assert r.rc == 0, r.err[-500:]
    doc = r.json()
    assert has_number(doc, fx.TOTAL_VALID), "total requests missing from JSON"
    assert has_number(doc, fx.UNIQUE_IPS), "unique IPs missing from JSON"
    assert has_rate(doc, fx.ERROR_RATE_PCT), "error rate missing from JSON"


# A4 — top: order correct incl. the value-ascending tie-break
def test_a4_top_tiebreak(loglens, logfile):
    r = loglens("top", str(logfile), "--by", "path", "-n", "3")
    assert r.rc == 0, r.err[-500:]
    expected = [p for p, _ in fx.TOP3_PATHS]
    pos = text_positions(r.out, expected)
    assert all(p >= 0 for p in pos), f"missing top paths {expected} in:\n{r.out}"
    assert pos == sorted(pos), f"top order wrong; expected {expected} in order:\n{r.out}"


# A5 — errors + time window: grouping (status,path), most-frequent first,
#      window filters correctly (no boundary records in this probe — H4
#      covers boundaries under the P1 precedent)
def test_a5_errors_window(loglens, logfile):
    r = loglens("errors", str(logfile))
    assert r.rc == 0, r.err[-500:]
    (status, path), count = fx.TOP_ERROR_GROUP
    assert str(status) in r.out and path in r.out, f"top group {status} {path} missing:\n{r.out}"
    first_group_pos = r.out.find(str(status))
    for s, _p in fx.ERROR_GROUPS:
        if s != status and (opos := r.out.find(str(s))) >= 0:
            assert first_group_pos <= opos, f"most-frequent group not first:\n{r.out}"

    r = loglens("errors", str(logfile), "--since", fx.WINDOW_SINCE, "--until", fx.WINDOW_UNTIL)
    assert r.rc == 0, f"windowed errors failed: {r.err[-500:]}"
    assert "/api/x" in r.out and "/missing" in r.out, f"in-window groups missing:\n{r.out}"
    # the 23:55 500 and 11:30 404 are outside; their counts must shrink:
    # in-window: 404 /missing = 3, 500 /api/x = 2
    assert "3" in r.out and "2" in r.out, f"windowed counts absent:\n{r.out}"


# A6 — hourly: runs clean, 24-bucket shape or per-hour counts present
def test_a6_hourly(loglens, logfile):
    r = loglens("hourly", str(logfile))
    assert r.rc == 0, r.err[-500:]
    for hour, count in fx.HOURLY.items():
        assert re.search(rf"\b0?{hour}\b", r.out), f"hour {hour} missing from histogram:\n{r.out}"


# A7 — malformed accounting on stderr only; stdout stays parseable
def test_a7_malformed_stderr(loglens_json, logfile):
    r = loglens_json("summary", str(logfile))
    assert r.rc == 0
    r.json()  # stdout must remain a single valid JSON doc
    assert str(fx.N_MALFORMED) in r.err, (
        f"malformed count {fx.N_MALFORMED} not on stderr (stderr: {r.err[:300]!r})"
    )


# A8 — exit codes: 2 missing file, 1 only-malformed, 0 success
def test_a8_exit_codes(loglens, logfile, tmp_path):
    assert loglens("summary", str(logfile)).rc == 0
    r = loglens("summary", str(tmp_path / "nope.log"))
    assert r.rc == 2, f"missing file must exit 2, got {r.rc}"
    bad = tmp_path / "allbad.log"
    bad.write_text("garbage\nmore garbage\n")
    r = loglens("summary", str(bad))
    assert r.rc == 1, f"no-valid-lines must exit 1, got {r.rc}"


# A9 — repo's own tests pass
def test_a9_pytest(repo):
    r = subprocess.run(["uv", "run", "pytest", "-q"], cwd=repo,
                       capture_output=True, text=True, timeout=600, env=clean_env())
    assert r.returncode == 0, f"repo pytest failed:\n{r.stdout[-2000:]}"


# A10 — lint clean
def test_a10_ruff(repo):
    r = subprocess.run(["uv", "run", "ruff", "check", "."], cwd=repo,
                       capture_output=True, text=True, timeout=120, env=clean_env())
    assert r.returncode == 0, f"ruff check failed:\n{r.stdout[-2000:]}"


# A11 — delivered fixture meets spec minimums
def test_a11_fixture_delivered(repo):
    f = repo / "tests" / "fixtures" / "sample.log"
    assert f.is_file(), "tests/fixtures/sample.log missing"
    lines = f.read_text().splitlines()
    assert len(lines) >= 30, f"fixture has {len(lines)} lines (<30)"
    clf = re.compile(r'^\S+ \S+ \S+ \[([^]]+)\] "[^"]*" (\d{3}) \S+')

    def parses(line):
        m = clf.match(line)
        if not m:
            return None
        from datetime import datetime
        try:
            datetime.strptime(m.group(1), "%d/%b/%Y:%H:%M:%S %z")
        except ValueError:
            return None
        return m

    statuses = {m.group(2)[0] for line in lines if (m := parses(line))}
    n_malformed = sum(1 for line in lines if line.strip() and not parses(line))
    assert {"2", "4"} <= statuses, f"status spread missing 2xx/4xx (got {statuses})"
    assert n_malformed >= 2, f"fixture needs >=2 malformed lines (found {n_malformed})"


# A12 — one-command check
def test_a12_one_command_check(repo):
    # `just` itself accepts justfile/Justfile/.justfile — the check must
    # too (defect S2b: a capital-J repo was failed on the filename alone)
    jf = [p for p in ("justfile", "Justfile", ".justfile", "JUSTFILE")
          if (repo / p).is_file()]
    assert jf, "no justfile — README-documented equivalent requires manual review"
    r = subprocess.run(["just", "check"], cwd=repo,
                       capture_output=True, text=True, timeout=600, env=clean_env())
    assert r.returncode == 0, f"just check failed:\n{r.stdout[-1500:]}\n{r.stderr[-500:]}"
