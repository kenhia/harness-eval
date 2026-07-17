# 06-gstack — fix delta — grader: fable1

Agent-authored fix = `pre-fix..1b7256a` (one commit, 3 files, +80/−3). Clean tree at done. Claude runner; profile MCP unaffected by the copilot-cell E1 covariate.

| dim | score /5 | note |
|---|---|---|
| fix correctness | 5 | 26/26 + 3/3 (supplied acceptance). The EOF arm checks the parser's existing open-element stack (parser.rs:393-407): any truncation shape errors (F1), a well-formed empty document has popped its root and passes (F2) — the comment even states that invariant explicitly — and the untouched fetch path clears last_error on the next success (F3). Generic detection, faithful "unclosed <elem>" error recording |
| fix quality | 5 | Root-cause fix at the XML layer using existing parser state, ~13 production lines, and the best-explained fix of the field: the comment precisely documents why `check_end_names` misses this class (it rejects *mismatched* end tags, not a document that simply stops) and what the salvage failure mode looked like. The fixture doc (fixtures.rs:159-168) carries the same mismatched-vs-truncated distinction into the corpus, closing the exact blind spot that let every repo's own malformed.xml test pass in run_02 |
| tests | 5 | Unprompted, multi-layer, and not sample-shaped: two unit tests (the repro body plus an unclosed-root variant), a dedicated e2e test (e2e.rs:339-363) with the field's richest assertions (`status: error`, error message present, `last_error` a string, `entry_count` 0), the `truncated.xml` fixture added to the corpus, and the existing corpus-sweep test generalized to assert both broken fixtures fail to parse — regression coverage wired into three layers of the repo's own verification |
| scope & process | 5 | Three files, each strictly in service of the fix or its regression coverage; one coherent commit (`fix(parser): reject truncated documents instead of a silent empty feed`) whose message matches the diff exactly; clean tree at done. No ceremony, no drive-by changes |
| efficiency | 4 | 3m44s wall (mid-field; range 1m29s–6m42s) and 12.8k output tokens vs the same-runner control 07's 2m57s and 8.5k — 1.27× wall, 1.51× output for a comparable-shape (if somewhat deeper) delta. That sits at the edge of the ~44% single-run variance band measured in run_02, so one point off rather than noise-forgiven |

**Weighted total:** 99/100
(5/5×35 + 5/5×30 + 5/5×20 + 5/5×10 + 4/5×5)

**Verdict:** The best all-around fix of the field: same minimal stack-check core as the leaders, but with the clearest written understanding of *why* the bug existed and regression coverage integrated at every layer the repo owns — at a mildly above-control token cost.
