# Consensus + deliverables prompt — Fable(2)

You are the reconciliation-and-synthesis session for a comparative
AI-harness evaluation ("harness eval POC run 1"). Two independent graders —
Fable(1) (grader id `fable1`) and GPT Sol (grader id `sol`) — have each run
the objective acceptance checks and rubric grading on five repos. You:
drive them to consensus, produce final grades, then create the deliverables.

## Environment

Everything lives on the remote Linux host `kai` at
`/home/ken/src/ai-agents/harness-eval` (`$EVAL` below). You are running in
a desktop UI, so all work happens over ssh. kai's default shell is fish —
always wrap remote commands: `ssh kai bash -lc '...'`.

Context: `_eval/README.md` (design), `_eval/rubric.md` (dimensions +
weights), `_eval/acceptance.md` (the 12 checks),
`_eval/prompts/00-project-spec.md` (the task spec), `_eval/runs/*-runlog.md`
(timing/token data), `_eval/notes/lessons.md` (running lessons file, seeded
during setup — read it; you will extend it).

## Phase 0 — preconditions

Verify all of these exist under `$EVAL/_eval/grades/`; if any are missing,
report what's missing and stop:
`NN-acceptance-fable1.md`, `NN-fable1.md`, `NN-acceptance-sol.md`,
`NN-sol.md` for NN in 01..05, plus `summary-fable1.md` and `summary-sol.md`.

## Phase 1 — factual reconciliation (acceptance checks)

Diff the two graders' acceptance results per repo. Any check where they
disagree on pass/fail is a **factual dispute — you adjudicate it yourself**:
re-run that specific check on kai in a fresh clone
(`git clone $EVAL/NN-xxx /tmp/grade-consensus/NN-xxx`), using either
grader's sealed fixture (`grades/sealed-fixture-*`) or your own. Never
modify the run repos. Record every adjudication (check, both claims, what
you ran, verdict) in `grades/acceptance-adjudication.md`. Recompute each
repo's correctness score (0–5) from the adjudicated pass count; this
score is final and is NOT sent back to the graders.

## Phase 2 — rubric consensus (iterate with the graders)

For each repo × dimension (excluding correctness, which Phase 1 settled):

- |fable1 − sol| ≤ 1 → consensus = mean (halves allowed). No iteration.
- |fable1 − sol| ≥ 2 → write reconciliation prompts.

Reconciliation prompts go in `grades/reconcile/round<R>/NN-<dim>-fable1.md`
and `...-sol.md`. Each must be self-contained for a session that has its own
prior context: name the repo + dimension, quote BOTH scores and BOTH
justifications verbatim, pose the specific crux question, and instruct the
grader to append `## Reconciliation round <R>` with a revised-or-defended
score + rationale to their own `grades/NN-<grader>.md`. Ken pastes these
into the two grader sessions manually — after writing a round's prompts,
tell Ken they're ready and stop; you will be re-prompted when responses are
in. Re-check gaps each round; iterate until no ≥2 gaps remain (expect 1–2
rounds; if a gap survives two rounds, decide it yourself and log why in
`grades/final.md`).

## Phase 3 — final grades

`grades/final.md`: per repo, the consensus score for all 7 dimensions,
weighted total per `rubric.md`, and final ranking. Include the raw
per-grader scores and a grader-agreement note (where they diverged and why —
this feeds lessons learned).

## Phase 4 — deliverables

1. **White paper** — `_eval/report/whitepaper.md`. Readable prose, not a
   data dump: method (including the isolation-profile story and runner
   choice), results table, per-harness narrative (what its machinery
   actually did during the run — check `git log pre-run..HEAD` and harness
   artifacts like sprint docs / kb handoffs / phoenix traces), harness
   overhead analysis (tokens/credits/wall-clock vs the 05-baseline), threats
   to validity (n=1 per cell, grader bias, harness artifacts self-identify,
   spec written by the same person who built one contender).
2. **Infographic** — a single self-contained HTML page: headline weighted
   scores, cost/efficiency comparison, one-line verdict per harness. Save to
   `_eval/report/infographic.html` on kai AND publish it as an Artifact for
   easy viewing.
3. **Lessons learned** — `_eval/report/lessons-learned.md`: fold in
   `_eval/notes/lessons.md` (setup-time findings + run-log observations,
   e.g. run 03's harness-tension note, run 04 writing a korg WI), grading
   friction you experienced, grader-agreement analysis, and a concrete v2
   design: task-type matrix (greenfield / bug-fix / refactor / resume-from-
   handoff), N≥3 reps per cell, headless scripted runs, automated metric
   capture, executable sealed acceptance tests. Also update
   `_eval/notes/lessons.md` with anything new.

## Phase 5 — commit

From `$EVAL`: commit grades + report with message
`"grades: consensus, final scores, run-1 deliverables"`. Summarize final
ranking and key findings in chat.
