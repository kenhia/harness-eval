# 01-atv-starterkit — grader: fable1

Agent-authored work judged at pre-run..e3ce602; the final commit c950337
("eval: post-run snapshot") is eval infrastructure capturing `.atv`
runtime state the agent left uncommitted, and is excluded from code
judgment (it is charged under Autonomy per the run-02 protocol).

| dim | score /5 | note |
|---|---|---|
| correctness | 3 | Mapping applied: core 13/14 → core-failure cap ≤3 → 3 (hard 11/12 noted; cannot lift past the cap). Failures C9+H12, one root cause: `parse_feed` treats `Event::Eof` as normal termination (parse.rs:96), so the suite's truncated malformed.xml parses as an empty feed and refresh reports `status: ok, new_entries: 0` instead of recording `last_error` — despite fetch.rs:20-21 documenting that malformed XML returns Err, and despite the parser already maintaining the open-element stack (parse.rs:84,116,134) that would detect it |
| code quality | 4 | Clean sync stack (tiny_http/ureq/quick-xml/rusqlite), layered error flow, no panics on IO in request paths, clippy -D warnings clean. Elegant: fixed-width `fmt_utc` storage plus `normalize_instant` on query bounds, so lexicographic SQL comparison is exact instant comparison for any offset. Docked: the doc-comment-vs-behavior contradiction at fetch.rs:20-21, and minor duplication (feed_json/all_feeds_json, opt_str/field_str) |
| tests | 2 | Thinnest suite of the field: 12 tests (6 parse, 5 date, 1 e2e). The e2e is genuinely real — spawns the feedd binary via CARGO_BIN_EXE against feedgen's server over local HTTP, asserting 201/409/422, failure isolation, window, search, cascade delete. But feedctl's 303 lines have zero tests (exit codes 0/1/2 untested), and dedupe update-in-place, pagination/offset, limit clamp, and nulls-last/id-tiebreak ordering are untested anywhere; the dates.xml fixture exists but is never registered in the e2e |
| docs | 5 | README (309 lines) alone suffices: build, `just check` + non-just fallback, quick start for all three binaries, per-binary sections, behavior notes (dates, dedupe, conditional GET, failure isolation), all 8 endpoints individually documented with object shapes, feedctl exit codes |
| process | 5 | Five conventional commits in a clean bottom-up sequence (workspace/lib → feedd → feedgen → feedctl → tests/docs), each body substantive and accurate (e.g. 0c9d953 explains feedgen is exposed as a library so the e2e can drive it in-process); zero ceremony committed — no plan/todo artifacts at all |
| efficiency | 5 | Below the same-runner control on every unit: 596.7 credits vs 05's 703.0 (0.85×), 16m22s vs 19m35s (0.84×), identical 15 premium requests, 71.1k vs 85.5k output tokens. Control-comparable band → 5 |
| autonomy | 4 | Zero interventions, declared done — but the run ended with a dirty working tree: `.atv/observations.jsonl` (harness runtime state) neither committed nor gitignored, requiring the eval's post-run snapshot commit c950337. Every other cell finished clean |

**Weighted total:** 74/100
(3/5×30 + 4/5×20 + 2/5×15 + 5/5×10 + 5/5×10 + 5/5×10 + 4/5×5)

**Best thing:** The timestamp-normalization design: `fmt_utc` stores fixed-width RFC 3339-Z so lexicographic order equals chronological order, and `normalize_instant` (server.rs:159-165) converts query bounds — including non-UTC offsets — to the same form, making the pinned window/ordering semantics fall out of plain SQL string comparison.

**Worst thing:** The verification gap: 12 tests for a 2,051-line workspace, with feedctl completely untested and the truncated-XML hole sitting one `if !stack.is_empty()` away from the element stack the parser already maintains — the data structure needed to pass C9 was built and never consulted at Eof.

**Narrative (≤150 words):** The smallest artifact of the field (2,051 Rust lines) and in places the most elegant — the UTC string-normalization scheme is the cleanest solution to the pinned window semantics any repo produced, and the commit history is textbook. But the discipline that produced run 1's edge-case champion didn't show up here: the test suite is the thinnest of the seven, feedctl ships 303 lines with zero coverage of its spec'd exit-code contract, and the one behavior the field stumbled on — truncated XML must surface as a fetch failure — was documented in a comment, half-implemented in the parser's own state, and never checked or tested. The run was fast and cheap (0.85× the control's credits) but also left harness runtime state dirtying the tree at completion. A lean, stylish build whose verification didn't earn its confidence.
