# Acceptance — run_03 (loglens scenario, sub-frontier tiers)

**SEALED. Never shown to working agents.** Executable port of run_01's
prose checklist (A1–A12) to the run_02 pytest pattern, for the
model-capability axis (Haiku tier now, BYOK/gemma next). The scenario
spec is run_01's frozen loglens spec, unchanged — including its silences
(window bounds), which the suite honors via precedent P1.

## Running it

```bash
_eval/bin/run-acceptance.sh run_03 NN-<name>          # archives + tallies
# or directly:
ACCEPTANCE_REPO=<repo> uv run --with pytest pytest _eval/run_03/acceptance -v
```

## Tiers

**Core (test_core.py)** — A1–A12, mechanical: entry point, summary
text/JSON values (implementation-agnostic scans — layouts are free),
top with the tie-break, errors grouping + in-window counts (no boundary
records — that's H4's job), hourly, malformed-count-on-stderr, exit
codes 0/1/2, repo pytest, ruff, delivered fixture minimums, one-command
check. Format placement is adaptive (`--format` accepted before or
after the subcommand — both are legitimate readings of "global
option").

**Hard (test_hard.py)** — the robustness edges run 1's graders found by
hand, mechanized: mixed UTC offsets (H1), offset-aware windows (H2),
Z-suffix ISO 8601 (H3), boundary semantics per P1 — since-side
inclusive, until-side either convention (H4), hour-of-day aggregation
across days (H5), empty windows (H6), JSON validity everywhere (H7),
CLF dash-bytes dual-accept (H8), and **H9: naive `--since` against
offset-carrying logs** — run 1's documented crash class; dual-accept
(interpret it or reject it cleanly) but a traceback fails.

## Validation — retro-run against the seven graded run_01 trees

Calibration target: all seven were adjudicated 12/12, so core must pass
7/7 — it does. Suite bugs found and fixed during calibration:
VIRTUAL_ENV leakage from the outer uv into the contender's `uv run`,
malformed-line detection weaker than real parsers, and hardcoded
`--format` placement. Result:

| repo | core | hard | hard failure |
|---|---|---|---|
| 01-atv-starterkit | 12/12 | 9/9 | — |
| 02-atv-phoenix | 12/12 | 8/9 | H9 traceback |
| 03-working-skill-repo | 12/12 | 8/9 | H9 traceback |
| 04-kprojects | 12/12 | 9/9 | — |
| 05-baseline | 12/12 | 8/9 | H9 traceback |
| 06-gstack | 12/12 | 9/9 | — |
| 07-baseline-claude | 12/12 | 9/9 | — |

**Retro-finding:** the executable hard tier mechanically discriminates
run 1's dead heat, and H9's three failures are exactly the repos run
1's graders flagged for the naive-vs-aware datetime crash (lessons 9,
21). Both Claude-runner cells pass, consistent with run 1.5's grading
notes. The suite freezes for the run_03 field once the first tier
contender runs; these retro results are calibration, not new grades —
run_01 scores remain frozen.
