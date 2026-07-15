# 07-baseline-claude — acceptance: sol

| id | result | evidence |
|---|---|---|
| A1 | PASS | Following the README flow, `uv sync` then `uv run loglens --help` exited 0 and listed all four required subcommands. |
| A2 | PASS | Text summary reported 40 requests, 7 unique IPs, the exact first/last timestamps, and a 50.00% error rate. |
| A3 | PASS | JSON summary stdout parsed as one document containing the A2 values, including `errors: 20` and `error_rate: 50.0`. |
| A4 | PASS | `top --by path -n 3` returned `/alpha 9`, `/beta 8`, `/charlie 7`, correctly resolving the tie with `/delta`. |
| A5 | PASS | The inclusive error window returned exactly `(404,/alpha,3)`, `(404,/beta,2)`, `(500,/charlie,2)`, `(500,/delta,1)`, `(503,/epsilon,1)`. |
| A6 | PASS | JSON hourly output contained all 24 buckets and every count matched the sealed ground truth. |
| A7 | PASS | Stderr alone reported `skipped 3 malformed lines`; JSON stdout remained independently parseable. |
| A8 | PASS | Missing file exited 2, malformed-only input exited 1, and valid input exited 0. |
| A9 | PASS | `uv run pytest -q` exited 0: `77 passed`. |
| A10 | PASS | `uv run ruff check .` exited 0 with `All checks passed!`. |
| A11 | PASS | The sample fixture has 34 lines: 32 valid, 2 malformed, with 2xx, 3xx, 4xx, and 5xx coverage. |
| A12 | PASS | `just check` exited 0 and ran Ruff plus all 77 tests. |

**Acceptance passes:** 12/12  
**Correctness score:** 5/5
