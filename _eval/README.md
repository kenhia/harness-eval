# Comparative Harness Evaluation — POC run 1

> [!IMPORTANT]
> **Results from this POC are an initial exploration.** One run per harness
> (n = 1), a single command each, a single small greenfield project. Expect
> higher separation as the eval harness improves and covers a wider scope of
> tasks. See the repo-root README for the results and caveats.


Five identical greenfield tasks, one per harness, same model, same prompt body.
Graded blind-ish by two independent LLM reviewers plus an objective acceptance
checklist. Output of run 1: a white paper, an infographic, and a
lessons-learned that turns this POC into a repeatable evaluation harness.

## Contenders

| repo | harness | install (committed as `eval: install …`) | go command | harness scope |
|---|---|---|---|---|
| `01-atv-starterkit` | ATV-StarterKit 2.x | `npx atv-starterkit@latest init` (94 files) | `/lfg …` | repo-local (`.github/{skills,agents,hooks}`, `.atv`, `.gstack`) |
| `02-atv-phoenix` | ATV-Phoenix | `setup.py` → global Copilot CLI install | `/phoenix-goal "…"` | **global** (`~/.copilot` — via the `phoenix` profile; repo stays clean) |
| `03-working-skill-repo` | working-skill-repo (KB) | `kb-install.mjs --target repo --profile core --router skip` | `kb-start "…"` | repo-local (`.github/skills`, `AGENTS.md`) |
| `04-kprojects` | kprojects | `install.sh --agent both` | ambient (instructions block) | repo-local (`CLAUDE.md`, `.github/copilot-instructions.md`, `sprints/`) |
| `05-baseline` | none | — | raw prompt | — |

Each repo has an empty `eval: clean repo baseline` commit, then the harness
install commit, then a `pre-run` tag. **Everything after `pre-run` is
agent-authored** — graders diff `pre-run..HEAD`.

## Runner

**GitHub Copilot CLI on kai with the Claude Opus model selected** (`/model`).
Rationale: three of the four harnesses are Copilot-first (StarterKit and
working-skill-repo hook `.github/skills` + instructions; Phoenix only exists
as a Copilot CLI plugin). kprojects is dual-target so it works there too.
Running under Claude Code instead would silently disable most of what the ATV
harnesses install — the comparison would be meaningless. A future run can add
a second axis (same harnesses under Claude Code where supported).

## Environment isolation (why profiles exist)

kai's live `~/.copilot` had Phoenix skills + agent + MCP server globally
installed — meaning *every* run, including baseline, would have had Phoenix
active. Fix: swappable profiles under `_eval/profiles/`, switched by
`_eval/bin/use-profile.sh`:

- `clean` — auth/config/permissions + klams/korg MCP servers. **No skills, no
  agents.** Used for runs 01, 03, 04, 05.
- `phoenix` — `clean` + the 19 phoenix skills, phoenix + token-master agents,
  phoenix MCP server, token-master state. Used for run 02.
- `original` — created by `use-profile.sh bootstrap`; your real day-to-day
  config. Switch back to it when not running evals.

Personal skills (ken-constitution, kwi-workitems, gratch, sprint-*,
refill-queue) are excluded from *both* eval profiles to minimize confounds.
klams/korg MCP stays in both: it is part of the standard machine environment
and available to every contender equally.

**Profiles contain live auth + bearer tokens — never commit them** (gitignored).

## Run protocol (per repo, fresh session each)

1. One-time: `_eval/bin/use-profile.sh bootstrap` (moves `~/.copilot` to
   `profiles/original`, replaces it with a symlink).
2. `_eval/bin/use-profile.sh clean` — or `phoenix` for run 02 only.
3. `cd` into the run repo; start `copilot`; select Claude Opus via `/model`.
4. Paste the entire matching file from `_eval/prompts/NN-*.md`. Note start time.
5. Hands off. If you must intervene, log every intervention verbatim in the
   run log — interventions are scored.
6. When the agent declares done: note end time, do **not** fix anything, fill
   in `_eval/runs/NN-runlog.md`, ensure final state is committed (if the agent
   didn't commit, commit as `eval: post-run snapshot (agent did not commit)`).
7. `use-profile.sh original` when finished for the day.

## Grading

1. **Objective pass** — a grader executes `_eval/acceptance.md` (sealed; never
   shown to the working agents) against each repo → functional score.
2. **Rubric pass** — Fable and GPT Sol independently grade each repo's
   `pre-run..HEAD` diff using `_eval/rubric.md`, writing per-dimension scores
   + justification to `_eval/grades/NN-<grader>.md`.
3. Reconciliation: where graders disagree by ≥2 points on a dimension, they
   argue it out; final scores recorded in `_eval/grades/final.md`.

## Deliverables (after grading)

- White paper: method, results table, per-harness narrative, threats to
  validity.
- Infographic: headline scores, cost/efficiency, one-line verdicts.
- Lessons learned → design for the real evaluation harness (see
  `notes/lessons.md`, seeded during setup).

## Expanding the field (run 1.5+)

New contenders are added incrementally — one run + a delta grading pass +
a short delta consensus; prior grades are frozen, never re-derived. Process:
[ADDING-A-HARNESS.md](ADDING-A-HARNESS.md). First expansion, run 1.5:

- `06-gstack` (garrytan/gstack) — Claude-Code-native, introducing the
  **runner covariate**; go command: `/autoplan`; global piece sandboxed in
  the `claude-gstack` HOME-profile.
- `07-baseline-claude` — control: no harness on the Claude Code runner
  (`claude-clean` HOME-profile), so 05-vs-07 isolates the runner effect and
  06-vs-07 isolates gstack's contribution.

Staging repos live at `~/src/ai-agents/harness-eval-runs/` (main's run
dirs are flattened publish trees). Delta grading prompts:
`grader_prompts/{fable1,sol,fable2}-delta-06-07.md`.
