# 03-working-skill-repo — acceptance (grader: fable1)

Clone: `/tmp/grade-fable1/03-working-skill-repo` @ HEAD (0a8337b). Sealed
fixture: `_eval/grades/sealed-fixture-fable1/`.

| id | result | evidence |
|---|---|---|
| A1 | PASS | `uv sync` per README, `uv run loglens --help` → exit 0 |
| A2 | PASS | `summary sealed.log` → 37 / 6 IPs / 05:02:11 / 22:55:07 / 18.92% (also reports "7 errors") — all match |
| A3 | PASS | `--format json summary` → single parseable (pretty-printed) JSON doc with matching values + bonus error_count 7 |
| A4 | PASS | `top --by path -n 3` → /index.html 10, /api/orders 8, **/about 6** (tie broken ascending, correct) |
| A5 | PASS | no window: 404/missing 4, 500/admin 2, 503/admin 1; window 07:00+00:00→14:00+00:00: 2/1/1, most-frequent first. *Footnote: offset-less `--since 2026-07-14T07:00:00` crashes with a naive-vs-aware `TypeError` traceback (same latent hole as run 02); README only documents offset-aware forms, and the offset forms tested here are correct, so the check passes* |
| A6 | PASS | 24 buckets; 05:6 06:6 07:8 08:5 13:6 22:6, rest 0 |
| A7 | PASS | "skipped 3 malformed lines" on stderr only; stdout JSON parses (2>file verified) |
| A8 | PASS | missing → 2; malformed-only → 1; success → 0 |
| A9 | PASS | `uv run pytest` → 27 passed from clean clone |
| A10 | PASS | `uv run ruff check .` → "All checks passed!" (agent excluded harness-owned `.github/` via `extend-exclude` — reasonable resolution of the setup tension noted in the run log) |
| A11 | PASS | sample.log: 34 lines, 2 malformed, statuses 200/201/204/301/302/403/404/500/502/503, hours 06–15, 8 IPs |
| A12 | PASS | `just check` → ruff clean + 27 passed |

**Passes: 12/12 → correctness score 5/5**
