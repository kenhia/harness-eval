# 005 — reps infrastructure + the Haiku tier

*Started 2026-07-18. Scope locked in roadmap: matrix driver + ksandbox
pilot + Haiku 4.5 tier on the loglens scenario; sprint 006 =
BYOK-kvllm-gemma (tooling here stays model/provider-agnostic for it).*

## Shipped so far

- **Loglens executable acceptance** (`_eval/run_03/acceptance/`):
  A1–A12 ported to the run_02 pytest pattern + hard tier H1–H9.
  **Retro-validated against the seven graded run_01 trees** — core 7/7
  at 12/12 (calibration target met after fixing three suite bugs:
  VIRTUAL_ENV leakage into the contender's `uv run`, malformed-line
  detection weaker than real parsers, hardcoded `--format` placement).
  **Retro-finding:** H9 (naive `--since` vs offset-carrying logs)
  fails exactly the three repos run 1's graders hand-flagged for the
  naive-datetime crash (02, 03, 05) — the executable hard tier
  discriminates run 1's dead heat mechanically, and P1 (window
  boundaries) is mechanized in H4.
- **Matrix driver** (`_eval/bin/run-matrix.sh`): serial, resumable
  (skips cells with existing runlogs), stops on run failure, treats
  acceptance failures as data. Cells manifest
  (`_eval/<group>/cells.tsv`) carries per-cell model — the capability
  axis is a manifest edit; provider config stays a profile concern
  (BYOK next sprint = new profile, same driver).
- **run_03 skeleton**: prompts (run_01's, byte-identical), cells.tsv
  (Haiku ids per runner), rubric (12-core/9-hard correctness mapping),
  README with pre-registered expected shapes for Mark's hypothesis vs
  the capability-floor story.

## ksandbox — corrected + descoped (2026-07-18)

Investigated the two failing Ken-commands; both were my error and they
reframed the design (full note: `_eval/KSANDBOX.md`). Reality: **kai is
the controller, ksandbox a Docker host driven over ssh** (context on
kai, verified `docker -c sandbox run` works; ksandbox has no
claude/copilot by design — runners live in images, auth injected). The
`claude setup-token` action runs **on kai**, not ksandbox.

**Descoped from this sprint.** run_03 is 5/7 Copilot cells, and
Copilot-in-container auth is the unsolved spike (E1's keyring/approval
tangle). A Claude-only sandbox can't run the majority-Copilot field, so
fake-HOME stays the only full-field path. ksandbox becomes its own
track (Claude-side proof + Copilot spike); it does not block run_03.

## run_03 staging — DONE (2026-07-18)

All 7 repos staged, `pre-run` tagged, harness versions held identical to
run_02 (so the tier comparison isolates model capability). Haiku id
confirmed from Copilot's bundled registry. Details in
`_eval/run_03/README.md`.

## Completed (2026-07-23)

- **Haiku field executed** (7 cells, headless, zero interventions;
  model verified Haiku in every session's metrics). Incidents handled:
  **I1** (controller ssh death mid-cell 05 → voided + resumed),
  **S1** (suite assumed repo-root project; two nested repos scored
  0/12 → fixed, nesting reclassified as a process observation),
  **S2** (JSON fallback matched argparse wording only, failing
  click-based CLIs — *found by a grader mid-grading*, fixed, 03 moved
  to a full pass, both affected cells delta re-graded).
- **Graded + consensus**: zero reconciliations (third consecutive
  round); field 50.25–77.5 on a tier-calibrated scale.
- **The finding** (`report/whitepaper.md`): harness value grows as
  capability drops **for light convention/skill harnesses** (KB 0 →
  +15.25, kprojects +0.5 → +7.75) and *shrinks* for heavy
  go-command harnesses (Phoenix −3 → −12, gstack −8 → −9.25).
  Encoded knowledge substitutes for capability; an autonomous process
  demands it.
- **Tooling added**: `run-matrix.sh`, `vet-grades.py`, loglens
  executable acceptance (retro-validated against run_01's seven graded
  trees, where it also retroactively discriminates run 1's dead heat).
- **Published**: whitepaper, infographic (validated diverging palette,
  dumbbell cross-tier chart), lessons 34–44, README, `run-output/run_03`
  + history/pre-run refs, secret scan clean.

## Remaining

- (Own track) ksandbox Claude-side proof + Copilot-in-container auth
  spike — see `_eval/KSANDBOX.md`.
