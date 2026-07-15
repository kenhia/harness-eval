# 04-kprojects — acceptance: sol

| id | result | evidence |
|---|---|---|
| A1 | PASS | `uv sync && uv run loglens --help` exited 0 and exposed `summary`, `top`, `errors`, and `hourly`. |
| A2 | PASS | Text summary reported 40 requests, 7 IPs, expected first/last timestamps, and 50.00% error rate. |
| A3 | PASS | JSON summary was one parseable object containing the A2 values (`40`, `7`, expected timestamps, `50.0`). |
| A4 | PASS | Path top-three output was `/alpha 9`, `/beta 8`, `/charlie 7`, correctly resolving the 7-count tie. |
| A5 | PASS | Inclusive window output matched all five expected `(status,path,count)` groups and excluded adjacent boundary records. |
| A6 | PASS | JSON `hourly` contained 24 entries and exactly matched all sealed-fixture bucket counts. |
| A7 | PASS | JSON stdout parsed as a single document; only stderr contained `skipped 3 malformed lines`. |
| A8 | PASS | Observed exit codes were 2 for missing, 1 for malformed-only, and 0 for success. |
| A9 | PASS | `uv run pytest -q` exited 0: `28 passed`. |
| A10 | PASS | `uv run ruff check .` exited 0: `All checks passed!`. |
| A11 | PASS | Delivered fixture has 34 lines: 32 valid, 2 malformed, with all 2xx–5xx classes represented. |
| A12 | PASS | `just check` exited 0 after running Ruff and all 28 tests. |

**Acceptance passes:** 12/12  
**Correctness score:** 5/5
