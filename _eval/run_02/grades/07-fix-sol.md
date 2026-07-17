# 07-baseline-claude fix delta — grader: sol

| dimension | score /5 | note |
|---|---:|---|
| Fix correctness | 5 | The supplied `runs/07-fix-acceptance.txt` is core 14/14, hard 12/12, and fix 3/3. Commit `aa72f53` rejects EOF whenever `crates/feedhub-core/src/parse.rs` still has an open path element, so correctness follows the XML nesting invariant rather than item count or the report's exact sample. |
| Fix quality | 5 | The fix lands at the parser root cause: `Event::Eof` now becomes `ParseError::Xml` for an unclosed document. Existing refresh error propagation remains untouched, preserving the established path that records `last_error` and reports `status: "error"`. |
| Tests | 5 | The delta adds a parser regression test, a `truncated.xml` feedgen fixture, and `a_truncated_feed_is_recorded_as_an_error` in `crates/feedd/tests/e2e.rs`. The end-to-end assertion covers status, zero inserts, persisted `last_error`, and entry count, making this genuine unprompted regression coverage across both parser and service behavior. |
| Scope & process | 5 | Commit `aa72f53` changes four directly relevant files with 58 insertions and one deletion: parser, parser tests, fixture corpus, and e2e coverage. The single commit is coherent and introduces no unrelated production or process changes. |
| Efficiency | 5 | `runs/07-fix-runlog.md` records 2m57s, 8.5k output tokens, and 26 assistant turns. It is faster and lighter than the other Claude cell and below the six-run field median of roughly 3m20s. |

**Weighted total:** 100/100
