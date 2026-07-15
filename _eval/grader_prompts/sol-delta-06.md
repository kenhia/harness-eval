# Grader prompt — GPT Sol delta: grade 06-gstack only

You are grader `sol` for the harness comparative evaluation. A previous
session under your grader id already graded five repos (01–05); those
grades are frozen. Your job: grade ONE new contender — `06-gstack` — on the
same scale, then stop. Grader `fable1` is doing the same independently; a
delta-consensus session integrates afterwards.

## Environment

You are on `kai` in `/home/ken/src/ai-agents/harness-eval` (`$EVAL`),
direct shell access. The new run lives in a STAGING repo (not inside
`$EVAL`): `/home/ken/src/ai-agents/harness-eval-runs/06-gstack` — branch
`main`, tag `pre-run` marks the boundary; agent work = `pre-run..HEAD`.

## Calibrate first (replaces run 1's blindness rule)

Read, in order: `_eval/README.md`, `_eval/prompts/00-project-spec.md`,
`_eval/rubric.md`, `_eval/acceptance.md`, then **your own** five prior
sheets (`_eval/grades/0N-sol.md`, `0N-acceptance-sol.md`, `summary-sol.md`)
to anchor your scale — score 06 as if the same grader were still holding
the pen. Fable's files (`*-fable1*`) remain OFF-LIMITS, as does
`grades/reconcile/` and `grades/final.md` (it contains consensus opinions
that would anchor you).

Then read `_eval/runs/06-runlog.md` and `_eval/ADDING-A-HARNESS.md` §1
(runner covariate).

## The runner covariate (important for two dimensions)

Run 06 used **Claude Code CLI** (gstack is Claude-native), not Copilot CLI
like runs 01–05 — same model family. Consequences:

- **Efficiency**: token/cost numbers are not directly comparable to
  Copilot "AI credits". Score efficiency primarily on wall-clock, thrash,
  and rework visible in the transcript/commits; state the caveat in your
  justification rather than penalizing the runner.
- **Everything else**: grade the artifact exactly as before.

## Procedure

1. Clone: `git clone /home/ken/src/ai-agents/harness-eval-runs/06-gstack /tmp/grade-sol/06-gstack`
   (never modify the staging repo). Files at `pre-run` are harness-installed
   — out of scope for code-quality judgment.
2. Objective pass: reuse YOUR sealed fixture
   (`_eval/grades/sealed-fixture-sol/`) and execute the 12 checks from
   `acceptance.md` → `_eval/grades/06-acceptance-sol.md` (per-check
   pass/fail + evidence, mapped to the 0–5 correctness score).
3. Rubric pass: score all 7 dimensions per `rubric.md` →
   `_eval/grades/06-sol.md` (score-sheet template; judge the artifact, not
   the brand; ceremony that didn't change the outcome scores LOW on
   Process).
4. Append the 06 row to `_eval/grades/summary-sol.md`.
5. Commit from `$EVAL`:
   `git add _eval/grades && git commit -m "grades(sol): delta pass — 06-gstack"`
   Do NOT push.

Report your 06 scores in chat and stop. No consensus work, no deliverables,
no peeking at Fable.
