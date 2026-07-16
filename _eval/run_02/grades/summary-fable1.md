# Summary — grader: fable1

Rubric pass, all seven repos, graded in order 01→07 per protocol.
Correctness mapped mechanically from the supplied acceptance results
(`runs/NN-acceptance.txt`, post-S1 suite): core 14/14 → 4 with the fifth
point from the hard tier (≥9/12); any core failure caps the dimension at
≤3. Efficiency anchored to the same-runner control (05 for Copilot
cells, 07 for Claude cells). Per-repo details in `NN-fable1.md`. No
ranking commentary here per the grading protocol; synthesis belongs to
the consensus session.

| repo | correctness | code quality | tests | docs | process | efficiency | autonomy | acceptance (core, hard) | weighted total |
|---|---|---|---|---|---|---|---|---|---|
| 01-atv-starterkit | 3 | 4 | 2 | 5 | 5 | 5 | 4 | 13/14, 11/12 | 74/100 |
| 02-atv-phoenix | 3 | 4 | 4 | 5 | 3 | 5 | 5 | 13/14, 11/12 | 77/100 |
| 03-working-skill-repo | 5 | 5 | 4 | 5 | 5 | 5 | 5 | 14/14, 12/12 | 97/100 |
| 04-kprojects | 3 | 4 | 3 | 5 | 5 | 5 | 5 | 13/14, 11/12 | 78/100 |
| 05-baseline | 3 | 4 | 4 | 5 | 5 | 5 | 5 | 13/14, 11/12 | 81/100 |
| 06-gstack | 3 | 5 | 5 | 5 | 5 | 4 | 5 | 13/14, 11/12 | 86/100 |
| 07-baseline-claude | 3 | 5 | 5 | 5 | 5 | 5 | 5 | 13/14, 11/12 | 88/100 |

Weights: correctness 30, code quality 20, tests 15, docs 10, process 10,
efficiency 10, autonomy 5. Weighted total = Σ(score/5 × weight).

Cross-cutting observations recorded for the consensus session (not a
ranking):

1. **The field's failures are one bug, six times.** Every C9/H12 failure
   has the identical root cause: a hand-driven event loop over quick-xml
   that treats `Event::Eof` as normal termination, so a document
   truncated *between* tags parses as a valid empty feed and refresh
   reports success instead of recording `last_error`. quick-xml reports
   mismatched end tags but not unclosed elements at EOF. All six
   affected repos also authored their own `malformed.xml` fixture in the
   mismatched-tag flavor — the one class their parser catches — so
   every repo's own malformed-XML test passes while the acceptance
   suite's truncation fixture fails. 03 passed because it chose
   roxmltree, a strict whole-document parser: a dependency choice, made
   before any test ran, was worth 23 weighted points.
2. **Copilot-cell efficiency did not discriminate.** All five Copilot
   cells landed 0.77–1.05× of the control on credits and wall clock
   with identical premium-request counts (15) — every cell scores 5.
   The only efficiency signal in the field is 06 vs 07 on the Claude
   runner (1.96× output tokens, 1.33× wall, inside a measured ~44%
   single-run variance band).
3. **Verification depth, not implementation quality, separated the
   field.** All seven repos implement the pinned semantics essentially
   correctly outside the shared parser hole; the spread in this
   scorecard comes from tests (2–5) and process granularity, with the
   two Claude-runner cells carrying the only exit-code test matrices in
   the field.
