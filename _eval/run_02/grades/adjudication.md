# Consensus adjudications — run_02

Factual checks by the consensus session (fable2). Interpretation
precedents, if any, go to `precedents.md`. No acceptance-level disputes
arose this run: correctness was supplied by the executable suite
(post-S1) and both graders' correctness rows match on all seven repos —
the dispute class that dominated run 1 is structurally gone.

## A1 — 06-gstack test count (fable1: 165, sol: 149)

Not score-driving (tests scored 5/4, a gap of 1 → mean), but `final.md`
cites the number. Verified in a fresh clone at HEAD `2abafec`:
`cargo test --workspace` runs **165 tests, 165 passed** (sum of every
runner's "running N tests" line, including doc-tests and integration
binaries). A raw `#[test]`/`#[tokio::test]` attribute grep gives 152 —
neither grader's figure — so sol's 149 was likely a partial static
count. **Ruling: 165 stands**; the executable count is authoritative.
