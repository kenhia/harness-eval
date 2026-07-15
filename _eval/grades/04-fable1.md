# 04-kprojects — grader: fable1

| dim | score /5 | note |
|---|---|---|
| correctness | 5 | 12/12 acceptance (see 04-acceptance-fable1.md) |
| code quality | 5 | Clean parser/analyze/cli layering; `CLIError` carries its exit code so `main` has one error path; naive `--since`/`--until` values are explicitly normalized to UTC (documented in `_ensure_utc`) — the exact hole runs 02 and 03 crash on. Documented non-empty assumption in `summary()` instead of dead defensive code |
| tests | 5 | 28 tests: tie-break, exit codes, stderr-vs-stdout, `test_stdout_is_valid_json_only`, negative `-n` rejection, and `test_errors_since_naive` — the only suite in the field that tests the naive-timestamp edge it handles |
| docs | 5 | README (165 lines) alone suffices: install, all four subcommands with example output, JSON section, behavior & exit codes, development; nothing documented that doesn't work |
| process | 5 | Three conventional commits (feat → test → docs) in a coherent sequence; `sprints/001-loglens-cli.md` is the anti-ceremony counterexample — its Decisions section records real, code-verifiable choices (tz normalization, stderr policy, tie orders) and the roadmap got genuine follow-ups; korg WI #445 linkage per run log |
| efficiency | 3 | 6m52s wall, 252 credits, 2.8M tokens up — mid-field alongside run 02 (~1.75× baseline credits), no visible rework |
| autonomy | 5 | Zero interventions; declared done; committed |

**Weighted total:** 96/100
(5/5×30 + 5/5×20 + 5/5×15 + 5/5×10 + 5/5×10 + 3/5×10 + 5/5×5)

**Best thing:** Timezone discipline end-to-end: naive ISO 8601 bounds are normalized to UTC in one documented helper, exercised by a dedicated test (`test_errors_since_naive`), and the decision is written down in the sprint record — implementation, test, and rationale all agree.

**Worst thing:** Nothing is broken, so the dings are small: 3xx coverage in `sample.log` is a single 302 line (letter of the spec, not its spirit), and three coarse commits give less archaeological value than the five-step histories elsewhere in the field.

**Narrative (≤150 words):** The most robust artifact of the five. Everything the acceptance list measures passes, and the places it doesn't measure — naive timestamp bounds, stdout purity under JSON, negative `-n` — were found, handled, and tested anyway. The sprint record is what process artifacts should be: a decisions log a future maintainer would actually consult, not a restatement of the diff. Code reads cleanly, with the field's nicest error-handling shape (`CLIError` carrying its exit code). Its weaknesses are marginal: mid-field token spend, a fixture that satisfies the 3xx requirement with exactly one line, and a slightly coarse three-commit history. On artifact quality alone this edges out run 01 by spending its robustness budget on the tool rather than on planning prose — and it cost 43% fewer credits getting there.
