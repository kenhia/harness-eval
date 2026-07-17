# 01-atv-starterkit — fix delta — grader: fable1

Agent-authored fix = `pre-fix..ba088ea` (one commit, `crates/feedcore/src/parse.rs` only, +24). The final commit 0df6ddb (`.atv` hook-telemetry snapshot) is eval-authored and excluded from all judgments per protocol.

| dim | score /5 | note |
|---|---|---|
| fix correctness | 5 | 26/26 + 3/3 (supplied acceptance @ ba088ea). The pass is achieved robustly: EOF-with-open-elements is detected generically via the parser's existing open-element stack (parse.rs:157-161), so any truncation shape errors — F1's alternate truncation falls out for free, and F2 passes because a well-formed empty feed leaves the stack empty. The error propagates through the existing ParseError → last_error path; no fetch-loop or handler special-casing |
| fix quality | 5 | Textbook root-cause fix at the XML layer: exactly the `if !stack.is_empty()` check my run_02 sheet noted the parser was one line away from — it finally consults the open-element stack it had maintained all along, with a comment accurately naming quick-xml's EOF leniency as the cause. Smallest possible blast radius (8 production lines, one file); the error message faithfully names the unclosed element. This also retires the fetch.rs:20-21 doc-vs-behavior contradiction docked in run_02 |
| tests | 4 | Two genuine, unprompted unit tests (parse.rs:308-324): the bug report's repro body plus a *different* truncation (`<rss><channel>` cut at a tag boundary), so the suite is not sample-shaped. Both drive the real `parse_feed` and necessarily fail pre-fix. Docked one: coverage stops at the parse layer — no e2e/fixture asserting the pipeline behavior the bug is actually about (refresh returns `status: error`, `last_error` recorded), in the repo whose run_02 weakness was exactly end-to-end verification thinness |
| scope & process | 4 | The fix commit itself is exemplary — one file, one coherent conventional-commit message, nothing extraneous. Docked one: the run again ended with a dirty tree (`.atv/observations.jsonl` untracked, per the runlog), the same blemish charged in run_02; the fix session had the run_02 status visible and a one-line `.gitignore` available, and was the only fix cell to finish dirty |
| efficiency | 5 | 1m53s wall, 86.2 credits, 6.3k output tokens, 15 premium requests — second-fastest of the field (range 1m29s–6m42s) and 0.72× the copilot-cell median credits (119.0). Well under median on every unit |

**Weighted total:** 94/100
(5/5×35 + 5/5×30 + 4/5×20 + 4/5×10 + 5/5×5)

**Verdict:** The minimal correct fix, landed in under two minutes at exactly the spot the run_02 grading identified — but verified only at the unit layer, and the `.atv` dirty-tree habit persisted.
