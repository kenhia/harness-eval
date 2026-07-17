# 07-baseline-claude — fix delta — grader: fable1

Agent-authored fix = `pre-fix..aa72f53` (one commit, 4 files, +58/−1). Clean tree at done. Claude runner; profile MCP unaffected by the copilot-cell E1 covariate.

| dim | score /5 | note |
|---|---|---|
| fix correctness | 5 | 26/26 + 3/3 (supplied acceptance). The EOF arm checks the parser's existing `path` stack (parse.rs:204-215): truncation of any shape errors with the unclosed element named (F1), a fully-closed empty document passes (F2), and the untouched error → last_error plumbing handles recovery (F3). No heuristics, no downstream special-casing |
| fix quality | 5 | Root cause at the XML layer in ~13 production lines, consuming state the parser already kept, with a comment that states the well-formedness invariant ("a well-formed document has closed every element it opened") rather than just the symptom. Minimal blast radius; the error message is faithful and specific |
| tests | 4 | Unprompted and multi-layer: a unit test against the real `parse_feed` (tests/parse.rs:205-220), the `truncated.xml` fixture added to the corpus with a description distinguishing it from the mismatched-tag `malformed.xml`, and a dedicated e2e test (e2e.rs:548-565) asserting `status: error`, `new_entries` 0, `last_error` recorded, and `entry_count` 0. Docked one: both the unit and e2e layers exercise only the bug report's exact repro body — no second truncation shape anywhere in the suite, so the armor is sample-shaped even though the fix itself is generic |
| scope & process | 5 | Four files, all strictly fix-or-verification; one coherent commit (`feedhub-core: treat truncated feed XML as a parse error`) matching the diff; clean tree at done. Nothing extraneous |
| efficiency | 5 | 2m57s wall (below the field midpoint; range 1m29s–6m42s), 8.5k output tokens — the lean end of the Claude runner (same-runner 06: 3m44s, 12.8k) while still delivering unit + fixture + e2e coverage. 26 assistant turns, no wasted cycles visible in the runlog |

**Weighted total:** 96/100
(5/5×35 + 5/5×30 + 4/5×20 + 5/5×10 + 5/5×5)

**Verdict:** A near-flawless economy fix — the stack-check core plus full-pipeline regression coverage at the field's second-best cost — kept from the top only by regression tests that never stray from the bug report's own sample.
