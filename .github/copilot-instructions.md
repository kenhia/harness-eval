<!-- kproject:begin — managed by kprojects; do not edit inside this block -->
## kproject conventions

This project uses the kproject minimal harness
(<https://github.com/kenhia/kprojects>). Keep context small; prefer doing
over ceremony.

### Layout

- `sprints/` — the project's evolution, one record per PR-sized unit of
  work (a "sprint")
  - `planning/` — planning docs; at minimum `roadmap.md` (the general plan)
  - `review/` — more formal reviews as the project matures
  - sprint records: `###-<short-name>.md` for small projects, or a
    `###-<short-name>/` directory of files for larger/more formal ones
  - a sprint record is one informal narrative: goal, decisions, what
    shipped, follow-ups — written during the sprint, not after
  - projects that deploy end the record with a `## Deployed` section:
    what shipped, where, when, and what was verified live — appended
    after the deploy, not predicted before it
- `docs/` — project documentation, architecture, usage
- `.scratch/` — git-ignored scratch space for user or agent ephemera;
  use it instead of /tmp
- `justfile` — dev recipes; default recipe is `@just --list`; `just check`
  runs the CI gates; `just deploy` (or variants) if the project deploys
- `.env` — git-ignored; tokens and environment vars

### Workflow

- One sprint ≈ one PR. Sprint proposals and work items are managed in
  `korg`; durable cross-project knowledge goes in `klams`.
- Mark each work item resolved as its work completes — don't batch the
  resolutions into sprint-ship. A proposal's progress should be readable
  while the sprint is running, which is the only time it is useful.
- If the korg or klams MCP tools are unavailable in your session, say so
  up front — don't silently work around missing infrastructure.
- TDD preferred: write the failing test first when practical.

### Tooling preferences

- No stack the harness could name, so `just check` is yours to write. Ask
  what this repo can actually get wrong — a documents repo's failure mode is
  a stale cross-reference, not a type error
- Add no dependency to make a gate: a stdlib script or a shell one-liner
  keeps a repo that had no dependencies still having none
- Skip what isn't yours to verify — external URLs, machine-local paths
- **Negative-test it.** Plant the error the gate exists to catch and watch it
  exit 1. A gate never seen to fail is not a gate, and the seeded placeholder
  fails on purpose until you replace it
- License is MIT unless specifically directed otherwise
<!-- kproject:end -->

## Project

Comparative evaluation of AI coding-agent harnesses: same model, same
prompt, one run per harness across runners, graded by two independent AI
reviewers with consensus. This repo holds the eval process, tooling,
results, and imported run trees — it is NOT the code under test.

- Read first: `_eval/README.md` (eval design + tooling),
  `_eval/ADDING-A-HARNESS.md` (incremental process),
  `_eval/run_01/report/lessons-learned.md` (why the design is what it is),
  `sprints/planning/roadmap.md` (what's next).
- Layout: `_eval/run_NN/` = everything for one eval run (prompts, rubric,
  acceptance, grades, runs, report); `run-output/run_NN/` = imported final
  trees of contender repos; staging repos live OUTSIDE this repo at
  `~/src/ai-agents/harness-eval-runs/run_NN/`.
- Tooling: `_eval/bin/new-run.sh` (staging repo), `_eval/bin/run-eval.sh`
  (preflight + timed launch + auto run log), `_eval/bin/collect-session.py`
  (session-log metrics). `just check` syntax-checks these.
- Gotchas:
  - `run-output/` trees and `_eval/run_NN/{grades,grader_prompts,runs}` are
    **frozen artifacts** — never "fix" or lint them. Sealed files
    (acceptance.md, grader materials) are never shown to working agents.
  - `_eval/profiles/` holds fake-HOME sandboxes with LIVE TOKENS —
    gitignored; never commit or read credentials from them.
  - Historical refs: branches `history/...`, tags `pre-run/...` — don't
    prune them.
