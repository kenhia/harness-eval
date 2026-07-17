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

## Completed (2026-07-17)

- Six fix runs, headless, zero interventions; all 26/26 + 3/3.
- Mid-round incident **E1** (Copilot MCP approval gate, misdiagnosed
  three times before probes isolated it): one run voided + rerun with
  verified spine; ambient MCP uniformly absent on copilot cells;
  tooling hardened (token scrubbing, profile logins, void protocol).
  Full forensics in `_eval/run_02/FIX-ROUND.md`.
- Delta grading + consensus: zero reconciliations (again). Fix field
  93.5–99: gstack 99 > bare Claude 98 > kprojects 96 > StarterKit 95 >
  Phoenix 94.5 > bare Copilot 94 — all four harness/control pairs to
  the harness.
- Combined report shipped: whitepaper, infographic (03/99
  absence-by-passing blurb), lessons 22–33, README rewrite.
- Publish import: `run-output/run_02/` + `history/run_02/*`,
  `pre-run/run_02/*`, `pre-fix/run_02/*` refs; secret scan clean.

## Headline findings

The build round's 22.5-point spread traces to one dependency default
(quick-xml vs roxmltree); six repos shipped and then identically fixed
the same bug; the fix round inverted the build round (machinery paid on
resume work). Executable acceptance produced zero grader
reconciliations across 79 dimension cells in two rounds.
