# Grading rubric — harness eval POC run 1

Two graders (Fable, GPT Sol) score each repo **independently**, without
reading the other's grades. Grade only agent-authored work: the diff
`git diff pre-run..HEAD` plus the run log `_eval/runs/NN-runlog.md`.
Harness-installed files (everything at the `pre-run` tag) are out of scope
for code-quality judgments, but harness-*prescribed* artifacts the agent
produced (sprint docs, todo.md, handoffs, plans) count under Process.

## Dimensions (weighted, score each 0–5)

| # | dimension | weight | what 5 looks like |
|---|---|---|---|
| 1 | Functional correctness | 30 | All acceptance checks pass (`_eval/acceptance.md` results supplied by the objective pass — do not re-derive) |
| 2 | Code quality & simplicity | 20 | Idiomatic, appropriately small, no speculative abstraction, no dead code, sensible structure |
| 3 | Test quality | 15 | Tests exercise behavior incl. edge cases (malformed lines, exit codes, tie-breaks, JSON validity); would catch real regressions; not tautological |
| 4 | Documentation | 10 | README alone is enough for a stranger to install and use every subcommand |
| 5 | Process & git hygiene | 10 | Coherent commit sequence with meaningful messages; planning/review artifacts (if the harness prescribes them) are genuinely useful, not ceremony |
| 6 | Efficiency | 10 | Low wall-clock and low token burn for the result delivered (from run log); no thrashing, no redundant rework loops |
| 7 | Autonomy | 5 | Zero human interventions between prompt paste and done |

Weighted total = Σ(score/5 × weight) → 0–100.

## Grader instructions

1. Read the spec (`prompts/00-project-spec.md`), then the run log, then the
   diff. Run nothing destructive; you may run the project's own tests/tools.
2. Score every dimension with 2–4 sentences of justification citing specific
   files/commits. No dimension may be justified by "the harness is good/bad"
   — judge the artifact, not the brand.
3. Note the single best and single worst thing the agent did.
4. Write to `_eval/grades/NN-<grader>.md` using the score-sheet template below.
5. Do not read other repos' grades until all five of yours are written.

### Score sheet template

```markdown
# NN-<repo> — grader: <name>
| dim | score /5 | note |
|---|---|---|
| correctness | | |
| code quality | | |
| tests | | |
| docs | | |
| process | | |
| efficiency | | |
| autonomy | | |
**Weighted total:** NN/100
**Best thing:** …
**Worst thing:** …
**Narrative (≤150 words):** …
```

## Known biases to resist

- Harness artifacts self-identify (phoenix traces, kb handoffs, sprint docs)
  — true blinding is impossible. Compensate: justify scores only from the
  artifact quality.
- Verbosity bias: more docs/artifacts ≠ better process. Ceremony that didn't
  change the outcome scores *low* on Process, not high.
- Both graders must grade all five repos in a different order (Fable 01→05,
  GPT Sol 05→01) to spread anchoring.
