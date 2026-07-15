# Summary — grader: fable1

Objective + rubric pass, all five repos, graded in order 01→05 against the
sealed fixture in `sealed-fixture-fable1/`. Per-repo details in
`NN-acceptance-fable1.md` and `NN-fable1.md`. No ranking commentary here per
the grading protocol; synthesis belongs to the consensus session.

| repo | correctness | code quality | tests | docs | process | efficiency | autonomy | acceptance | weighted total |
|---|---|---|---|---|---|---|---|---|---|
| 01-atv-starterkit | 5 | 5 | 5 | 5 | 4 | 2 | 5 | 12/12 | 92/100 |
| 02-atv-phoenix | 5 | 4 | 4 | 4 | 5 | 3 | 5 | 12/12 | 87/100 |
| 03-working-skill-repo | 5 | 4 | 4 | 5 | 5 | 4 | 5 | 12/12 | 91/100 |
| 04-kprojects | 5 | 5 | 5 | 5 | 5 | 3 | 5 | 12/12 | 96/100 |
| 05-baseline | 5 | 4 | 4 | 5 | 4 | 5 | 5 | 12/12 | 91/100 |

Weights: correctness 30, code quality 20, tests 15, docs 10, process 10,
efficiency 10, autonomy 5. Weighted total = Σ(score/5 × weight).

Cross-cutting observation recorded for the consensus session (not a ranking):
every repo passed all 12 acceptance checks, so dimension 1 does not
discriminate in this run; the discriminating signal came from robustness
edges outside the checklist (offset-less ISO 8601 window bounds crash runs
02, 03, and 05 with an unhandled naive-vs-aware `TypeError`; runs 01 and 04
normalize them, and only 04 tests that edge) and from cost (143–440 credits
for materially similar outcomes).
