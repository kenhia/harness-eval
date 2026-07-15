# Grader prompt — Fable(1): objective + rubric grading

You are one of two independent graders for a comparative AI-harness
evaluation ("harness eval POC run 1"). Five agents built the same project —
the `loglens` CLI — in five repos, each under a different harness. Your job:
run the objective acceptance checks AND the rubric grading for all five
repos, then stop. Another grader (GPT Sol) is doing the same independently;
a third session will reconcile afterwards.

**Your grader id is `fable1`.** Use it in every filename you write.

## Environment

Everything lives on the remote Linux host `kai` at
`/home/ken/src/ai-agents/harness-eval` (`$EVAL` below). You are running in
a desktop UI, so all work happens over ssh. kai's default shell is fish —
always wrap remote commands: `ssh kai bash -lc '...'`.

## Read these first (in order)

1. `$EVAL/_eval/README.md` — design and contenders table
2. `$EVAL/_eval/prompts/00-project-spec.md` — the spec all five agents got
3. `$EVAL/_eval/rubric.md` — dimensions, weights, score sheet, bias notes
4. `$EVAL/_eval/acceptance.md` — the 12 objective checks
5. `$EVAL/_eval/runs/NN-runlog.md` per repo — timing, tokens, observations

## Hard rules

- **Independence**: do not read anything under `$EVAL/_eval/grades/`
  containing `sol` in the filename, and nothing under `grades/reconcile/`.
- **Never modify the five run repos.** Grade against clones:
  `ssh kai bash -lc 'git clone $EVAL/NN-xxx /tmp/grade-fable1/NN-xxx'`
  and run everything (uv sync, pytest, ruff, the CLI) inside the clone on kai.
- Agent-authored work = `git diff pre-run..HEAD` (plus `git log pre-run..HEAD`).
  Files present at the `pre-run` tag are harness-installed — out of scope for
  code-quality judgment.
- **Grading order: 01 → 05** (Sol grades 05 → 01; this spreads anchoring).

## Procedure

### Step 0 — sealed fixture (once)

Per `acceptance.md`, build your own sealed test fixture in
`$EVAL/_eval/grades/sealed-fixture-fable1/`: a deterministic generator
script (fixed seed), the generated CLF log (~40 lines: multiple IPs/paths,
2xx/3xx/4xx/5xx spread, several hours, an exact tie in path counts for the
A4 tie-break check, timestamps that straddle a `--since`/`--until` boundary,
2+ malformed lines), and `expected.md` with ground-truth values **computed
independently** (your own script or by hand — never using any project under
test).

### Step 1 — objective pass (per repo)

Execute all 12 checks from `acceptance.md` in the clone. Write
`$EVAL/_eval/grades/NN-acceptance-fable1.md`: per check, pass/fail + one
line of evidence (command + observed vs expected). Map passes to the 0–5
correctness score per the table in `acceptance.md`.

### Step 2 — rubric pass (per repo)

Score all 7 dimensions per `rubric.md` using its score-sheet template →
`$EVAL/_eval/grades/NN-fable1.md`. Notes:

- Dimension 1 (correctness) comes straight from your Step 1 result.
- Efficiency: use the run log's wall clock / tokens / AI credits. Honor any
  "give back" adjustments noted in the log observations (e.g. run 03 lost
  ~15s to eval-setup friction that was our fault, not the harness's).
- Autonomy: every run log reports zero interventions — differentiate on
  whether the agent *finished* (declared done, committed) without help.
- Judge the artifact, not the brand; ceremony that didn't change the outcome
  scores LOW on Process.

### Step 3 — summary

Write `$EVAL/_eval/grades/summary-fable1.md`: one table — rows = repos,
columns = the 7 dimension scores + acceptance passes (n/12) + weighted
total. No ranking commentary; the consensus session does synthesis.

### Step 4 — commit

From `$EVAL` (the root is a git repo):
`git add _eval/grades && git commit -m "grades(fable1): objective + rubric pass, all five repos"`

Then report your summary table in chat and stop. Do not write deliverables,
do not reconcile, do not peek at Sol's work.
