# Evaluation design

How the comparative harness evaluation works: isolation, run protocol,
tooling, grading. Everything specific to one run of the eval (prompts,
acceptance checks, rubric, grades, run logs, reports) lives in
`_eval/run_NN/`; this file plus [ADDING-A-HARNESS.md](ADDING-A-HARNESS.md)
are the living process docs.

## Layout

```
_eval/
  README.md            this file — the eval design
  ADDING-A-HARNESS.md  incremental process for adding a contender to a field
  bin/                 tooling (below)
  templates/           runlog-template.md
  profiles/            fake-HOME sandboxes (live tokens — gitignored, always)
  run_NN/              one eval run: prompts, rubric, acceptance, grades,
                       runs (logs+transcripts), report, grader_prompts
run-output/run_NN/     imported final trees of each contender's repo
~/src/ai-agents/harness-eval-runs/run_NN/   staging repos (outside this repo)
```

A "run group" (`run_NN`) is one comparable field: same spec, same
acceptance suite, frozen for its lifetime. Incremental expansions (like run
1.5 adding 06/07 to run 1) stay inside the same group.

## Environment isolation: fake-HOME profiles

Every run executes with `HOME` overridden to a profile directory —
`env HOME=_eval/profiles/<name> <runner> …` — so global config, harness
global pieces, session logs, and stray writes (`~/.gstack`, `~/.cache`) all
land in the profile, never in the real account. Both runners resolve `~`
through `$HOME`, so one mechanism covers Copilot CLI (`.copilot/` in the
profile) and Claude Code (`.claude/` + `.claude.json` at the profile root).
Run 1 used symlink-swapping via `bin/use-profile.sh` (now deprecated); run
02+ uses the HOME method for everything.

Rules (earned the hard way — see run_01 lessons 1, 16, 17):

- **One profile per environment flavor.** A harness-free control never
  shares a profile with anything: `clean` / `claude-clean` are
  harness-free; a harness with global pieces gets its own copy
  (`claude-<harness>`).
- **Build a profile under its final name** — installers bake absolute
  profile paths into hooks and compiled binaries.
- Claude profiles need `.claude/settings.json` with
  `permissions.defaultMode: "bypassPermissions"` (a HOME-sandbox does not
  inherit the real account's permission settings), and `.claude.json` at
  the profile **root**, not `.claude/.claude.json`.
- klams/korg MCP stays in all profiles (standard machine environment,
  available to every contender equally); personal skills/hooks/statusline
  stay out.
- Profiles hold live auth tokens — `_eval/profiles/` is gitignored, always.
- **Profile credentials go stale.** They're snapshots of the real
  account's OAuth state, and refresh tokens rotate — a profile that sat
  idle while other sessions refreshed the chain can hit "OAuth session
  expired and could not be refreshed". `run-eval.sh` preflight probes
  auth (cheap haiku call) and re-syncs `.credentials.json` from the real
  `~/.claude` automatically for Claude runners.
- **Copilot profiles: login + MCP approval are per-profile rituals.**
  Auth lives in the desktop keyring/session store (do `/login` once per
  profile, from your terminal); MCP servers imported from a
  pre-existing `mcp-config.json` are UNAPPROVED until re-added via the
  CLI's `/mcp add` (an approval gate GitHub enforces since 2026-07-16 —
  unapproved servers are blocked or silently hidden). gh-token
  injection (`run-eval.sh --inject-gh-token`) is a last resort: MCP is
  policy-blocked under gh-CLI tokens.
- **Fish users: never write `env HOME=~/...`** — fish does not expand
  `~` after `HOME=`, so the process gets a LITERAL `~` home and mirrors
  the profile tree into `./~/...` relative to cwd (including auth
  state, which then "logs out" when you delete the junk dir). Use
  `env HOME=$HOME/src/...` or an absolute path. `run-eval.sh` is
  unaffected (absolute paths internally).

## Tooling (`_eval/bin/`)

- **`new-run.sh <run-group> <NN-name> [--no-harness]`** — stamps out a
  staging repo (`git init`, empty `eval: clean repo baseline` commit;
  `--no-harness` tags `pre-run` immediately for control runs). Harness runs
  stop so you can install + commit + tag.
- **`run-eval.sh --runner claude|copilot --profile <p> --run-group run_NN
  --repo <NN-name> [--model <id>] [--prompt-file <f>] [--headless]`** —
  the run wrapper: preflight (profile shape, `pre-run` tag, clean tree,
  runner version), timed launch under the profile HOME, then auto-fills
  the run log's *Auto-captured* section from the session logs (timestamps,
  tokens/credits, diffstat, real-HOME leak canary). You fill only the
  *Manual* section: declared-done, interventions, observations, `/cost`
  paste for interactive Claude runs.
- **`run-acceptance.sh <run-group> <NN-name>`** — runs the run group's
  executable acceptance suite against one repo and archives the full
  output + core/hard tier tally to `_eval/<run-group>/runs/NN-acceptance.txt`.
- **`collect-session.py --runner claude|copilot <session files>`** — parses
  a session log into metrics by hand; `run-eval.sh` calls it automatically.
  Claude: `<profile>/.claude/projects/<slug>/*.jsonl` (+ `subagents/*`);
  Copilot: `<profile>/.copilot/session-state/<id>/events.jsonl` (has
  premium requests, AI credits, tokens, API duration, code-change stats).

## Run protocol (per contender)

1. `new-run.sh run_NN NN-<name>` → install harness (repo-local pieces) →
   commit `eval: install <harness> (<exact command>)` → `git tag pre-run`.
   Everything after `pre-run` is agent-authored — graders diff
   `pre-run..HEAD`.
2. Sanity-check the profile (cheap `claude -p` / `copilot -p`: what skills
   and MCP servers are visible?).
3. `run-eval.sh …` — hands off. Every intervention is logged verbatim in
   the run log and is scored.
4. When the agent declares done: do **not** fix anything; fill the Manual
   section; if the agent didn't commit, commit as
   `eval: post-run snapshot (agent did not commit)`.

## Grading

1. **Objective pass** — sealed acceptance checks (`run_NN/acceptance.md`),
   never shown to the working agents.
2. **Rubric pass** — two independent graders (Fable, GPT Sol) score each
   repo's `pre-run..HEAD` diff on `run_NN/rubric.md`, writing to
   `run_NN/grades/NN-<grader>.md`.
3. **Consensus** — adjudicate factual disputes; reconcile dimension gaps
   ≥2; final scores in `run_NN/grades/final.md`. Mean on ≤1 gaps.

New contenders join an existing field via the incremental process in
[ADDING-A-HARNESS.md](ADDING-A-HARNESS.md) (delta grading, frozen priors).

## Run 1 (POC, July 2026)

Five harnesses + two baseline controls across two runners, one small
greenfield Python CLI. Design details, contenders, and what it found:
[`run_01/report/whitepaper.md`](run_01/report/whitepaper.md); process
lessons that produced the current tooling:
[`run_01/report/lessons-learned.md`](run_01/report/lessons-learned.md).
Run 1 ran pre-tooling: Copilot runs used symlink-swapped profiles and
hand-filled run logs; 06/07 introduced the HOME method this design now
standardizes.
