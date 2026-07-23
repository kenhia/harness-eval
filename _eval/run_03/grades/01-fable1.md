# 01-atv-starterkit — grader: fable1 (run_03, Haiku 4.5 tier)

| dim | score /5 | note |
|---|---|---|
| correctness | 3 | Mechanical mapping: core 11/12, hard 4/9 → base 3, no adjustment (hard < 6/9). Sole core miss (A5) and four hard misses share one root cause: the parser discards the CLF timezone offset, so any aware `--since/--until` crashes with `TypeError: can't compare offset-naive and offset-aware datetimes` (`loglens/loglens/handlers.py:70`) |
| code quality | 3 | Clean parser/handlers/cli split, tight CLF regex, correct tie-break (`key=(-count, value)`), sensible exit-code paths. Docked for: the timezone shortcut is *knowing* — `parser.py:50` says "For now, we'll assume UTC… we might parse the timezone offset"; the identical 12-line load/warn/empty-check block is pasted into all four `cmd_*` handlers; `LogEntry` carries `http_version`/`referer`/`user_agent` no subcommand uses; mixed `Optional`/`Tuple` vs `list[...]` typing |
| tests | 3 | 48 tests across parser/handlers/integration — the broadest suite of the field: tie-break (`test_tie_breaking_by_value_ascending`), since/until filters, exit code 2, JSON validity, help. But no exit-code-1 (no-valid-lines) test, `test_stderr_reporting_of_malformed` is guarded by an `if` that makes it pass vacuously, and zero timezone coverage — the exact area the repo failed on |
| docs | 4 | `loglens/README.md` (171 lines) alone suffices: uv and pip install, every subcommand with example output, JSON mode, `just check`. Sits one level down due to the nested layout (weighed under process, not here) |
| process | 2 | One giant commit (`db8c6b2`, 16 files/1927 lines) despite the StarterKit plan-then-implement workflow. The 548-line plan doc has a genuine requirements trace, but its own risk register names "Timestamp parsing bugs (timezone handling)" with mitigation "thorough unit tests for timestamp parsing" — and the implementation shipped a commented-out-loud tz punt with no tz tests. Plan ceremony that did not change the outcome scores low. Nested `loglens/` layout (defect S1: spec-permissible, convention/discoverability observation). Tree dirty at agent-done |
| efficiency | 2 | 153.8 AI credits vs control 05's 57.1 = 2.7×; wall 9m29s vs 4m39s = 2.0×. Both in the ≤3× band. Quality delta vs control (+2 core) is real but bought at the field's highest burn |
| autonomy | 4 | Headless, zero interventions, runner exit 0; docked one for finishing with an uncommitted-changes tree at agent-done (runlog) |

**Weighted total:** 59/100
(3/5×30 + 3/5×20 + 3/5×15 + 4/5×10 + 2/5×10 + 2/5×10 + 4/5×5)

**Best thing:** Test breadth — 48 behavior-focused tests in three layers, including an explicit tie-break ordering test and JSON-document validity checks; nothing else in the field tests this many distinct behaviors.

**Worst thing:** The timezone punt was premeditated and un-mitigated: the plan flagged tz as the top parsing risk, the code comments admit the offset is being dropped "for now", and no test covers it — that one decision accounts for all six acceptance failures.

**Narrative (≤150 words):** A competent, well-organized artifact undermined by one consciously deferred decision. The module split is right, the CLI is conventional argparse, the README is genuinely usable, and the test suite is the field's largest. But the agent wrote down the timezone risk in its own 548-line plan, then dropped the offset in the parser with a "for now" comment, tested nothing in that area, and shipped — turning one shortcut into its only core failure plus four hard-tier failures. Process didn't help: all 1927 lines landed in a single commit, the tree was dirty at done, and the project was nested a level down (S1). At 2.7× the control's credits and 2× its wall clock, the harness's planning machinery produced the field's most expensive run and a plan the implementation didn't obey.
