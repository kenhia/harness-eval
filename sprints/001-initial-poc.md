# 001 — initial POC (runs 1 + 1.5)

*2026-07-14 → 2026-07-15. Retrospective record — this sprint predates the
sprint layout. This documents the infrastructure we built, not the results
(those live in `_eval/run_01/report/`).*

## Goal

Prove the comparative-eval concept end to end: can we run the same
greenfield task under N harnesses, isolate environments well enough for
the comparison to mean anything, and grade the output reproducibly?

## What we built

- **Isolation machinery** — swappable Copilot profiles under
  `_eval/profiles/` (symlink method, `use-profile.sh`), discovered to be
  load-bearing on day one: kai's live `~/.copilot` had Phoenix globally
  installed and would have silently invalidated every run. Run 1.5
  introduced the better mechanism: fake-HOME profiles
  (`env HOME=<profile> <runner>`) for Claude Code.
- **Run protocol** — staging repos outside the eval repo; empty baseline
  commit → harness install commit → `pre-run` tag → agent-authored diff;
  zero-intervention discipline; per-run logs with timing/cost.
- **Prompt discipline** — one frozen project spec (`00-project-spec.md`,
  the loglens CLI), per-harness prompts differing only in the first "go"
  line, generated from `prefixes.txt`.
- **Grading pipeline** — sealed 12-check acceptance list + 7-dimension
  weighted rubric; two independent grader identities (fable1, GPT Sol)
  with per-grader sealed fixtures; consensus session adjudicating factual
  disputes and reconciling ≥2-point gaps; frozen-prior **delta grading**
  for incremental expansion (exercised by run 1.5: 06-gstack +
  07-baseline-claude, new runner covariate + same-runner control).
- **Publish format** — flattened final trees on `main`, full per-run
  history preserved on `history/*` branches and `pre-run/*` tags.
- **Reports** — white paper, infographic, and a 21-lesson
  lessons-learned that became the v2 design input.

## What we learned (headlines — full list in run_01/report/lessons-learned.md)

- Environment isolation *is* the experiment; preflight must prove it.
- The runner is a quality intervention of the same order as the harnesses
  (bare Claude Code topped the field).
- Objective checklists at frontier capability need a hard tier — 7/7 repos
  passed 12/12; all signal came from robustness edges.
- Manual timing/transcription is the weakest link (fixed in sprint 002).

## Follow-ups

→ sprint 002 (run mechanics + repo reorg), roadmap for run 02+.
