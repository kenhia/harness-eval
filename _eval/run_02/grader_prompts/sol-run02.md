# Grader prompt — GPT Sol: rubric grading, run_02 field

You are one of two independent graders for a comparative AI-harness
evaluation ("harness eval run 02"). Seven agents built the same project —
the `feedhub` Rust workspace (three binaries + shared crate) — in seven
staging repos, five under different harnesses and two harness-free
controls. Your job: rubric-grade all seven repos, then stop. Another
grader (Fable) is doing the same independently; a third session
reconciles afterwards.

**Your grader id is `sol`.** Use it in every filename you write.

## Environment

Everything lives on the remote Linux host `kai`. `$EVAL` =
`/home/ken/src/ai-agents/harness-eval`; the seven repos are staging repos
at `~/src/ai-agents/harness-eval-runs/run_02/NN-<name>` (they are NOT yet
imported into $EVAL). kai's default shell is fish, and
`ssh kai bash -lc 'cmd args'` silently drops the args — run remote work
as heredoc scripts: `ssh kai bash -s <<'EOF' ... EOF`.

## Read these first (in order)

1. `$EVAL/_eval/run_02/README.md` — the field, decisions, defect log, report notes
2. `$EVAL/_eval/run_02/prompts/00-project-spec.md` — the spec (edge semantics are PINNED — no interpretation needed)
3. `$EVAL/_eval/run_02/rubric.md` — dimensions, weights, correctness mapping, bias notes
4. `$EVAL/_eval/run_02/grades/precedents.md` — adjudicated interpretations; do not re-litigate
5. Per repo: `$EVAL/_eval/run_02/runs/NN-runlog.md` (auto-captured metrics) and `NN-acceptance.txt` (executable acceptance results — **supplied, never re-derive**)
6. **Calibration**: your predecessor sessions' run-1 sheets — `$EVAL/_eval/run_01/grades/*-sol.md` and `summary-sol.md`. Your scale must be continuous with theirs. (Fable's sheets, `final.md`, and `reconcile/` remain off-limits, run_01 and run_02 both.)

## Hard rules

- **Independence**: read nothing with `fable` in the filename, nothing under `grades/reconcile/`.
- **Never modify the seven run repos.** Grade against clones:
  `git clone ~/src/ai-agents/harness-eval-runs/run_02/NN-x /tmp/grade-sol-run02/NN-x`
  and run everything (cargo build/test, the binaries) inside the clone on kai.
- Agent-authored work = `git diff pre-run..HEAD` (+ `git log pre-run..HEAD`). Files at the `pre-run` tag are harness-installed — out of scope for code-quality judgment; harness-*prescribed* artifacts the agent produced count under Process.
- **Grading order: 07 → 01** (Fable grades 01 → 07).
- There is no sealed-fixture step and no objective pass — the executable acceptance suite already ran (results in `runs/NN-acceptance.txt`, adjudicated defect history in the README's Suite defect log).

## Procedure

### Step 1 — rubric pass (per repo)

Score all 7 dimensions per `run_02/rubric.md` using run_01's score-sheet
template → `$EVAL/_eval/run_02/grades/NN-sol.md`. Notes:

- **Correctness** maps mechanically from `NN-acceptance.txt`: core 14/14 → 4, fifth point from the hard tier (≥9/12 ≈ 5); any core failure caps at ≤3. State the mapping you applied.
- **Efficiency** anchors to the same-runner control (05 for Copilot cells, 07 for Claude cells) per the rubric bands. Copilot credit numbers are NOT comparable to run 1's (unit drift — see README report notes); premium requests and wall clock are the stabler units.
- **Autonomy**: all runs were headless with zero interventions — differentiate on finished-and-committed (see runlogs; 01's `.atv` state needed a post-run snapshot commit, noted there).
- Judge the artifact, not the brand; ceremony that didn't change the outcome scores LOW on Process.

### Step 2 — summary

`$EVAL/_eval/run_02/grades/summary-sol.md`: one table — rows = repos,
columns = 7 dimension scores + acceptance (core n/14, hard n/12) +
weighted total. No ranking commentary; consensus does synthesis.

### Step 3 — commit

From `$EVAL`:
`git add _eval/run_02/grades && git commit -m "grades(sol): run_02 rubric pass, all seven repos"`

Report your summary table in chat and stop. Do not write deliverables, do
not reconcile, do not peek at Fable's work.
