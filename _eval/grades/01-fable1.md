# 01-atv-starterkit — grader: fable1

| dim | score /5 | note |
|---|---|---|
| correctness | 5 | 12/12 acceptance (see 01-acceptance-fable1.md); only repo where every edge behavior (tie-break, window, streams, exit codes) was exact |
| code quality | 5 | Clean parser/analyze/cli split; tight CLF regex; documented since-inclusive/until-exclusive semantics in `analyze.errors`; deterministic secondary sort; no dead code. Nit: LogRecord carries referer/protocol/user_agent/size no subcommand uses |
| tests | 5 | 25 tests across parser/analyze/cli; explicitly cover tie-break (`test_top_by_ip_desc_with_tiebreak`), exit codes 1/2, stderr-vs-stdout separation, JSON single-document validity, time window, invalid `--since` — would catch real regressions |
| docs | 5 | README alone suffices: uv install (two paths), every subcommand with example output, JSON mode, error handling, exit codes, `just check` + non-just equivalent |
| process | 4 | 4 meaningful commits + feature-branch merge (`ff73575`→`bc5b3bf`); plan doc has a genuine requirements trace and was marked completed; docked one point because 220 plan lines for a ~370-line CLI is more ceremony than the outcome needed |
| efficiency | 2 | Most expensive run of the field by a wide margin: 10m54s wall, 440 AI credits, 4.7M tokens up (vs 143 credits/4m32s for run 05). Result is the best, but the rubric asks for low burn *for the result delivered*; ~3× baseline cost for a margin that is real but not 3× |
| autonomy | 5 | Zero interventions; declared done; committed and merged its own feature branch |

**Weighted total:** 92/100
(5/5×30 + 5/5×20 + 5/5×15 + 5/5×10 + 4/5×10 + 2/5×10 + 5/5×5)

**Best thing:** Edge-case discipline — tie-break ordering, since/until boundary semantics, and stderr/stdout separation are each implemented, documented, *and* tested (e.g. `test_no_valid_lines_exit_1` asserts stdout is empty AND both messages hit stderr).

**Worst thing:** Cost. 440 credits and 4.7M uploaded tokens — 3× the baseline run — plus a fix commit (`a99b165`) for invalid-date handling that ideally lands in the first pass. The parser also stores four record fields nothing consumes.

**Narrative (≤150 words):** The strongest artifact of the five. The agent planned, implemented on a feature branch, self-reviewed (the invalid `--since`/`--until` fix commit shows the review loop caught something real), and merged — and the result is the only 12/12 in the field. The code reads like a maintainer wrote it: parsing, analysis, and presentation are separated; ambiguous spec points (window boundary semantics, tie ordering within equal error counts) were resolved explicitly and documented rather than left to chance. Tests are behavior-focused and target exactly the places a regression would hide. The trade-off is spend: roughly 2.4× the wall clock and 3× the credits of the no-harness baseline, and a long plan document whose value over a shorter one is unclear. If cost matters less than correctness, this is the winner.
