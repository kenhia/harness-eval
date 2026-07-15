# 06-gstack — acceptance: sol

| id | result | evidence |
|---|---|---|
| A1 | PASS | Following the README development flow, `uv sync` then `uv run loglens --help` exited 0 and listed all four required subcommands. |
| A2 | PASS | Text summary matched 40 requests, 7 unique IPs, the exact first/last timestamps, and a 50.00% error rate. |
| A3 | PASS | JSON summary stdout parsed as one document containing every required A2 value. |
| A4 | PASS | `top --by path -n 3` returned `/alpha 9`, `/beta 8`, `/charlie 7`, correctly resolving the tie with `/delta`. |
| A5 | **FAIL** | The window returned `(404,/alpha,2)` and 8 total errors because `filter_entries` applies an exclusive upper bound and drops the record exactly at `11:00:00`. |
| A6 | PASS | JSON hourly output contained all 24 buckets and every count matched the sealed ground truth. |
| A7 | PASS | Stderr alone reported 3 malformed lines with first-error detail; JSON stdout remained independently parseable. |
| A8 | PASS | Missing file exited 2, malformed-only input exited 1, and valid input exited 0. |
| A9 | PASS | `uv run pytest -q` exited 0: `137 passed, 1 skipped`. |
| A10 | PASS | `uv run ruff check .` exited 0 with `All checks passed!`. |
| A11 | PASS | The sample fixture has 36 lines: 34 valid, 2 malformed, with 2xx, 3xx, 4xx, and 5xx coverage. |
| A12 | PASS | `just check` exited 0 and ran Ruff lint, format checking, all tests, and JSON smoke checks. |

**Acceptance passes:** 11/12  
**Correctness score:** 4/5
