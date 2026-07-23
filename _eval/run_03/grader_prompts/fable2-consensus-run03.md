# Consensus prompt — Fable(2): run_03 (Haiku tier) reconciliation

Both graders (`fable1`, `sol`) have committed rubric sheets for all
seven run_03 repos. You are the consensus session. `$EVAL` =
`/home/ken/src/ai-agents/harness-eval` on kai (fish shell — use
`ssh kai bash -s <<'EOF'` heredocs).

## Inputs (you are the first session allowed to read all of it)

- Both graders' sheets + summaries: `$EVAL/_eval/run_03/grades/*`
- `run_03/rubric.md` (incl. the mechanical correctness mapping),
  `README.md`, `DEFECTS.md` (S1 layout, I1 voided cell), `runs/*`
- `run_01/grades/precedents.md`; run_02's `final.md` for protocol shape

## Procedure

1. **Factual disputes first** — correctness is mechanical here (mapping
   table + supplied acceptance), so disputes should be rare. Adjudicate
   any in a fresh clone; document in `grades/adjudication.md`.
2. **Reconcile ≥2 gaps** per dimension (one round, ruling documented in
   `grades/reconcile/NN-<dim>.md`); gaps ≤1 → mean.
3. **Write `grades/final.md`**: consensus table (7 × 7 + weighted
   totals), ranking, per-grader raws, reconciliation notes, acceptance
   tallies (core n/12, hard n/9), and these standing notes:
   - **Tier scale**: run_03 scores are calibrated to this tier's own
     controls and are **not comparable point-for-point** to
     run_01/run_02 totals.
   - **Cross-tier statistic**: compute and report the
     **harness-minus-control delta** per cell (Copilot cells vs 05,
     Claude cell 06 vs 07) alongside run_01's frontier deltas for the
     same harnesses. This is the deliverable that answers the tier's
     motivating question — do NOT compare raw totals across tiers.
   - **S1/I1**: layout nesting (01, 03) treated as process observation;
     cell 05 is a clean rerun after a voided interrupted attempt.
4. **Append** any adjudicated interpretations to a run_03
   `grades/precedents.md`.
5. Commit: `git add _eval/run_03/grades && git commit -m
   "grades: run_03 Haiku-tier consensus — final scores"`.

## Scope limits

Consensus only — no whitepaper, no infographic, no README, no publish
import. Report the final table plus the delta comparison in chat, and
stop.
