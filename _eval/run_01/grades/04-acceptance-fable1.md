# 04-kprojects — acceptance (grader: fable1)

Clone: `/tmp/grade-fable1/04-kprojects` @ HEAD (67d70bd). Sealed fixture:
`_eval/grades/sealed-fixture-fable1/`.

| id | result | evidence |
|---|---|---|
| A1 | PASS | `uv sync` per README, `uv run loglens --help` → exit 0 |
| A2 | PASS | `summary sealed.log` → 37 / 6 IPs / 05:02:11 / 22:55:07 / 18.92% (+ "Error responses: 7") — all match |
| A3 | PASS | `--format json summary` → single parseable (indent=2) JSON doc, matching values + error_count 7 |
| A4 | PASS | `top --by path -n 3` → /index.html 10, /api/orders 8, **/about 6** (tie broken ascending, correct) |
| A5 | PASS | no window: 404/missing 4, 500/admin 2, 503/admin 1; window 07:00→14:00: 2/1/1, most-frequent first. Bonus: offset-less `--since 2026-07-14T07:00:00` works too (normalized to UTC) → 3/1/1, correct — the only repo besides 01 that doesn't crash on it |
| A6 | PASS | 24 buckets; 05:6 06:6 07:8 08:5 13:6 22:6, rest 0 |
| A7 | PASS | "skipped 3 malformed lines" on stderr only; stdout JSON parses (2>file verified) |
| A8 | PASS | missing → 2; malformed-only → 1; success → 0 |
| A9 | PASS | `uv run pytest` → 28 passed from clean clone |
| A10 | PASS | `uv run ruff check .` → "All checks passed!" |
| A11 | PASS | sample.log: 34 lines, 2 malformed, statuses 200/201/204/302/401/404/500/503, hours 06–16, multiple IPs (3xx coverage is a single 302 line — minimal but meets spec) |
| A12 | PASS | `just check` → ruff clean + 28 passed |

**Passes: 12/12 → correctness score 5/5**
