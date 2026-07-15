# Run log — 07-baseline-claude (control: no harness, Claude Code runner)

Purpose: anchor the Claude Code runner. 05-vs-07 isolates the runner effect
on a bare baseline; 06-vs-07 isolates gstack's contribution on that runner.

- date:
- runner: **Claude Code CLI on kai** (same as run 06; covariate vs runs 01–05)
- launch (from bash — fish mangles the env invocation): `cd ~/src/ai-agents/harness-eval-runs/07-baseline-claude && env HOME=~/src/ai-agents/harness-eval/_eval/profiles/claude-clean claude --model claude-opus-4-8`
- model: claude-opus-4-8 (confirm exact id in-session)
- claude-code version: (run `claude --version`; should match run 06's)
- profile used: claude-clean (HOME-sandbox: credentials + klams/korg MCP only, no harness, no personal skills)
- profile sanity-checked before start (no gstack, no personal skills): yes/no
- start time:
- end time:
- wall clock:
- agent declared done: yes / no / gave up
- final state committed by agent: yes / no (post-run snapshot commit added)

## Interventions (verbatim, timestamped — empty section = zero)

1.

## Token/cost data (`/cost` before exiting; NOT comparable to Copilot AI credits — compare to run 06 only)

## Observations (free-form: loops, stalls, notable behavior)
