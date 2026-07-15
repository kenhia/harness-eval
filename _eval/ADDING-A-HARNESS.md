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
- Optional but recommended when a new runner enters the field: a control
  run of **baseline (no harness) under the new runner** to re-anchor
  efficiency. Number it like a repo (e.g. `07-baseline-claude`).

## 2. Isolation

- Repo-local harness bits → the staging repo (step 3).
- Global bits → a **profile**, never the live config:
  - Copilot CLI: `_eval/profiles/<name>/.copilot`, swapped with
    `_eval/bin/use-profile.sh` (symlink on `~/.copilot`).
  - Claude Code: `_eval/profiles/claude-clean/` used as a **fake HOME**
    (`env HOME=<profile> PATH=$PATH claude …`). Contains: `.claude/` with
    `.credentials.json`, minimal `.claude.json` (onboarding flags +
    klams/korg MCP only), `skills/<harness>/` for the harness's global
    piece; plus `.gitconfig` (agents commit). The HOME override also
    captures stray global state (`~/.gstack` etc.).
- After installing, **verify no real-HOME leakage** (check `~/.claude`,
  `~/.copilot`, `~/.<harness>` timestamps/paths).
- Profiles hold live tokens — `_eval/profiles/` stays gitignored, always.

## 3. Staging repo

Run repos on `main` are flattened trees (publish format), so new runs
happen in a **staging repo** outside the eval repo:

```bash
cd ~/src/ai-agents/harness-eval-runs
mkdir NN-<name> && cd NN-<name>
git init -b main
git commit --allow-empty -m "eval: clean repo baseline"
# install the harness (repo-local pieces)
git add -A && git commit -m "eval: install <harness> (<exact command>)"
git tag pre-run
```

## 4. Prompt

Append one line to `_eval/prompts/prefixes.txt`
(`NN-<name>|<go-command line>`), then regenerate:

```bash
cd _eval/prompts
{ grep '^NN-<name>' prefixes.txt | cut -d'|' -f2; echo; cat 00-project-spec.md; } > NN-<name>.md
```

Only the first line may differ between contenders. The spec body is frozen
for the life of run 1's field — changing it invalidates comparability.

## 5. Run

Fresh session, correct profile active, hands off, run log from
`_eval/runs/runlog-template.md` (note the runner and how token/cost data is
captured on that runner). Zero-intervention discipline as in run 1.

## 6. Delta grading

Two **new** grader sessions (same grader identities: `fable1`, `sol`), each
given `_eval/grader_prompts/<grader>-delta-NN.md`. Key differences from run
1 grading:

- Grade **only** the new repo.
- **Calibrate first**: re-read your own five prior sheets and `summary-
  <grader>.md` — the scale must be consistent with your predecessor
  session's. (You may read your own prior work; the other grader's remains
  off-limits.)
- Reuse your existing sealed fixture (`grades/sealed-fixture-<grader>/`).
- Output: `NN-acceptance-<grader>.md`, `NN-<grader>.md`, append a row to
  `summary-<grader>.md`, commit.

Then a **delta consensus** session (`fable2-delta-NN.md`): adjudicate
factual disputes on the new repo only, reconcile ≥2 gaps on the new repo
only, then update `grades/final.md`, the whitepaper results + narrative +
threats, the infographic, the root README results table, and lessons.

## 7. Publish import

After consensus, fold the run into the published repo:

```bash
cd ~/src/ai-agents/harness-eval
mkdir NN-<name>
git -C ~/src/ai-agents/harness-eval-runs/NN-<name> archive main | tar -x -C NN-<name>
git add NN-<name> && git commit -m "runs: import final tree of NN-<name>"
git fetch --no-tags ~/src/ai-agents/harness-eval-runs/NN-<name> \
  main:refs/heads/history/NN-<name> refs/tags/pre-run:refs/tags/pre-run/NN-<name>
git push origin main "refs/heads/history/NN-<name>" "refs/tags/pre-run/NN-<name>"
```

Run the secret scan first (tracked files + full staging history + the run
transcript), same patterns as run 1.

## Comparability caveats to restate in the whitepaper each time

- New runs use later tool/model builds than run 1 (record exact versions in
  the run log; Copilot CLI and Claude Code both auto-update).
- Grader sessions are new instances calibrating from written sheets, not
  the original graders' full context.
- Runner covariate where applicable (§1).
