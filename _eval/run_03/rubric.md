# Grading rubric — run_03 (loglens, Haiku 4.5 tier)

Two graders (Fable, GPT Sol) score each repo **independently**, without
reading the other's grades. Grade only agent-authored work: the diff
`git diff pre-run..HEAD` plus the run log `_eval/run_03/runs/NN-runlog.md`.
Harness-installed files (everything at the `pre-run` tag) are out of scope
for code-quality judgments, but harness-*prescribed* artifacts the agent
produced (sprint docs, todo.md, handoffs, plans) count under Process.

Changes from run_01's rubric: correctness comes from the **executable
acceptance suite** (core tier — supplied, never re-derived by graders);
the hard tier feeds the same dimension's upper half; efficiency is
anchored to the **same-runner control**, not the field. Weights unchanged
(they earned their keep in run 1).

## Dimensions (weighted, score each 0–5)

| # | dimension | weight | what 5 looks like |
|---|---|---|---|
| 1 | Functional correctness | 30 | Core 12/12 gets a 4; the fifth point tracks the hard tier (≥8/9 ≈ 5). Core failures cap this dimension ≤3. Results supplied by the acceptance run — do not re-derive |
| 2 | Code quality & simplicity | 20 | Idiomatic Python, sensible module split, no speculative abstraction, no dead code, clean error handling (clean error handling) |
| 3 | Test quality | 15 | Repo's own tests exercise behavior incl. edge cases (malformed lines, exit codes, tie-breaks, JSON validity, timezone handling); would catch regressions; not tautological |
| 4 | Documentation | 10 | README alone is enough for a stranger to install and use every subcommand |
| 5 | Process & git hygiene | 10 | Coherent commit sequence with meaningful messages; planning/review artifacts (if the harness prescribes them) genuinely useful, not ceremony |
| 6 | Efficiency | 10 | Anchored to the same-runner control (05 for Copilot, 07 for Claude Code): control-comparable cost/wall-clock = 5; ≤1.5× = 4; ≤2× = 3; ≤3× = 2; >3× = 1, absent justifying quality delta |
| 7 | Autonomy | 5 | Zero human interventions between prompt and done |

Weighted total = Σ(score/5 × weight) → 0–100.

## Grader instructions

1. Read the spec (`prompts/00-project-spec.md`), then the acceptance
   results supplied to you, then the run log, then the diff. Run nothing
   destructive; you may run the project's own tests/tools.
2. Score every dimension with 2–4 sentences of justification citing
   specific files/commits. No dimension may be justified by "the harness
   is good/bad" — judge the artifact, not the brand.
3. **Read `grades/precedents.md` first** (adjudicated interpretations
   carry forward; do not re-litigate them).
4. Note the single best and single worst thing the agent did.
5. Write to `_eval/run_03/grades/NN-<grader>.md` using the run_01 score
   sheet template.
6. Do not read other repos' grades until all of yours are written.

## Known biases to resist

- Harness artifacts self-identify — true blinding is impossible.
  Compensate: justify scores only from artifact quality.
- Verbosity bias: more docs/artifacts ≠ better process. Ceremony that
  didn't change the outcome scores *low* on Process, not high.
- Graders grade the repos in opposite orders (Fable 01→07, Sol 07→01) to
  spread anchoring.
- Rust-specific: robustness beyond spec is not free quality — margins the
  spec never asked for cost tokens (run 1.5, lesson: gstack). Score what
  the spec bought.
