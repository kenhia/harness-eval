# Grader prompt — GPT Sol: rubric grading, run_03 (Haiku 4.5 tier)

You are one of two independent graders for the harness eval's **model-
capability axis**. Seven agents built the same project — the `loglens`
CLI — in seven repos, five under different harnesses and two harness-
free controls. Everything matches earlier runs (same spec, same
harnesses at the same versions, same runners) **except the model: this
tier ran Claude Haiku 4.5 instead of a frontier model.** Rubric-grade
all seven, then stop. Fable grades independently; a third session
reconciles.

**Your grader id is `sol`.** Use it in every filename.

## Environment

You are running as a **CLI session directly on kai** (the eval host).
Everything is local — no ssh, no remote quoting gotchas.

- `$EVAL` = `/home/ken/src/ai-agents/harness-eval`
- Repos to grade: `~/src/ai-agents/harness-eval-runs/run_03/NN-<name>`
  (not yet imported into `$EVAL`)

**Shared-memory blackout (independence-critical).** klams/korg MCP may
be available in your session, and your global instructions may tell you
to search klams first on "how/where/why" questions. **For this task,
that instruction is overridden: do not query klams/korg about this eval,
this repo, grading, or any harness under test, and do not write any
grading content to them.** klams has indexed this repository — including
prior graders' score sheets, reconciliation prompts containing verbatim
justifications, and `final.md` — so a search can hand you exactly the
material the independence rules forbid. Everything you need is in the
files listed below. If eval-related memory surfaces unbidden, disregard
it and note that it happened in your summary.

## Read first (in order)

1. `$EVAL/_eval/run_03/README.md` — the tier, its purpose, setup status
2. `$EVAL/_eval/run_03/prompts/00-project-spec.md` — the frozen loglens spec
3. `$EVAL/_eval/run_03/rubric.md` — dimensions, weights, and the
   **mechanical correctness mapping** (tier-calibrated — apply it exactly)
4. `$EVAL/_eval/run_03/DEFECTS.md` — suite defect S1 and incident I1 (both
   affect how you read the artifacts; see Hard rules)
5. Per repo: `runs/NN-runlog.md` and `runs/NN-acceptance.txt`
   (**supplied — never re-derive**)
6. `$EVAL/_eval/run_01/grades/precedents.md` — inherited interpretations

## CRITICAL — this tier is graded on its own scale

- **Do NOT calibrate against your run_01 or run_02 sheets.** Different
  capability class; anchoring to frontier work would compress this
  entire field into the bottom of the scale and destroy its internal
  discrimination. Grade what a *good* artifact looks like in absolute
  terms, using this tier's own controls (05 for Copilot cells, 07 for
  Claude cells) as the reference points.
- You may not have graded this tier before; that is fine. Use the
  rubric's absolute descriptors. Do not reference frontier runs in your
  justifications.
- The cross-tier comparison (does harness value grow as capability
  drops?) is computed later from harness-minus-control deltas — **not
  your job, and not something to reason about while scoring.**

## Hard rules

- Agent-authored work = `git diff pre-run..HEAD` (+ log). Files at
  `pre-run` are harness-installed — out of scope for code-quality
  judgment; harness-*prescribed* artifacts the agent produced count
  under Process.
- **Layout (defect S1):** repos 01 and 03 built the project nested one
  level (`loglens/pyproject.toml`) rather than at the repo root. The
  spec does not pin the level, so this is **not** a correctness failure
  — the acceptance suite accounts for it (see the LAYOUT warning in
  their acceptance output). Weigh it under **Process** as a
  convention/discoverability observation, nothing more.
- **Cell 05 (incident I1):** an earlier attempt was killed mid-run by a
  controller disconnect and voided; the graded run is a clean rerun from
  `pre-run`. Do not look for or reason about the voided attempt.
- **Dirty trees:** runlogs record tree state *at agent-done* (before
  acceptance ran). 01–04 finished dirty; 05–07 clean. Judge only that
  recorded state — later `__pycache__`/`.venv` artifacts are the
  acceptance suite's doing, not the agent's.
- Never modify run repos; clone to `/tmp/grade-sol-run03/` to run
  anything.
- **Grading order: 07 → 01** (Fable grades 01 → 07).

## Procedure

1. Per repo, score all 7 dimensions → `$EVAL/_eval/run_03/grades/NN-sol.md`
   (run_01's score-sheet template). Notes:
   - **Correctness**: apply the rubric's mapping table mechanically;
     state the (core, hard) pair and resulting score.
   - **Efficiency**: anchor to the same-runner control. Copilot cells
     report AI credits + premium requests; Claude cells report tokens —
     never compare across runners. Cite the numbers.
   - **Autonomy**: all runs were headless with zero interventions —
     differentiate on finished-and-committed (see each runlog's
     agent-done tree state).
   - Judge the artifact, not the brand; ceremony that didn't change the
     outcome scores LOW on Process.
2. `$EVAL/_eval/run_03/grades/summary-sol.md`: one table — rows = 7
   repos, columns = 7 dimension scores + acceptance (core n/12, hard
   n/9) + weighted total.
3. Commit: `git add _eval/run_03/grades && git commit -m
   "grades(sol): run_03 Haiku-tier rubric pass, all seven repos"`.
   Report the table in chat and stop. No reconciliation, no
   deliverables, no peeking at Fable's work.
