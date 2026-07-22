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

## Remaining

1. **Execute the Haiku field**: `run-matrix.sh run_03` (Ken — needs
   Copilot profile logins per E1; serial, resumable). Model-mismatch
   check on cell 1.
2. **Grade** against tier-own controls; report the cross-tier
   harness-minus-control delta vs run_01's frontier deltas (Mark's
   question).
3. (Own track) ksandbox Claude-side proof + Copilot-in-container auth
   spike.
