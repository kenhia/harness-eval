# 01-atv-starterkit — grader: sol

| dim | score /5 | note |
|---|---:|---|
| correctness | 3 | The supplied acceptance result is **(core 11/12, hard 4/9)**, which maps mechanically to 3 with no hard-tier adjustment. The only core miss and five hard misses share one cause: `parser.py` strips CLF offsets, so aware `errors` windows crash in `handlers.py`. |
| code quality | 3.5 | The parser, pure handlers, and argparse layer are cleanly separated, typed, and easy to navigate, with sensible text/JSON formatting. The timestamp parser explicitly assumes UTC while returning naive datetimes, status checks accept values above 599, error-group ties are not deterministic, and some formatting expressions are needlessly awkward. |
| tests | 4 | The 48 tests are the broadest suite in the field, covering parser failures, dash bytes, summaries, exact tie-breaking, time filters, grouping, all hours, JSON, exits, help, and subprocess integration. They still use only naive filters, and the malformed-stderr integration assertion is conditional enough to pass when nothing is reported, so the acceptance crash was not caught. |
| docs | 3 | `README.md` covers uv installation, all four commands, output formats, exits, malformed reporting, and check commands. However, all `--format` examples put the global option after the subcommand although argparse only accepts it before, the nested project location is not explained from repo root, and `ruff check src/` references a nonexistent layout. |
| process | 1 | A detailed 21KB active plan traces requirements, but its checkboxes remain open, it prescribes a `src/` layout and `uv.lock` that were not delivered, and all implementation arrived in one large commit. Combined with the nested project and dirty agent-done tree, this is substantial ceremony without corresponding reviewability or completion hygiene. |
| efficiency | 2 | Relative to Copilot control 05, this run used 153.8 versus 57.1 AI credits, the same 0.33 premium requests, and 9m29s versus 4m39s. Credits were 2.69× control and wall time just over 2×, placing it in the rubric’s ≤3× band without a sufficient quality delta. |
| autonomy | 4 | The run was headless, required zero interventions, and exited successfully after producing a nearly complete artifact. It loses one point because the agent-done tree remained dirty rather than fully finished and committed. |

**Weighted total:** 60/100

**Best thing:** It produced the tier’s broadest regression suite and a clean separation between parsing, analysis, and CLI concerns.

**Worst thing:** A very large planning artifact failed to prevent the central timezone bug, incorrect documentation examples, and an unfinished dirty delivery.

**Narrative (≤150 words):** StarterKit yielded a strong basic implementation and by far the largest test suite, but the extra process cost did not translate into hard-case robustness. The parser deliberately removes timezone offsets, so otherwise good error-window logic crashes on standard aware ISO8601 values. Tests cover most required behavior but repeat the same naive-datetime assumption, while a conditional stderr assertion masks malformed-reporting risk. Documentation is extensive yet contains invalid global-option examples and stale layout commands. The nested layout is not a correctness defect under S1, but together with one giant commit, an unchecked active plan, and a dirty final tree it makes process quality poor. At 2.69× control credits, this was also the least efficient Copilot run.
