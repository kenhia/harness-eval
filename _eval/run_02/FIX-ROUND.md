# run_02.1 — organic bug-fix round (STATUS: setup)

Six of seven run_02 implementations failed C9/H12 identically (quick-xml
streaming leniency: truncated/malformed XML silently becomes a
successful empty fetch). Each of those six agents now receives a **bug
report** against its own repo and fixes its own codebase. 03 (the only
full pass, roxmltree) sits out — that absence is itself a result and is
called out in the report/infographic alongside the ungraded 99 control's
identical pass.

## Design rules

- **Bug report, not tests.** The sealed suite never enters prompts or
  repos. The report's reproduction sample is a *different* truncation
  than the sealed fixture, so special-casing the sample cannot pass.
- **Same repos, continued.** Boundary tag `pre-fix` at each repo's
  post-run-02 HEAD; agent-authored fix = `pre-fix..HEAD`.
- **Same go-command style** per harness (fix-flavored first line,
  shared body — `prompts/fix-prefixes.txt` + `prompts/NN-fix.md`).
- **Headless, hands-off, serial**, via `run-eval.sh --tag pre-fix
  --suffix fix` → `runs/NN-fix-runlog.md`.
- **Verification** = the full frozen suite (must flip C9/H12 with zero
  regressions) **plus a sealed fix addendum** (`acceptance/test_fix.py`,
  gated behind `FIX_ROUND=1`, frozen before the first fix run):
  - F1: a *different* truncated feed also errors (no sample
    special-casing);
  - F2: a **well-formed feed with zero items still refreshes "ok"** —
    kills the cheap "0 items = error" heuristic;
  - F3: `last_error` clears after a subsequent successful fetch.
  Run: `run-acceptance.sh run_02 NN-<name> --fix` →
  `runs/NN-fix-acceptance.txt` (27 + addendum results).
- The frozen 26-test suite is untouched; the addendum is additive and
  cannot run in default (non-fix) invocations.

## Grading (delta, after all six fix runs)

Two fresh grader sessions + short consensus (prompts to be written after
runs complete, modeled on the run 1.5 delta process). Dimensions for the
fix delta:

| dim | weight | what 5 looks like |
|---|---|---|
| Fix correctness | 35 | Full suite 26/26 + addendum clean |
| Fix quality | 30 | Root cause (strict parse-error detection at the XML layer), not symptom patch; minimal blast radius |
| Tests | 20 | Agent added a genuine regression test unprompted (the report does not ask for one) |
| Scope & process | 10 | Touched what the fix needed, nothing else; coherent commits |
| Efficiency | 5 | Wall clock / tokens vs the field's fix-round median |

## What this round measures

Resume-own-work + debugging differentiation: run 1 and run 02 were both
greenfield; this is the first cell where harness handoff machinery
(sprints, KB context, gstack state) can matter — or be exposed — against
bare controls resuming from nothing but the code.

## Status log

- 2026-07-16: protocol written; prompts, addendum, tags, tooling in
  place (this commit). Awaiting fix runs.
- 2026-07-16 (late): **E1 — MCP servers blocked by policy in fix-round
  copilot runs** (`! 3 MCP servers were blocked by policy: 'klams',
  'korg', 'phoenix'`). Root cause (established by probes 2026-07-17):
  **token-class policy** — GitHub began policy-blocking ALL MCP servers
  under gh-CLI OAuth tokens (`gho_`) between Jul 16 ~17:06 and ~22:51
  UTC; run-eval.sh had been injecting exactly that token class since the
  morning auth fix. The native Copilot login is unaffected (interactive
  `/mcp list` works). Related trap found during diagnosis: VS Code
  integrated terminals export a short-lived `COPILOT_GITHUB_TOKEN` PAT
  that goes stale and hijacks fake-HOME copilot auth — this, not
  credential rotation, likely caused the original 05 launch failure.
  Fixes: run-eval.sh now scrubs ambient token vars from every launch,
  uses the profile's own Copilot login by default (one-time
  `env HOME=<profile> copilot` + `/login` per profile), and demotes
  gh-token injection to an explicit `--inject-gh-token` flag with a
  loud MCP warning. Run 02's *graded* phoenix session (17:06) made
  successful `phoenix_sense` MCP calls — run 02 results unaffected.
  Impact and handling per cell:
  - **02 fix run: ran without the phoenix harness's MCP spine —
    recommended VOID + rerun after policy is restored** (the round
    exists to measure harness machinery; phoenix's was off). Run 02
    main phoenix cell: verified unaffected.
  - **01 fix run: stands** — klams/korg ambient MCP absent, but
    StarterKit's machinery is fully repo-local (no MCP dependency);
    noted as covariate.
  - **04 (kprojects): hold until restored** — its conventions
    reference korg/klams ambiently.
  - 05/06/07: no harness MCP dependency (Claude cells' profile MCP is
    not subject to GitHub policy); may run, with the runner-side
    ambient-MCP asymmetry noted (lesson 7 again: ambient-service
    availability is part of the environment definition and can change
    UNDER you mid-field — v3 preflight should capture an MCP
    availability manifest per run mechanically).
