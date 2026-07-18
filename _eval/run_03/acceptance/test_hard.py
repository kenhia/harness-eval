"""Hard tier — the robustness edges that actually discriminated run 1
(grader-observed, outside the old checklist), now mechanical. Failures
expected; report the count.

H4 implements precedent P1 mechanically: with the spec silent on window
bounds, EITHER half-open or closed passes — but the behavior must match
one convention exactly, and since-side inclusivity (shared by both
conventions) is required.
"""

from conftest import text_positions

import fixtures as fx


# H1 — mixed UTC offsets in one file: no crash, correct instants
def test_h1_mixed_offsets_summary(loglens, mixed_tz_log):
    r = loglens("summary", str(mixed_tz_log))
    assert r.rc == 0, f"mixed-offset file crashed summary: {r.err[-800:]}"
    assert "3" in r.out, f"3 valid lines expected in summary:\n{r.out}"


# H2 — offset-aware --since against offset-carrying logs (run 1's killer:
#      naive datetime compared to aware -> TypeError crash)
def test_h2_aware_window_filter(loglens, mixed_tz_log):
    r = loglens("errors", str(mixed_tz_log),
                "--since", "2026-07-12T06:00:00+00:00",
                "--until", "2026-07-12T09:00:00+00:00")
    assert r.rc == 0, f"aware/naive datetime crash: {r.err[-800:]}"
    # only the 404 (08:30Z) is an in-window error; the 500 is 10:15Z
    assert "404" in r.out, f"in-window 404 missing:\n{r.out}"
    assert "500" not in r.out, f"10:15Z 500 leaked into 06-09Z window:\n{r.out}"


# H3 — Z-suffix ISO 8601 accepted
def test_h3_z_suffix(loglens, mixed_tz_log):
    r = loglens("errors", str(mixed_tz_log),
                "--since", "2026-07-12T06:00:00Z", "--until", "2026-07-12T09:00:00Z")
    assert r.rc == 0, f"Z-suffix ISO 8601 rejected/crashed: {r.err[-500:]}"
    assert "404" in r.out and "500" not in r.out


# H4 — boundary semantics: P1 mechanized. since-side must include the
#      exact-boundary record; until-side may include or exclude, but the
#      result must match one convention exactly.
def test_h4_boundary_convention(loglens, boundary_log):
    r = loglens("errors", str(boundary_log),
                "--since", "2026-07-12T06:00:00+00:00",
                "--until", "2026-07-12T09:00:00+00:00")
    assert r.rc == 0, r.err[-500:]
    has = {p: p in r.out for p in ("/at-since", "/inside", "/at-until")}
    assert has["/inside"], f"mid-window record missing:\n{r.out}"
    assert has["/at-since"], (
        f"record exactly at --since excluded — both conventions include it:\n{r.out}"
    )
    # /at-until: either convention OK — nothing further to assert; H4's
    # value is that the two required memberships above hold simultaneously.


# H5 — hourly aggregates hour-of-day across days (not per-day buckets)
def test_h5_hourly_across_days(loglens, tmp_path):
    two_days = (
        fx.clf_line("203.0.113.7", "/a", 200, 10, 6, 0, day=12)
        + "\n"
        + fx.clf_line("203.0.113.7", "/a", 200, 10, 6, 30, day=13)
        + "\n"
    )
    f = tmp_path / "twodays.log"
    f.write_text(two_days)
    r = loglens("hourly", str(f))
    assert r.rc == 0, r.err[-500:]
    assert "2" in r.out, f"hour 06 should aggregate to 2 across days:\n{r.out}"


# H6 — windowed query matching nothing: clean empty result, exit 0
def test_h6_empty_window(loglens, logfile):
    r = loglens("errors", str(logfile),
                "--since", "1971-01-01T00:00:00+00:00",
                "--until", "1971-01-02T00:00:00+00:00")
    assert r.rc == 0, f"empty window must not error (rc={r.rc}): {r.err[-500:]}"
    assert "404" not in r.out and "500" not in r.out, f"phantom groups:\n{r.out}"


# H7 — JSON stays a single valid document under windowed/filtered queries
def test_h7_json_everywhere(loglens_json, logfile):
    for args in (
        ("errors", str(logfile), "--since", fx.WINDOW_SINCE, "--until", fx.WINDOW_UNTIL),
        ("top", str(logfile), "--by", "status"),
        ("hourly", str(logfile)),
    ):
        r = loglens_json(*args)
        assert r.rc == 0, f"{args[0]} json rc={r.rc}: {r.err[-300:]}"
        r.json()


# H8 — CLF dash-for-bytes: spec-silent, dual-accept (valid OR malformed),
#      but never a crash and stdout stays parseable
def test_h8_dash_bytes(loglens_json, tmp_path):
    f = tmp_path / "dash.log"
    f.write_text(fx.DASH_BYTES + fx.clf_line("192.0.2.9", "/b", 200, 10, 7, 0) + "\n")
    r = loglens_json("summary", str(f))
    assert r.rc == 0, f"dash-bytes line crashed summary: {r.err[-500:]}"
    doc = r.json()
    from conftest import has_number
    assert has_number(doc, 1) or has_number(doc, 2), (
        "dash-bytes line must be counted as valid (2 total) or malformed (1 total)"
    )


# H9 — NAIVE --since (no offset) against offset-carrying logs: run 1's
#      documented killer (README examples crashed on three of five
#      repos). Dual-accept: interpret the naive instant somehow (rc 0)
#      OR reject it with a clean usage error — but never a traceback.
def test_h9_naive_since(loglens, mixed_tz_log):
    r = loglens("errors", str(mixed_tz_log), "--since", "2026-07-12T06:00:00")
    assert "Traceback" not in r.err, (
        f"naive --since crashed with a traceback:\n{r.err[-800:]}"
    )
    assert r.rc in (0, 1, 2), f"unexpected exit {r.rc}"
    if r.rc != 0:
        assert r.err.strip(), "nonzero exit needs a user-facing message"
