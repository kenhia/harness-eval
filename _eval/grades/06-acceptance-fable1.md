# 06-gstack — acceptance (grader: fable1)

Clone: `/tmp/grade-fable1/06-gstack` @ HEAD (0886dc4). Sealed fixture:
`_eval/grades/sealed-fixture-fable1/` (37 valid + 3 malformed lines; see
`expected.md` for ground truth). Runner covariate noted; acceptance checks
are runner-independent.

| id | result | evidence |
|---|---|---|
| A1 | PASS | README's clone path: `uv sync` then `uv run loglens --help` → exit 0 (the README's *first* install path is a placeholder GitHub URL that doesn't exist — noted under docs, not here, since the documented from-a-clone path works) |
| A2 | PASS | `summary sealed.log` → 37 requests, 6 IPs, 2026-07-14T05:02:11+00:00 / 2026-07-14T22:55:07+00:00, 18.92% — all match ground truth |
| A3 | PASS | `--format json summary` → single `json.loads`-able doc with identical values (error_rate 18.92) |
| A4 | PASS | `top --by path -n 3` → /index.html 10, /api/orders 8, **/about 6** (6–6 tie vs /contact broken ascending, correct). Bonus: `--by ip -n 3` also breaks the designed 7–7 IP tie ascending |
| A5 | PASS | no window: 404/missing 4, 500/admin 2, 503/admin 1; window 07:00→14:00 UTC → 404/missing 2, 500/admin 1, 503/admin 1; most-frequent first (half-open bounds, documented; no fixture record sits on a boundary) |
| A6 | PASS | `hourly` → 24 buckets; 05:6 06:6 07:8 08:5 13:6 22:6, rest 0 — matches |
| A7 | PASS | "skipped 3 of 40 malformed lines (first at line 11 …)" on stderr only; JSON stdout parses clean (verified via 2>file redirect) |
| A8 | PASS | missing file → 2; malformed-only.log → 1 with empty stdout; sealed.log → 0 |
| A9 | PASS | `uv run pytest` from clean clone → 137 passed, 1 skipped (locale test, guarded skip on hosts without de_DE.UTF-8 — legitimate) |
| A10 | PASS | `uv run ruff check .` → "All checks passed!" |
| A11 | PASS | sample.log: 36 lines, 2 malformed, statuses 2xx (200/201), 3xx (304/301), 4xx (404/403), 5xx (500/502), hours 06–14 (9 distinct), 4 IPs |
| A12 | PASS | `just check` runs ruff check + ruff format --check + pytest + a JSON smoke test over all four subcommands; README documents the non-just equivalent |

**Passes: 12/12 → correctness score 5/5**
