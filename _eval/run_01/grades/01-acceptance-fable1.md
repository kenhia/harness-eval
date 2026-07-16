# 01-atv-starterkit — acceptance (grader: fable1)

Clone: `/tmp/grade-fable1/01-atv-starterkit` @ HEAD (bc5b3bf). Sealed fixture:
`_eval/grades/sealed-fixture-fable1/` (37 valid + 3 malformed lines; see
`expected.md` for ground truth).

| id | result | evidence |
|---|---|---|
| A1 | PASS | README's `uv sync` then `uv run loglens --help` → exit 0 |
| A2 | PASS | `summary sealed.log` → 37 requests, 6 IPs, 2026-07-14T05:02:11+00:00 / 2026-07-14T22:55:07+00:00, 18.92% — all match ground truth |
| A3 | PASS | `--format json summary` → single `json.loads`-able doc with identical values (error_rate 18.92) |
| A4 | PASS | `top --by path -n 3` → /index.html 10, /api/orders 8, **/about 6** (6–6 tie vs /contact broken ascending, correct) |
| A5 | PASS | no window: 404/missing 4, 500/admin 2, 503/admin 1; window 07:00→14:00 → 404/missing 2, 500/admin 1, 503/admin 1; most-frequent first |
| A6 | PASS | `hourly` → 24 buckets; 05:6 06:6 07:8 08:5 13:6 22:6, rest 0 — matches |
| A7 | PASS | "skipped 3 malformed lines" on stderr only; JSON stdout parses clean (verified with 2>file redirect) |
| A8 | PASS | missing file → 2; malformed-only.log → 1; sealed.log → 0 |
| A9 | PASS | `uv run pytest` → 25 passed from clean clone |
| A10 | PASS | `uv run ruff check .` → "All checks passed!" |
| A11 | PASS | sample.log: 32 lines, 2 malformed, statuses 2xx/3xx/4xx/5xx (200/201/204, 302/304, 401/403/404, 500/502), hours 06–13, 7 IPs |
| A12 | PASS | `just check` runs ruff + pytest (25 passed); justfile committed, README documents non-just equivalent |

**Passes: 12/12 → correctness score 5/5**
