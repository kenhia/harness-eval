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

## Delta pass (run 1.5) — 06-gstack, 07-baseline-claude

Graded 2026-07-15, order 06 → 07, same sealed fixture
(`sealed-fixture-fable1/`) and same scale as rows 01–05 (which are frozen).
Runner covariate: both runs used Claude Code CLI, not Copilot CLI —
efficiency for 06 is scored against 07 (same runner) plus absolute
wall-clock; `/cost` dollars are not comparable to run-1 AI credits.

| repo | correctness | code quality | tests | docs | process | efficiency | autonomy | acceptance | weighted total |
|---|---|---|---|---|---|---|---|---|---|
| 06-gstack | 5 | 5 | 5 | 5 | 5 | 2 | 5 | 12/12 | 94/100 |
| 07-baseline-claude | 5 | 4 | 5 | 5 | 5 | 5 | 5 | 12/12 | 96/100 |

Cross-cutting observation for the consensus session (not a ranking):
acceptance still does not discriminate — now 7/7 repos at 12/12. Both
Claude-runner artifacts handle the naive-timestamp edge that crashed
02/03/05, and both carry a self-review loop that demonstrably changed the
outcome (06's TODOS.md deferrals; 07's review-fix commit 00f8445). The
discriminating signal on this runner was cost: 06 spent 2.2× the wall-clock
and ~3× the tokens of 07 for margins (hostile-input hardening, locale-proof
parsing, 137 vs 77 tests) the spec never asks for — the same
robustness-for-spend trade run 01 made on the Copilot runner.
