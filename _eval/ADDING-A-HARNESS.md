# Adding a harness to the field (incremental — no regrading of prior contenders)

The design goal: a new contender costs **one run + one delta-grading pass +
one short consensus session**. Prior repos, grades, and consensus are never
reopened (their scores are frozen artifacts of their run).

## 0. Research the harness first

Establish before touching anything: install method and **scope** (repo-local
vs global), **native runner** (Copilot CLI, Claude Code, Codex…), and the
autonomous "go" command. Read the installer source, not just the README —
run 1 found undocumented repo-local modes (working-skill-repo) and
undocumented global writes (Phoenix, gstack's `~/.gstack`).

## 1. Runner policy (the covariate rule)

- If the harness supports **Copilot CLI** (run 1's runner): use it, same
  model as prior runs.
- If it's native to a **different runner** (e.g. gstack → Claude Code): use
  the native runner with the **same model**, and flag `runner` as a
  covariate in the run log, whitepaper threats, and README. Efficiency
  metrics are not directly comparable across runners (Copilot credits vs
  Claude Code tokens) — graders weight wall-clock and behavior instead.
- When a new runner enters the field, add a control run of **baseline (no
  harness) under that runner** to re-anchor efficiency — realized for
  Claude Code as `07-baseline-claude` (05-vs-07 isolates the runner effect;
  06-vs-07 isolates the harness).

## 2. Isolation

- Repo-local harness bits → the staging repo (step 3).
- Global bits → a **profile**, never the live config. All profiles are
  **fake HOMEs** (`env HOME=<profile> <runner> …` — run-eval.sh does this
  for you); both runners resolve `~` through `$HOME`:
  - Copilot CLI: profile contains `.copilot/` (config, auth, session
    state). The run-1 symlink mechanism (`use-profile.sh`) is deprecated.
  - Claude Code: `claude-clean` is the harness-free base (credentials +
    klams/korg MCP only, no skills); a harness with global pieces gets its
    own copy, `claude-<harness>` (e.g. `claude-gstack`), with the global
    piece in the profile's `.claude/skills/`. Profile layout: `.claude/`
    with `.credentials.json`; **`.claude.json` at the profile ROOT** (with
    a HOME override Claude Code reads `$HOME/.claude.json`, NOT
    `.claude/.claude.json` — a minimal one: onboarding flags + MCP servers);
    `.gitconfig` (agents commit). The HOME override also captures stray
    global state (`~/.gstack`, `~/.cache`, …).
- **Never run a harness-free control with a harness-bearing profile** —
  that's the Phoenix-contamination mistake with new paint. One profile per
  environment flavor.
- Profiles need their own `settings.json` (`.claude/settings.json` in the
  profile): at minimum `permissions.defaultMode: "bypassPermissions"` +
  `skipDangerousModePermissionPrompt: true` for hands-off runs — a
  HOME-sandbox does NOT inherit the real account's permission settings.
  Exclude personal hooks/statusline (they'd publish eval sessions to
  personal dashboards).
- **Build a profile under its final name.** Harness installers bake
  absolute profile paths into hooks, compiled binaries, and caches
  (gstack's session hook + `browse` binary). Renaming a profile afterwards
  strands those; if it happens, grep the profile for the old path and
  rebuild the harness's compiled artifacts under the corrected HOME.
- After installing, **verify no real-HOME leakage** (check `~/.claude`,
  `~/.copilot`, `~/.<harness>` timestamps/paths) and sanity-check the
  profile with a cheap `claude -p` asking what skills/MCP servers it sees.
- Profiles hold live tokens — `_eval/profiles/` stays gitignored, always.

## 3. Staging repo

Run repos on `main` are flattened trees (publish format), so new runs
happen in a **staging repo** outside the eval repo, grouped by run:

```bash
_eval/bin/new-run.sh <run-group> NN-<name>          # harness run
_eval/bin/new-run.sh <run-group> NN-<name> --no-harness  # control run
```

This creates `~/src/ai-agents/harness-eval-runs/<run-group>/NN-<name>`
with the empty `eval: clean repo baseline` commit. For harness runs:
install the repo-local pieces, `git add -A && git commit -m "eval: install
<harness> (<exact command>)"`, then `git tag pre-run`.

## 4. Prompt

Append one line to `_eval/<run-group>/prompts/prefixes.txt`
(`NN-<name>|<go-command line>`), then regenerate:

```bash
cd _eval/<run-group>/prompts
{ grep '^NN-<name>' prefixes.txt | cut -d'|' -f2; echo; cat 00-project-spec.md; } > NN-<name>.md
```

Only the first line may differ between contenders. The spec body is frozen
for the life of the run group's field — changing it invalidates
comparability.

## 5. Run

```bash
_eval/bin/run-eval.sh --runner claude|copilot --profile <name> \
  --run-group <run-group> --repo NN-<name> [--model <id>] \
  [--prompt-file _eval/<run-group>/prompts/NN-<name>.md] [--headless]
```

Preflight, timing, runner version, session metrics, diffstat, and the
real-HOME leak canary are captured automatically into
`_eval/<run-group>/runs/NN-runlog.md`; you fill only the Manual section
(declared-done, interventions verbatim, observations, `/cost` paste for
interactive Claude runs). Zero-intervention discipline as in run 1.

## 6. Delta grading

Two **new** grader sessions (same grader identities: `fable1`, `sol`), each
given the matching `_eval/<run-group>/grader_prompts/<grader>-delta-*.md`.
Key differences from full-field grading:

- Grade **only** the new run(s), in opposite order per grader.
- **Calibrate first**: re-read your own prior sheets and
  `summary-<grader>.md` — the scale must be consistent with your
  predecessor session's. (Your own prior work is readable; the other
  grader's remains off-limits, as do `final.md` and `reconcile/`.)
- **Precedents are readable**: `grades/precedents.md` records adjudicated
  *interpretations* (not scores) from prior consensus sessions — e.g. run
  1's half-open-window ruling. Delta graders read it before grading so
  settled questions aren't re-litigated (run 1.5 wasted a full dispute
  cycle re-arguing an already-adjudicated boundary question). Consensus
  sessions append to it.
- Reuse your existing sealed fixture (`grades/sealed-fixture-<grader>/`).
- Output: `NN-acceptance-<grader>.md`, `NN-<grader>.md`, appended summary
  rows, one commit.

Then a **delta consensus** session (`fable2-delta-*.md`): adjudicate
factual disputes on the new runs only, reconcile ≥2 gaps on the new runs
only, then update `grades/final.md`, the whitepaper (results, narratives,
threats), the infographic, the root README results table, and lessons.

## 7. Publish import

After consensus, fold each run into the published repo (final trees live
under `run-output/<run-group>/`):

```bash
cd ~/src/ai-agents/harness-eval
mkdir -p run-output/<run-group>/NN-<name>
git -C ~/src/ai-agents/harness-eval-runs/<run-group>/NN-<name> archive main \
  | tar -x -C run-output/<run-group>/NN-<name>
git add run-output/<run-group>/NN-<name> \
  && git commit -m "runs: import final tree of <run-group>/NN-<name>"
git fetch --no-tags ~/src/ai-agents/harness-eval-runs/<run-group>/NN-<name> \
  main:refs/heads/history/<run-group>/NN-<name> \
  refs/tags/pre-run:refs/tags/pre-run/<run-group>/NN-<name>
git push origin main "refs/heads/history/<run-group>/NN-<name>" \
  "refs/tags/pre-run/<run-group>/NN-<name>"
```

Run the secret scan first (tracked files + full staging history + the run
transcript), same patterns as run 1.

## Comparability caveats to restate in the whitepaper each time

- New runs use later tool/model builds than run 1 (record exact versions in
  the run log; Copilot CLI and Claude Code both auto-update).
- Grader sessions are new instances calibrating from written sheets, not
  the original graders' full context.
- Runner covariate where applicable (§1) — compare new-runner entries to
  the new-runner control, not raw cost numbers across runners.
