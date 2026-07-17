# Consensus prompt — Fable(2): run_02.1 fix-round reconciliation

Both graders (`fable1`, `sol`) have committed fix-round sheets for the
six participating repos. You are the consensus session for the fix
round ONLY — run_02 build grades in `final.md` are frozen. `$EVAL` =
`/home/ken/src/ai-agents/harness-eval` on kai (fish shell — use
`ssh kai bash -s <<'EOF'` heredocs).

## Inputs (you are the first session allowed to read all of it)

- Fix sheets + summaries: `grades/*-fix-*.md`, `grades/summary-fix-*.md`
- `FIX-ROUND.md` (protocol, rubric, E1 incident + corrections),
  `prompts/fix-00-bug-report.md`, `runs/*-fix-*`
- `grades/precedents.md`; run_02 `final.md` + `reconcile/` for protocol
  precedent and build-context

## Procedure (mirror the established consensus protocol)

1. **Factual disputes** (rare — acceptance was uniform): adjudicate in
   a fresh clone; document in `grades/adjudication.md` (append a fix
   -round section).
2. **Reconcile ≥2 gaps** per dimension, one round, ruling documented in
   `grades/reconcile/NN-fix-<dim>.md`. Gaps ≤1: mean.
3. **Write the fix-round consensus into `grades/final.md`** as a new
   `## Fix round (run_02.1)` section: 6 repos × 5 dimensions table,
   weighted totals, per-grader raws, reconciliation notes, and the two
   standing context notes — 03 absent-by-passing (with the ungraded 99
   shakedown also passing: 2 of 8 implementations chose strict
   parsing), and E1's environment note (ambient MCP uniformly absent
   for copilot cells; phoenix spine present).
4. **Append any adjudicated interpretations** to `grades/precedents.md`.
5. Commit: `git add _eval/run_02/grades && git commit -m
   "grades: run_02.1 fix-round consensus"`.

## Scope limits

No whitepaper, no infographic, no README, no publish import — the
combined run_02 + 02.1 report is a separate session. Report the
fix-round table in chat and stop.
