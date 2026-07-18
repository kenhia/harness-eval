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

## Remaining

1. **ksandbox pilot** — `run-eval.sh --sandbox` + Claude runner image.
   Ken actions: `claude setup-token` (one-time, interactive) and
   confirm `docker context sandbox` reachable from kai. Then the
   Copilot spike (keyring-less login persistence, /mcp approval
   survival) gates that runner's containerization.
2. **run_03 staging**: 7 staging repos + harness install refresh at
   current versions; verify Copilot's Haiku model id (manifest guess:
   `claude-haiku-4.5`); model-mismatch check on first cell.
3. **Freeze + execute** the Haiku field (`run-matrix.sh run_03`),
   grade against tier-own controls.
