# Roadmap

> The general plan for harness-eval. Detail lives in the sprint records;
> the eval's own design docs are `_eval/README.md` and
> `_eval/ADDING-A-HARNESS.md`. Lessons driving all of this:
> `_eval/run_01/report/lessons-learned.md`.

## Now — sprint 005: reps infrastructure + the Haiku tier (decided 2026-07-18)

One unit: exercise new isolation on cheap runs, produce the first reps,
answer Mark's rung-1 question.

1. **Matrix driver** — loop run-eval.sh over a run group's prefixes,
   serial by default; **model + provider endpoint are parameters** (the
   gemma tier next sprint must be a config change, not a tooling
   change).
2. **ksandbox pilot** — `run-eval.sh --sandbox` with Claude cells
   first. Auth: one-time `claude setup-token` on kai → long-lived token
   in the sandbox env (no per-run manual auth, no credential-snapshot
   rotation). Copilot side: one-container auth spike (keyring-less
   login persistence + does /mcp add approval survive?) before
   committing it.
3. **Haiku 4.5 tier on the run_01 loglens scenario** (per Ken: stick to
   loglens for sub-frontier tiers; revisit feedhub only if Haiku rocks
   it). Prereq: **port loglens acceptance to an executable suite**
   (run_02 pytest pattern, core + hard tiers) and validate it against
   the seven graded `run-output/run_01/` trees — which doubles as a
   retroactive would-it-have-discriminated-run-1 finding.

## Next — sprint 006: BYOK-kvllm-gemma tier

Copilot CLI BYOK -> kvllm's vLLM endpoint serving Gemma on the 5090:
same runner + harnesses, only the model swaps. Gate on a bare-control
tool-loop shakedown; own controls, own scale, harness-minus-control
delta is the cross-tier statistic; loglens scenario; free inference +
ksandbox parallelism make this the N>=3 reps proving ground.

## Later — task-type matrix

- **Task-type matrix, one cell per run group** (v2 design; run_02.1
  covered the bug-fix cell organically):
  - run 03: **behavior-preserving refactor** on a messy repo, or
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
- **Model-capability axis** (Mark's question, 2026-07-17): does harness
  value grow as model capability drops — or is there a floor where
  machinery hurts? Each tier = its own field with own controls, ranked
  independently; cross-tier statistic = harness-minus-control DELTA;
  executable acceptance tallies compare objectively across tiers.
  Rung 1 (cheap, no infra): Haiku 4.5 tier on existing runners via
  --model. Rung 2: local models via Copilot CLI BYOK -> kvllm's vLLM
  endpoint (same runner + harnesses, only the model swaps); gate each
  model on a bare-control tool-loop shakedown. Use run 1's loglens spec
  for lower tiers (feedhub would zero them out) — port it an executable
  acceptance suite. Synergy: free local inference + ksandbox
  parallelism = the affordable place to pioneer N>=3 reps.

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

