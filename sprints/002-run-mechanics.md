# 002 — run mechanics + per-run structure

*2026-07-16. Addresses the run-mechanics friction from run 1 (lessons 8,
16, 19) and preps the repo for multiple run groups.*

## Goal

Make a run cheap and mechanically captured: one command per run, run logs
that fill themselves, and a repo layout that survives run 02+.

## What shipped

- **Fake-HOME everywhere** — verified Copilot CLI resolves `~/.copilot`
  through `$HOME` (same as Claude Code), so the HOME-override method used
  for runs 06/07 now covers both runners; `use-profile.sh` symlink
  swapping is deprecated (kept for the historical record).
- **`_eval/bin/new-run.sh`** — stamps out staging repos under
  `~/src/ai-agents/harness-eval-runs/<run-group>/` (baseline commit, and
  `pre-run` tag for `--no-harness` controls).
- **`_eval/bin/run-eval.sh`** — preflight (profile shape, pre-run tag,
  clean tree, runner version) → timed launch under the profile HOME →
  auto-fills the run log's *Auto-captured* section: ISO timestamps, wall
  clock, session metrics, `git diff --stat`, real-HOME leak canary. The
  human fills only the *Manual* section (lesson 19's decay problem).
  Smoke-tested end to end with a headless haiku run.
- **`_eval/bin/collect-session.py`** — parses both runners' session logs.
  Validated against known run-1 numbers: Copilot `events.jsonl` → 215.6
  credits / 15 premium requests for run 03 (matches the hand-recorded 216
  cr); Claude project jsonl → 122.7k output tokens for run 06 (matches the
  `/cost` paste exactly, after last-wins dedupe of per-content-block
  records). Timestamps come from the logs — no more stopwatch.
- **Per-run structure** — run trees moved to `run-output/run_01/`;
  run-specific eval material (prompts, rubric, acceptance, grades,
  grader_prompts, runs, report, notes) moved to `_eval/run_01/`; shared
  process docs, `bin/`, `templates/`, `profiles/` stay at `_eval/` top.
  Staging mirror: `harness-eval-runs/run_01/`. New runs namespace history
  refs as `history/run_NN/…`, `pre-run/run_NN/…`.
- **`grades/precedents.md`** — adjudicated *interpretations* now travel to
  future delta graders (lesson 18's wasted re-litigation); seeded with the
  half-open-window ruling.
- **kproject harness** — sprint layout, CLAUDE.md /
  copilot-instructions.md with frozen-artifact + live-token gotchas,
  `just check` wired to syntax-check the tooling.

## Decisions

- Run groups are `run_NN`; contender numbering restarts per group.
- Cost for interactive Claude runs stays a one-line manual `/cost` paste;
  token sums from the session log are the authoritative auto-captured
  numbers (session logs carry no dollar figures).

## Follow-ups

- Executable sealed acceptance suite + hard tier — the single
  highest-value v2 change (lessons 9, 21); goes with run 02's spec.
- Headless-matrix automation (`--headless` exists; a matrix driver does
  not). See `planning/roadmap.md`.
