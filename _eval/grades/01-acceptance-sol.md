# 01-atv-starterkit — acceptance: sol

| id | result | evidence |
|---|---|---|
| A1 | PASS | `uv sync && uv run loglens --help` exited 0 and listed all required subcommands. |
| A2 | PASS | Text summary matched 40 requests, 7 IPs, exact first/last timestamps, and 50.00% errors. |
| A3 | PASS | JSON summary stdout parsed as one document and contained all expected summary values. |
| A4 | PASS | Top paths were `9 /alpha`, `8 /beta`, `7 /charlie`, correctly resolving the tie with `/delta`. |
| A5 | **FAIL** | Windowed errors reported `(404,/alpha,2)` instead of expected count 3 because the `11:00:00` record was excluded by `timestamp >= until`. |
| A6 | PASS | Nested JSON `hourly` object contained 24 buckets with all expected counts. |
| A7 | PASS | Stderr alone reported `skipped 3 malformed lines`; summary JSON stdout remained parseable. |
| A8 | PASS | Missing file exited 2, malformed-only exited 1, and valid input exited 0. |
| A9 | PASS | `uv run pytest -q` exited 0: `25 passed`. |
| A10 | PASS | `uv run ruff check .` exited 0: `All checks passed!`. |
| A11 | PASS | Sample fixture has 32 lines: 30 valid, 2 malformed, with 2xx/3xx/4xx/5xx coverage. |
| A12 | PASS | `just check` exited 0 and ran Ruff plus all 25 tests. |

**Acceptance passes:** 11/12  
**Correctness score:** 4/5
