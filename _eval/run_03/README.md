# run_03 — Haiku 4.5 tier, loglens scenario (STATUS: setup)

First rung of the **model-capability axis** (Mark's question: does
harness value grow as capability drops — or is there a floor where
machinery hurts?). Same seven cells, same runners and harnesses as
runs 1/02; only the model changes: **Claude Haiku 4.5**. Scenario is
run_01's frozen loglens spec verbatim (sub-frontier tiers stay on
loglens by decision 2026-07-18; feedhub only if Haiku aces this).

**This tier is its own field**: own controls, own grading scale, ranked
independently. The cross-tier statistic is the **harness-minus-control
delta** compared against run_01's frontier tier — never raw scores.
Acceptance tallies (same executable suite semantics) ARE comparable
across tiers.

## Contents

- `prompts/` — run_01's loglens prompts, byte-identical (frozen)
- `acceptance.md` + `acceptance/` — executable A1–A12 port + hard tier
  (H1–H9), **retro-validated against the seven graded run_01 trees**:
  core 7/7 at 12/12; H9 mechanically reproduces run 1's
  naive-datetime finding (fails exactly 02/03/05). Freezes at the
  first tier run.
- `cells.tsv` — the matrix manifest (cell|runner|profile|model|prompt);
  drive with `_eval/bin/run-matrix.sh run_03`
- `rubric.md` — run_02 structure, correctness mapping for 12-core/9-hard
- `runs/`, `grades/` — as usual

## Setup status (2026-07-18)

- **Staging DONE** — all 7 repos at `pre-run`, clean trees, in
  `~/src/ai-agents/harness-eval-runs/run_03/`. Harness versions match
  run_02 exactly (StarterKit 2.6.3, KB source @ 34804ea, kprojects
  current, gstack 1.60.1.0, phoenix profile MCP verified) — so run_03
  vs run_01/02 isolates **model capability**, harness version held.
- **Haiku id CONFIRMED** — `claude-haiku-4.5` is in Copilot's own
  bundled model registry (`.cache/copilot/pkg/.../definitions/`); the
  Claude-runner cells use `claude-haiku-4-5-20251001`. cells.tsv is
  correct.
- Real-HOME leak check clean.

## Remaining before the field runs

1. **Model-mismatch guard**: after the first cell, check the runlog's
   session-metrics model line is Haiku, not a silent default fallback.
2. **Freeze** prompts + suite at first contender run; then
   `run-matrix.sh run_03` (resumable; serial).
3. Grading: fresh grader sessions, **calibrate to this tier's own
   controls** — do NOT anchor to run_01/run_02 sheets (different
   capability class); precedents still apply.

## Expected shapes worth pre-registering

- If Mark's hypothesis holds: harness cells beat same-tier controls by
  MORE than run_01's frontier deltas (which were ~0 to negative on
  greenfield).
- If the capability-floor story holds: heavy go-command harnesses
  (gstack /autoplan, phoenix goal loops) underperform their control —
  machinery the model can't drive becomes pure tax.
- Watch H9 and the hard tier: does Haiku fall into the same traps
  frontier models did, or different ones?
