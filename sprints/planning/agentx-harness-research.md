# AgentX — step-0 harness research

> Prep for adding AgentX to the field, per `_eval/ADDING-A-HARNESS.md` §0.
> Researched 2026-07-23 against **v8.6.1** (tag pinned by `install.sh`;
> repo HEAD `406c9f0`). Source read locally, not just the README.
> Working clone: `.scratch/agentx-research/` (gitignored).
>
> Status: **research only — no profile built, no staging repo, nothing
> installed.** Blocked decisions are listed at the bottom.

Upstream: <https://github.com/jnPiyush/AgentX/> · Apache-2.0 · author jnPiyush.

## Verdict

**Viable, and the most invasive contender the field has seen.** The one
thing that looked like a hard blocker — a mandatory PowerShell 7
dependency on a machine with no `pwsh` — is solved cleanly by a portable
tarball inside the fake-HOME profile (verified working, below). Everything
else is manageable but needs three protocol decisions before a run.

## What it is

A "Harness-Oriented Architecture": 24 agent role definitions, 128 skill
files, 15 templates, 7 instruction files, a repo-local PowerShell state
CLI, git hooks that gate commits, and a VS Code extension. The knowledge
layer is all Markdown; the enforcement layer is PowerShell + bash hooks.

Marquee features: quality loop (min 5 iterations, one must be a subagent
review), Model Council (3-role multi-model deliberation on ADR/PRD/review),
context compaction, execution plans, learning capture.

## Install method and scope

```bash
curl -fsSL https://raw.githubusercontent.com/jnPiyush/AgentX/v8.6.1/install.sh | bash
```

**Repo-local, into cwd.** Default mode is `local` (no GitHub issues
required); `--mode github` prompts for repo/project — skipped
automatically when piped. Lands **~577 files** across
`.agentx/ .github/ .claude/ .cursor/ .vscode/ scripts/ packs/`, plus
`AGENTS.md`, `Skills.md`, and five `docs/*.md`. For scale: run 1's
largest install (ATV-StarterKit) was 94 files.

**No writes to `$HOME`.** Grepped `install.sh`, `install.ps1`, the
`.agentx/` runtime, `scripts/`, and the hooks: the only `~/` references
are documentation prose in `.agentx/mcp-server/README.md`. Nothing writes
to `~/.copilot`, `~/.claude`, or a `~/.agentx`. So unlike Phoenix or
gstack, AgentX has **no global piece** — a profile copy per harness isn't
strictly needed, though we'd still run under a fake HOME as standard.

Side effects worth knowing, all inside the repo:

- runs `git init` if there's no `.git` (harmless — staging repos have one)
- copies `pre-commit`, `post-commit`, `commit-msg` into `.git/hooks/`
- merges a marked block into `.gitignore` (see the landmine below)
- writes `.vscode/mcp.json` with the GitHub MCP HTTP server (or an empty
  one if the git remote looks like Azure DevOps)
- tries to `code --install-extension ms-azuretools.vscode-azure-mcp-server`,
  but only when it detects Azure signals — loglens/feedhub won't trip it

## The PowerShell dependency — solved

> **The README's "Bash on Linux/macOS" is not accurate.** README line 158
> lists prerequisites as *"PowerShell 7.4+ on Windows, or Bash on
> Linux/macOS"* — but that's a scoped bullet under **VS Code extension**
> setup, and it doesn't hold even there. Four places in the source
> contradict it, on every install path:
>
> | path | evidence |
> |---|---|
> | `install.sh` bootstrap | line 224 `ensure_dependency pwsh … \|\| exit 1` |
> | repo-local CLI | `.agentx/*.sh` are one-line `pwsh` wrappers; `agentx-cli.ps1` is 333 KB of PowerShell with **no bash equivalent** |
> | commit gates | `pre-commit:511` hard-fails: *"PowerShell 7+ (pwsh) is required for the mandatory scrub gate"* |
> | VS Code extension | `initializeInternals.ts:613` errors *"PowerShell 7.4+ (pwsh) is required"*; the wrappers it generates are `shell: 'pwsh'` with `#!/usr/bin/env pwsh` |
>
> The extension's `shellKind: 'pwsh' \| 'bash'` switch only picks which
> shell *launches the wrapper* — and on Linux that wrapper is the `.sh`
> file, which calls `pwsh` anyway. The only genuinely bash-only components
> are 3 of the 4 tool hooks. Treat `pwsh` as an unconditional requirement.
>
> This is exactly the README-vs-source gap §0 warns about, and it's worth
> a line in the run log's environment notes.

`install.sh` hard-requires `pwsh` 7.4+ and exits 1 without it. Every bash
entry point is a one-line wrapper:

```bash
# .agentx/agentx.sh
pwsh "$SCRIPT_DIR/agentx-cli.ps1" "$@"
```

The core is ~12.1k lines of PowerShell (`agentx-cli.ps1` 7,631 +
`agentic-runner.ps1` 4,503). The `pre-commit` hook hard-fails without
`pwsh` — *"[FAIL] PowerShell 7+ (pwsh) is required for the mandatory scrub
gate"* — so with no `pwsh`, the agent could neither run the mandatory
quality loop nor commit at all. AgentX's own `COUNCIL-401.md` concedes the
point: *"Copilot CLI/Cloud preinstall Node, not pwsh."*

kai has no `pwsh`, no `dotnet`, and `powershell` is not in Ubuntu 24.04's
apt repos — so the installer's `ensure_dependency pwsh powershell` would
run `sudo apt-get install -y powershell`, prompt for a password mid-run,
and fail.

**Fix: portable tarball into the profile.** Verified end to end today:

```
powershell-7.6.4-linux-x64.tar.gz  → self-contained, 191 MB extracted
$ ./pwsh/pwsh -NoProfile -Command '$PSVersionTable.PSVersion'   → 7.6.4
```

Then smoke-tested the actual AgentX CLI against it in a throwaway repo —
`loop status`, `loop start -p ...`, `config show` all work correctly on
Linux (the loop printed its five iteration focuses and gates). So AgentX
runs fine on Linux; it just needs `pwsh` on PATH.

Recommendation: unpack pwsh under `_eval/profiles/<profile>/.local/pwsh/`
and prepend it to PATH for the run. This keeps the fake-HOME isolation
intact and requires **no system change** (no `sudo snap install
powershell --classic`, no `record-machine-change`). Cost: 191 MB per
profile, already gitignored.

## Runner

AgentX targets Copilot, Claude Code, Cursor, and its own VS Code
extension. Per policy §1 (supports Copilot CLI → use it), **Copilot CLI
with the field's model** is the call. Two supporting facts:

- `.github/copilot-instructions.md` carries `applyTo: '**'` and is a thin
  router into `AGENTS.md`, `docs/WORKFLOW.md`, `Skills.md` — exactly the
  ambient-instruction shape `04-kprojects` already runs under.
- `install.sh` **does not ship `CLAUDE.md`** — it extracts `AGENTS.md` and
  `Skills.md` but not `CLAUDE.md`, which exists only in the upstream repo.
  So a repo-local install gives Claude Code `.claude/commands/` but a
  weaker ambient path than Copilot gets. Copilot is the better-supported
  target here.

**Nested-runner risk: checked, and it's clear.** `agentic-runner.ps1`
talks to `https://api.githubcopilot.com/chat/completions` directly and can
spawn the `claude` CLI with `--permission-mode bypassPermissions` — which
would have wrecked token accounting and defeated `--model`. But it is
**not reachable from `agentx.ps1` / `agentx-cli.ps1`**; nothing outside
the VS Code extension runtime, tests, and docs references it. As long as
we drive the CLI and not the extension, model control and
`collect-session.py` accounting hold.

Similarly, Model Council defaults to *agent-internal* mode (the calling
agent plays all three roles in-context). Only the opt-in `-AutoInvoke`
flag shells out to `gh models`, and the `gh-models` extension isn't
installed here — so no out-of-band model calls by default.

## Landmines for the eval protocol

**1. The `.gitignore` block hides most of the harness from git.** The
installer appends a marked block ignoring `.agentx/`, `.claude/`,
`scripts/`, `packs/`, `.cursor/…`, and most of `.github/`
(`agents/ instructions/ prompts/ skills/ templates/ hooks/ scripts/
schemas/ ISSUE_TEMPLATE/`, `copilot-instructions.md`, `CODEOWNERS`, …).

Consequences:

- Our `git add -A && git commit -m "eval: install agentx"` → **near-empty
  install commit**, so the `pre-run` tag wouldn't record the harness. Needs
  `git add -A -f` on the harness paths, or stripping the block.
- Survives into the working tree: `scripts/` and `packs/` are **generic
  names**. If the agent creates `scripts/` for the task, it is silently
  untracked and vanishes from the graded `pre-run..HEAD` diff. Low risk for
  loglens (single-module Python CLI), real risk on a bigger scenario.

Not ignored, so these *would* land in the diff: `AGENTS.md`, `Skills.md`,
`memories/`, five `docs/*.md`, `.github/AGENT-PROTOCOL.md`,
`.github/registries/`, `.github/security/`, `.github/copilot-mcp-config.json`,
and **12 `.github/workflows/*.yml`** (inert without a remote).

**2. The commit gates are the strictest in the field.** `pre-commit`
blocks any code commit unless `.agentx/state/loop-state.json` shows a
completed, non-consumed loop with ≥5 iterations, one of whose summaries
matches `review`. A completed loop is marked consumed after one commit, so
**each subsequent commit needs a fresh 5-iteration loop**. Other gates:
Model Council `COUNCIL-*.md` required alongside any new ADR (explicitly
**no skip token**), execution plan required at ≥8 changed code files
(`[skip-plan]` escape), compound-capture learning file alongside approved
reviews (`[skip-capture]` escape). `commit-msg` enforces the conventional
format; issue references are optional in local mode.

This is scoreable behavior, not a defect — but expect either heavy process
output or an agent that stalls on its own gates, and it makes "did the
agent commit?" a live question for the run log.

**3. `.claude/settings.json` ships malformed.** It sets
`permissions.allow` to VS Code tool names (`read_file`, `write_file`,
`run_terminal_command`) that aren't Claude Code's, sets `permissions.deny`
to a **string** rather than an array, and nests `model` under a bogus
`settings` key. Harmless under Copilot; if we ever run the Claude side,
this project-level file could conflict with the profile's
`bypassPermissions` or be rejected outright. Worth a preflight check.

## Go command

No CLI slash-command surface exists for Copilot: `/project:*` are
`.claude/commands/` (Claude Code only) and `@agentx` / "AgentX Auto" are
the VS Code extension only. So the prefix is **ambient-style**, like
`04-kprojects`. Proposed line for `prefixes.txt`:

```
NN-agentx|Build the loglens CLI described below, start to finish, following this repository's AgentX workflow.
```

Restating "AgentX Auto"-style orchestration in the prefix is an option if
we want the hub-routing behavior explicitly invoked; the ambient router
should reach it on its own, and staying close to the `04` phrasing keeps
the field comparable.

**Threat to declare:** AgentX's headline UX is the VS Code extension
(chat participant, sidebars, per-workspace LLM adapters). A CLI run tests
the ambient-instructions + PowerShell-CLI slice, not the product's
intended surface. Run 1 made the same scope call for harnesses with richer
UIs; it belongs in the whitepaper threats section either way.

## Prep checklist (when sprint 005 lands)

1. Build the profile under its final name; unpack pwsh 7.6.4 into it and
   confirm `run-eval.sh` puts it on PATH.
2. `new-run.sh <run-group> NN-agentx`; install with
   `AGENTX_PATH` unset, cwd = staging repo, `--no-setup` not needed
   (piped execution already skips the GitHub prompts).
3. Force-add the harness paths, commit `eval: install agentx
   (curl … | bash)`, tag `pre-run`.
4. Sanity-check with a cheap `copilot -p` — does it see the AgentX
   instructions, and does `./.agentx/agentx.sh loop status` run?
5. Append the prefix line, regenerate the prompt file, run.

## Decisions needed from Ken

1. **Which field does AgentX join?** run_01's loglens/Copilot field is the
   natural home (the `04-kprojects` ambient precedent lives there). Not
   run_03 unless we also want it in the Haiku tier.
2. **`.gitignore` block: force-add or strip?** Force-adding at `pre-run`
   preserves the harness's real behavior and keeps the record complete;
   stripping the block removes the `scripts/`-swallowing confound but
   edits the harness. Recommend force-add, and note the confound.
3. **Confirm the pwsh-in-profile approach** over a system install — it
   avoids a machine change but adds a 191 MB profile dependency and is a
   deviation the run log should record as an environment note.
