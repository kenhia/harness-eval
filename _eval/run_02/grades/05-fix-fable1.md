# 05-baseline — fix delta — grader: fable1

Agent-authored fix = `pre-fix..cb7f388` (one commit, `crates/feedlib/src/parse.rs` only, +19/−1). Clean tree at done.

| dim | score /5 | note |
|---|---|---|
| fix correctness | 5 | 26/26 + 3/3 (supplied acceptance). The EOF arm now checks the parser's existing open-element stack (parse.rs:113-119) and errors with the unclosed element named, so any truncation shape fails (F1) and a fully-closed empty document still parses (F2) — generic detection, no sample- or zero-items-shaped heuristic. Propagation rides the existing ParseError::Xml → last_error path untouched (F3 for free) |
| fix quality | 5 | Root cause at the XML layer with the smallest diff of the field: the fix is entirely inside the `Event::Eof` arm, consuming state the parser already maintained, with a comment and error message that faithfully describe the failure. Zero collateral — no fetch-loop or handler changes, no robustness the bug didn't ask for |
| tests | 3 | One genuine, unprompted regression test (parse.rs:461-470) — it drives the real `parse_feed` and fails pre-fix, which the bug report did not ask for. But it is the thinnest verification of the field: the test body is the bug report's repro sample verbatim, with no second truncation shape to guard against sample-shaped fixes, no well-formed-empty guard, and no e2e/fixture asserting the actual reported behavior (`status: error`, `last_error` set) |
| scope & process | 5 | Perfectly minimal blast radius: one file, one coherent commit whose message says exactly what the diff does, working tree clean at done. Nothing extraneous touched |
| efficiency | 5 | Fastest and cheapest of the field: 1m29s wall (field range 1m29s–6m42s), 67.4 credits (0.57× the copilot-cell median of 119.0), 4.5k output tokens, 15 premium requests |

**Weighted total:** 92/100
(5/5×35 + 5/5×30 + 3/5×20 + 5/5×10 + 5/5×5)

**Verdict:** The purest minimal fix — right layer, existing state, nineteen lines, ninety seconds — whose only weakness is that its regression armor is a single copy of the bug report's own sample.
