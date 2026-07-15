# 02-atv-phoenix — acceptance: sol

| id | result | evidence |
|---|---|---|
| A1 | PASS | README installation followed by `uv run loglens --help` exited 0 and displayed all four subcommands. |
| A2 | PASS | Sealed text summary returned 40 total, 7 unique IPs, exact first/last timestamps, and 50.00%. |
| A3 | PASS | Entire JSON summary stdout parsed as one document and contained every expected A2 value. |
| A4 | PASS | Top paths were `/alpha 9`, `/beta 8`, `/charlie 7`, proving the ascending tie-break. |
| A5 | PASS | The `10:00:00`–`11:00:00` error window returned exactly the expected five grouped counts. |
| A6 | PASS | JSON hourly object contained all 24 hours and every bucket matched sealed ground truth. |
| A7 | PASS | Stderr reported exactly 3 malformed lines while stdout remained clean JSON. |
| A8 | PASS | Exit codes observed: missing file 2, malformed-only 1, valid fixture 0. |
| A9 | PASS | `uv run pytest -q` exited 0: 30 tests passed. |
| A10 | PASS | `uv run ruff check .` exited 0 with `All checks passed!`. |
| A11 | PASS | Sample fixture has 33 lines: 31 valid, 2 malformed, covering 2xx, 3xx, 4xx, and 5xx. |
| A12 | PASS | `just check` exited 0 and reported Ruff clean plus `30 passed`. |

**Acceptance passes:** 12/12  
**Correctness score:** 5/5
