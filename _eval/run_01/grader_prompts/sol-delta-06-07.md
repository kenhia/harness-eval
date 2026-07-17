# Grader prompt — GPT Sol delta: grade 07-baseline-claude and 06-gstack

You are grader `sol` for the harness comparative evaluation. A previous
session under your grader id already graded five repos (01–05); those
grades are frozen. Your job: grade TWO new runs on the same scale, then
stop. Grader `fable1` is doing the same independently; a delta-consensus
session integrates afterwards.

The two runs (both on the **Claude Code** runner — see covariate below):

- `07-baseline-claude` — control: NO harness, staging repo
  `/home/ken/src/ai-agents/harness-eval-runs/07-baseline-claude`
- `06-gstack` — the gstack harness (Claude-Code-native), staging repo
  `/home/ken/src/ai-agents/harness-eval-runs/06-gstack`

Each staging repo: branch `main`, tag `pre-run` marks the boundary; agent
work = `pre-run..HEAD` (for 07 the pre-run tag is the empty baseline
commit — everything is agent-authored).

**Grade 07 first, then 06** (Fable does the reverse order).

## Environment

You are on `kai` in `/home/ken/src/ai-agents/harness-eval` (`$EVAL`),
direct shell access; `uv`, `pytest`, `ruff`, `python3`, `git` available.

## Calibrate first (replaces run 1's blindness rule)

Read, in order: `_eval/README.md`, `_eval/prompts/00-project-spec.md`,
`_eval/rubric.md`, `_eval/acceptance.md`, then **your own** five prior
sheets (`_eval/grades/0N-sol.md`, `0N-acceptance-sol.md`, `summary-sol.md`)
to anchor your scale — score the new runs as if the same grader were still
holding the pen. Fable's files (`*-fable1*`) remain OFF-LIMITS, as do
`grades/reconcile/` and `grades/final.md` (consensus opinions that would
anchor you).

Then read `_eval/runs/06-runlog.md`, `_eval/runs/07-runlog.md`, and
`_eval/ADDING-A-HARNESS.md` §1 (runner covariate).

## The runner covariate

Runs 06 and 07 used **Claude Code CLI**, not Copilot CLI like 01–05 — same
model family. Consequences:

- **Efficiency**: Claude Code `/cost` numbers are not comparable to Copilot
  "AI credits". 07 exists precisely to anchor this: score 06's efficiency
  primarily relative to 07 (wall-clock, tokens, thrash/rework) plus
  absolute wall-clock; state the caveat in justifications rather than
  penalizing the runner. Score 07's efficiency on wall-clock and absence of
  thrash, as you did for 05.
- **Everything else**: grade the artifacts exactly as before.

## Procedure (for each of 07, then 06)

1. Clone to `/tmp/grade-sol/<NN-name>` — never modify staging repos. Files
   at `pre-run` are harness-installed, out of scope for code-quality.
2. Objective pass with YOUR sealed fixture
   (`_eval/grades/sealed-fixture-sol/`): all 12 checks →
   `_eval/grades/NN-acceptance-sol.md` (pass/fail + evidence, mapped to the
   0–5 correctness score).
3. Rubric pass: all 7 dimensions per `rubric.md` →
   `_eval/grades/NN-sol.md`. Judge the artifact, not the brand; ceremony
   that didn't change the outcome scores LOW on Process. For 07, Process is
   graded the same way it was for 05 (git hygiene, coherent commits — no
   harness artifacts expected).
4. Append the row to `_eval/grades/summary-sol.md`.

Then commit once from `$EVAL`:
`git add _eval/grades && git commit -m "grades(sol): delta pass — 06-gstack, 07-baseline-claude"`
Do NOT push.

Report both score rows in chat and stop. No consensus work, no
deliverables, no peeking at Fable.
