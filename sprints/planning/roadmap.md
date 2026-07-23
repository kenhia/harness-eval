# Roadmap

> The general plan for harness-eval. Detail lives in the sprint records;
> the eval's own design docs are `_eval/README.md` and
> `_eval/ADDING-A-HARNESS.md`. Lessons driving all of this:
> `_eval/run_01/report/lessons-learned.md`.

## Now — sprint 006: BYOK-kvllm-gemma (the third rung)

Sprint 005 shipped (run_03 Haiku tier complete, graded, published —
see sprint 005 + `_eval/run_03/`). The capability ladder now has two
rungs and a clear pattern; the third rung tests whether it holds at
open-weight scale.

- Copilot CLI BYOK → kvllm's vLLM endpoint serving Gemma on the 5090:
  same seven cells, same loglens scenario, same executable suite; only
  the model swaps (a `cells.tsv` edit + a profile).
- Gate on a bare-control tool-loop shakedown — if the model can't drive
  the runner's tool loop, the tier is untestable and that is itself the
  finding.
- **Free inference makes this the place to finally buy N≥3 reps** —
  the top outstanding debt (lesson 43).
- Report the same cross-tier delta statistic; three rungs turn a
  two-point comparison into a trend.

## Later — task-type matrix

- **Task-type matrix, one cell per run group** (v2 design; run_02.1
  covered the bug-fix cell organically):
  - next task-type run: **behavior-preserving refactor** on a messy repo, or
    **resume-from-handoff** (a deliberately half-finished run 02 repo) —
    lesson 27 argues for pressing the resume axis first;
  - lesson 25 task idea: a build where a load-bearing dependency
    decision is explicit and scoreable.

## Later / Ideas

- **ksandbox runs** (proposed 2026-07-17): containerize cells on the
  ksandbox Docker host (kvllm precedent: docker context over ssh,
  per-episode containers, MaxStartups fix known). Kills the environment
  hoops structurally: image-pinned runner versions (no more mid-field
  CLI drift), no ambient env leaks, mechanical environment manifest
  (image digest), safe parallel reps, blast-radius isolation for
  model-generated code. Path: `run-eval.sh --sandbox`, pilot Claude
  cells (file-based auth — trivial), spike Copilot auth in a headless
  container (keyring-less login fallback + does /mcp add approval
  survive?) before that side. Containers, not VMs; needs egress (API,
  crates.io, tailnet MCP) so isolation ≠ hermeticity.
- **Model-capability axis — rung 1 SHIPPED** (run_03, Haiku 4.5).
  Remaining rungs: open-weight local models (sprint 006), and possibly
  a mid-tier (Sonnet) to turn the ladder into a curve. Method settled:
  each tier is its own field with own controls and own calibrated
  scale; the cross-tier statistic is the harness-minus-control delta.

- **N ≥ 3 reps per cell**; medians + spread, not single runs. Variance
  is now measured, not hypothetical: the run 02 control cell ran twice
  (99-shakedown vs 07) at 52m/207k vs 36m/145k output tokens — 44%
  wall-clock spread, identical throughput, leakage forensically ruled
  out. Personal-project budget means ≈21+ runs will take **weeks**:
  publish run 02 as preliminary N=1 (caveat stated prominently), then
  accumulate reps as time/money allow and refine. Needs the headless
  matrix driver (a justfile/script looping run-eval.sh `--headless`) —
  mechanics exist, orchestration doesn't.
- **Harness × runner axis** where dual-target harnesses exist (kprojects,
  possibly others) — lesson 15 showed the runner effect is as large as
  the harness effect.
- Third grader identity, or CI-run acceptance so graders never touch
  objective checks at all.
- Model axis (same harness, different model) once per-cell costs are
  understood.
- Automated secret-scan step folded into the publish-import tooling.

