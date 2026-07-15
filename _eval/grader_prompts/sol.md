# Grader prompt — GPT Sol: objective + rubric grading

You are one of two independent graders for a comparative AI-harness
evaluation ("harness eval POC run 1"). Five agents built the same project —
the `loglens` CLI — in five repos, each under a different harness. Your job:
run the objective acceptance checks AND the rubric grading for all five
repos, then stop. Another grader (Fable) is doing the same independently; a
third session will reconcile afterwards.

**Your grader id is `sol`.** Use it in every filename you write.

## Environment

You are running on `kai` inside `/home/ken/src/ai-agents/harness-eval`
(`$EVAL` below). You have direct filesystem and shell access; `uv`,
`pytest`, `ruff`, `python3`, and `git` are available.

## Read these first (in order)

1. `_eval/README.md` — design and contenders table
2. `_eval/prompts/00-project-spec.md` — the spec all five agents got
3. `_eval/rubric.md` — dimensions, weights, score sheet, bias notes
4. `_eval/acceptance.md` — the 12 objective checks
5. `_eval/runs/NN-runlog.md` per repo — timing, tokens, observations

## Hard rules

- **Independence**: do not read anything under `_eval/grades/` containing
  `fable` in the filename, and nothing under `grades/reconcile/`.
- **Never modify the five run repos.** Grade against clones:
  `git clone $EVAL/NN-xxx /tmp/grade-sol/NN-xxx` and run everything
  (uv sync, pytest, ruff, the CLI) inside the clone.
- Agent-authored work = `git diff pre-run..HEAD` (plus `git log pre-run..HEAD`).
  Files present at the `pre-run` tag are harness-installed — out of scope for
  code-quality judgment.
- **Grading order: 05 → 01** (Fable grades 01 → 05; this spreads anchoring).

## Procedure

### Step 0 — sealed fixture (once)

Per `acceptance.md`, build your own sealed test fixture in
`_eval/grades/sealed-fixture-sol/`: a deterministic generator script (fixed
seed), the generated CLF log (~40 lines: multiple IPs/paths, 2xx/3xx/4xx/5xx
spread, several hours, an exact tie in path counts for the A4 tie-break
check, timestamps that straddle a `--since`/`--until` boundary, 2+ malformed
lines), and `expected.md` with ground-truth values **computed independently**
(your own script or by hand — never using any project under test).

### Step 1 — objective pass (per repo)

Execute all 12 checks from `acceptance.md` in the clone. Write
`_eval/grades/NN-acceptance-sol.md`: per check, pass/fail + one line of
evidence (command + observed vs expected). Map passes to the 0–5 correctness
score per the table in `acceptance.md`.

### Step 2 — rubric pass (per repo)

Score all 7 dimensions per `rubric.md` using its score-sheet template →
`_eval/grades/NN-sol.md`. Notes:

- Dimension 1 (correctness) comes straight from your Step 1 result.
- Efficiency: use the run log's wall clock / tokens / AI credits. Honor any
  "give back" adjustments noted in the log observations (e.g. run 03 lost
  ~15s to eval-setup friction that was our fault, not the harness's).
- Autonomy: every run log reports zero interventions — differentiate on
  whether the agent *finished* (declared done, committed) without help.
- Judge the artifact, not the brand; ceremony that didn't change the outcome
  scores LOW on Process.

### Step 3 — summary

Write `_eval/grades/summary-sol.md`: one table — rows = repos, columns =
the 7 dimension scores + acceptance passes (n/12) + weighted total. No
ranking commentary; the consensus session does synthesis.

### Step 4 — commit

From `$EVAL` (the root is a git repo):
`git add _eval/grades && git commit -m "grades(sol): objective + rubric pass, all five repos"`

Then report your summary table in chat and stop. Do not write deliverables,
do not reconcile, do not peek at Fable's work.
