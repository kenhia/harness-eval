# 02-atv-phoenix — fix delta — grader: fable1

Agent-authored fix = `pre-fix..a36d0b2` (one commit, 6 files, +66/−4) — the graded rerun with the phoenix MCP spine verified present. Per protocol the voided E1 attempt (`void/fix-e1`, `.scratch/`) was not read and plays no part in this sheet.

| dim | score /5 | note |
|---|---|---|
| fix correctness | 5 | 26/26 + 3/3 (supplied acceptance). The EOF guard consults the parser's existing open-element stack (parse.rs:181-189), so detection is generic over truncation shape (F1) and a fully-closed empty document still parses (F2) — no zero-items heuristic anywhere. Error flows through the existing FeedError::Parse → last_error path; the fix even anticipated F3's recovery semantics implicitly by changing nothing in the fetch loop |
| fix quality | 5 | Root cause at the XML layer, using state the parser already maintained, with an accurate comment naming quick-xml's clean-EOF behavior. The production change is ~9 lines; everything else in the commit is test/fixture/doc support. Uniquely in the field, it also authored a `well_formed_empty_feed_is_ok` unit test (parse.rs:377-383) — the agent independently identified and fenced off the cheap "0 items = error" wrong-fix that F2 exists to kill |
| tests | 5 | Deepest regression suite of the field, unprompted: three unit tests (repro body, a different truncation, and the well-formed-empty guard), plus a new e2e stage (e2e.rs:293-308) driving a real `truncated.xml` fixture through feedgen and asserting `status: error`, `last_error` set, and neighbor-feed isolation, plus the fixture wired into `feedgen make-fixtures` with corpus README rows distinguishing mismatched-tag vs truncation flavors |
| scope & process | 4 | Everything the fix needed is present and coherent in one commit with an accurate message; README/corpus docs updated in the same breath. Docked one for the harness-state edits bundled in: `.phoenix-ralph/done-check.json` was retargeted from `just check` to the single e2e test (with timeout halved) — narrowing the done gate below even the new unit tests it had just written (the committed `.phoenix/trace.jsonl` shows only the e2e target sensed during the fix run; fmt/clippy/units were left to luck, though acceptance C12a-c confirm they held) |
| efficiency | 3 | Slowest of the field: 6m42s wall (field range 1m29s–6m42s), 261.6 credits (2.2× the copilot-cell median of 119.0, 3.9× the cheapest cell), 20.9k output tokens, 15 premium requests. The extra spend bought the field's best test suite, but on this dimension's own terms it is the clear outlier |

**Weighted total:** 96/100
(5/5×35 + 5/5×30 + 5/5×20 + 4/5×10 + 3/5×5)

**Verdict:** The most complete fix of the field — the only cell to independently fence off the F2 wrong-fix — paid for with the field's longest and costliest run and a quietly weakened phoenix done gate.
