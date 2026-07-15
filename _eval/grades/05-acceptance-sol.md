# 05-baseline — acceptance: sol

| id | result | evidence |
|---|---|---|
| A1 | PASS | `uv sync && uv run loglens --help` exited 0 and listed all four subcommands. |
| A2 | PASS | `uv run loglens summary sealed.log` reported 40 requests, 7 IPs, `00:05:00`–`23:59:59`, and 50.00%, matching ground truth. |
| A3 | PASS | `uv run loglens --format json summary sealed.log` exited 0; the single JSON object contained `40`, `7`, the expected timestamps, and `50.0`. |
| A4 | PASS | `uv run loglens top sealed.log --by path -n 3` produced `/alpha 9`, `/beta 8`, `/charlie 7`; `/charlie` won the tie with `/delta`. |
| A5 | PASS | Windowed `errors` produced `(404,/alpha,3)`, `(404,/beta,2)`, `(500,/charlie,2)`, `(500,/delta,1)`, `(503,/epsilon,1)`. |
| A6 | PASS | JSON `hourly` returned exactly 24 keys with nonzero counts `00:3, 01:2, 06:4, 09:3, 10:9, 11:4, 12:5, 15:5, 23:5`. |
| A7 | PASS | JSON summary stdout parsed cleanly while stderr alone reported `skipped 3 malformed lines`. |
| A8 | PASS | Missing file exited 2, malformed-only file exited 1, and sealed fixture summary exited 0. |
| A9 | PASS | `uv run pytest -q` exited 0: `23 passed`. |
| A10 | PASS | `uv run ruff check .` exited 0: `All checks passed!`. |
| A11 | PASS | `tests/fixtures/sample.log` has 34 lines: 32 valid, 2 malformed, and 2xx/3xx/4xx/5xx statuses. |
| A12 | PASS | `just check` exited 0 and ran both Ruff and all 23 tests. |

**Acceptance passes:** 12/12  
**Correctness score:** 5/5
