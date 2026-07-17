# 05-baseline — grader: fable1

| dim | score /5 | note |
|---|---|---|
| correctness | 5 | 12/12 acceptance (see 05-acceptance-fable1.md) |
| code quality | 4 | Appropriately minimal — smallest diff of the field (908 insertions) with the same clean parser/analyze/cli layering; flat package layout is a legitimate simplicity choice. Docked: `parse_lines(lines: object)` with a `# type: ignore[assignment]` where `Iterable[str]` was the obvious annotation; offset-less `--since` hits the naive-vs-aware `TypeError` traceback |
| tests | 4 | 23 tests covering the important edges: tie-break, exit codes 1/2, JSON single-document purity, malformed variants (bad timestamp, bad request line), blank-line handling. Thinnest suite of the field; no test around `--since` parsing or `--format` positioning |
| docs | 5 | README alone suffices: install, log-format reference, global options, every subcommand with example output, exit-code table, development section; only working (offset-aware) examples shown |
| process | 4 | Three commits with accurate messages in a sensible scaffold → implement → test sequence; docked because the third bundles fixture + tests + justfile into one commit, and there's no trace of self-review anywhere (nothing prescribed it, and nothing happened) |
| efficiency | 5 | Cheapest and fastest of the field by a wide margin: 4m32s wall, 143 credits, 1.3M tokens up — roughly a third of run 01's spend — for a 12/12 result |
| autonomy | 5 | Zero interventions; declared done; committed |

**Weighted total:** 91/100
(5/5×30 + 4/5×20 + 4/5×15 + 5/5×10 + 4/5×10 + 5/5×10 + 5/5×5)

**Best thing:** Value for money: a raw prompt with no machinery produced a 12/12, lint-clean, well-documented tool in 4½ minutes and 143 credits — the reference point every harness in this eval has to justify itself against.

**Worst thing:** The corners nobody forced it to check: offset-less ISO 8601 bounds crash with a raw traceback, the test suite is the thinnest of the five, and a stray `lines: object` annotation needed a `# type: ignore` to paper over itself.

**Narrative (≤150 words):** The control run makes the eval's central question sharp. With no harness at all, the agent produced a complete, spec-correct loglens — every acceptance check passes, including the designed tie-break and window traps — with clean layering, a genuinely sufficient README, and the lowest cost of the field by ~2–3×. What's missing is depth at the margins: the test suite covers the spec's stated edges but stops there, an un-forced typing wart survived, and the same naive-datetime crash that slipped past runs 02 and 03 lives here too. The three-commit history is serviceable but coarse. In short: the baseline matched the harnesses on everything the spec demanded, and fell behind only on the robustness and process depth that the better harness runs added — at a third of the price.
