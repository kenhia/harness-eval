# Acceptance checklist — SEALED (graders only, never shown to working agents)

Run by one grader per repo, results shared with both rubric graders as the
input to dimension 1. Work in the repo at `HEAD`; use a throwaway venv via
the project's own install instructions. For A2–A7, generate a **sealed
fixture** independently at grading time (a fresh CLF file ~40 lines with
known counts — write a tiny generator script, keep it and the expected values
in `_eval/grades/sealed-fixture/`), so agents cannot have overfit to it.

| id | check | pass criteria |
|---|---|---|
| A1 | install & entry point | Following README alone: `loglens --help` exits 0 |
| A2 | summary (text) | totals, unique IPs, first/last timestamp, error rate all match sealed-fixture ground truth |
| A3 | summary `--format json` | stdout is a single `json.loads`-able document containing the A2 values |
| A4 | top | `top --by path -n 3` correct order; tie between two equal-count paths broken by value ascending |
| A5 | errors + time window | `--since`/`--until` filter correctly; grouping is (status, path); most-frequent first |
| A6 | hourly | 24 buckets, counts match ground truth |
| A7 | malformed handling | malformed count on **stderr** only; stdout stays clean/parseable |
| A8 | exit codes | missing file → 2; file with only malformed lines → 1; success → 0 |
| A9 | tests | `pytest` passes from a clean checkout |
| A10 | lint | `ruff check` clean |
| A11 | fixture delivered | `tests/fixtures/sample.log` meets the spec's minimums (30+ lines, status spread, ≥2 malformed) |
| A12 | one-command check | `just check` (or the README-documented equivalent) runs A9+A10 |

Score for dimension 1: passes/12, mapped to 0–5 (12→5, 10–11→4, 8–9→3,
6–7→2, 3–5→1, <3→0). Record per-check pass/fail + evidence in
`_eval/grades/NN-acceptance.md`.
