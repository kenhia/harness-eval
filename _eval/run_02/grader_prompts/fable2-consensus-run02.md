# Consensus prompt — Fable(2): run_02 reconciliation

Both graders (`fable1`, `sol`) have committed rubric sheets for all seven
run_02 repos. You are the consensus session. `$EVAL` =
`/home/ken/src/ai-agents/harness-eval` on kai (fish shell — use
`ssh kai bash -s <<'EOF'` heredocs for remote work).

## Inputs (read all of it — you are the first session allowed to)

- Both graders' sheets + summaries: `$EVAL/_eval/run_02/grades/*`
- `$EVAL/_eval/run_02/rubric.md`, `prompts/00-project-spec.md`,
  `README.md` (incl. Suite defect log + Report notes), `runs/*`
- `$EVAL/_eval/run_02/grades/precedents.md` and run_01's
  `final.md`/`reconcile/` for protocol precedent

## Procedure (mirror run 1's consensus protocol)

1. **Factual disputes first.** Correctness is mechanical this run
   (executable acceptance) — disputes should be rare. Where graders
   disagree on a fact (not a judgment), adjudicate in a fresh clone,
   document in `grades/acceptance-adjudication.md` (append run_02
   section) or a new `grades/adjudication.md`.
2. **Reconcile score gaps ≥2** per dimension: both graders' arguments,
   your ruling, one round. Gaps ≤1: mean. Write
   `grades/reconcile/NN-<dim>.md` per reconciled cell.
3. **Write `grades/final.md`**: consensus table (7 repos × 7 dimensions,
   weighted totals), per-grader raw scores, reconciliation notes,
   acceptance tallies (core n/14, hard n/12).
4. **Append to `grades/precedents.md`** any interpretation you
   adjudicated that future graders must inherit.
5. **Commit**: `git add _eval/run_02/grades && git commit -m
   "grades: run_02 consensus — final scores"`.

## Scope limits

Consensus only: no whitepaper, no README results table, no infographic,
no publish import — those happen in a separate session after final.md
lands. Report the final table in chat and stop.
