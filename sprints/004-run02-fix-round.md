# 004 — run_02.1 fix round + combined report

*Started 2026-07-16, after run_02 consensus (final: KB 93.5 tops a
71–93.5 field — the hard tier + Rust task finally discriminated).*

## Goal

Run the organic bug-fix round (six repos that share the quick-xml
malformed-XML defect fix their own codebases from a bug report), then
publish run 02 + 02.1 as one combined report.

## Decisions

- **Combined report** (Ken): whitepaper/infographic/README wait for 02.1;
  the narrative pairs the shared failure with the fix behaviors.
  Infographic carries an explicit blurb: 03 sits out the fix round
  because it passed (roxmltree, same strict choice as the ungraded 99
  shakedown — 2 of 8 implementations dodged the trap).
- **Bug report, not sealed tests**, with a reproduction sample that
  differs from the sealed fixture (no special-casing path to a pass).
- **Sealed fix addendum** (F1 alt-truncation, F2 empty-valid-feed-ok,
  F3 error-clears-on-recovery) gated behind FIX_ROUND=1; frozen before
  the first fix run; validated 3/3 against 99 with no changes — strict
  parsing is the target state.

## Shipped in setup

Protocol (`_eval/run_02/FIX-ROUND.md` incl. fix-delta rubric), bug
report + 6 prompts (`prompts/NN-fix.md`), addendum tests, tooling
(`run-eval.sh --tag/--suffix`, `run-acceptance.sh --fix`), `pre-fix`
tags in all six staging repos, `.scratch/NN-fix-cmds.txt` helpers.

## Remaining

1. Six fix runs (Ken, serial, headless).
2. Fix-delta grading prompts + two graders + short consensus.
3. Combined report: whitepaper, infographic (with the 03/99 blurb),
   README results, publish import (post-02.1 so trees include fixes).
