# Grader prompt — Fable(1): run_02.1 fix-round delta grading

You are one of two independent graders for the harness eval's run_02.1
**fix round**. After run_02 was graded, six of seven implementations
shared an identical defect (malformed feed XML silently treated as a
successful empty fetch). Each of the six agents received a bug report
against its own repo and fixed its own codebase. You grade the **fix
deltas only** — run_02 build grades are frozen and not reopened.

**Your grader id is `fable1`.** Use it in every filename.

## Environment

`$EVAL` = `/home/ken/src/ai-agents/harness-eval` on kai; repos at
`~/src/ai-agents/harness-eval-runs/run_02/NN-<name>`. kai's shell is
fish and `ssh kai bash -lc 'cmd args'` silently drops args — use
`ssh kai bash -s <<'EOF' ... EOF` heredocs for remote work.

## Read first (in order)

1. `$EVAL/_eval/run_02/FIX-ROUND.md` — protocol, **the fix-delta rubric
   (weights differ from run_02!)**, E1 incident + final handling
2. `$EVAL/_eval/run_02/prompts/fix-00-bug-report.md` — the bug report
   all six agents got (note: it does NOT ask for a regression test)
3. Per repo: `runs/NN-fix-runlog.md` and `runs/NN-fix-acceptance.txt`
   (**supplied — never re-derive**; every cell scored core 14/14, hard
   12/12, fix 3/3)
4. `grades/precedents.md`
5. **Calibration**: your own run_02 sheets (`grades/*-fable1.md`,
   `summary-fable1.md`). Sol's sheets, `final.md`, `reconcile/` remain
   off-limits.

## The field

01, 02, 04, 05, 06, 07. **03 is absent because it passed run_02 26/26
(strict roxmltree) — nothing to fix.** Do not penalize or reward absent
repos; simply no sheet for 03.

## Hard rules

- Agent-authored fix = `git diff pre-fix..HEAD` (+ log). Grade ONLY
  that delta; the run_02 build is out of scope except as context.
- **Repo 01**: its final commit (`.atv` hook-telemetry snapshot) is
  EVAL-authored, not agent work — exclude it from all judgments.
- **Repo 02**: grade the current `pre-fix..HEAD` only. A first fix
  attempt was voided for environment reasons (E1) — do not read branch
  `void/fix-e1`, anything in `.scratch/`, or speculate about it. The
  graded rerun had the phoenix MCP spine (verified).
- Environment covariate (uniform, not scoreable): copilot cells ran
  with ambient klams/korg MCP unapproved/absent; phoenix's own MCP was
  present (harness component). Claude cells keep profile MCP.
- Never modify run repos; clone to /tmp/grade-fable1-fix/ to run tests.
- **Grading order: 01 → 07** (Sol grades 07 → 01).

## Procedure

1. Per repo, score the five fix-delta dimensions from FIX-ROUND.md
   (fix correctness 35 / fix quality 30 / tests 20 / scope & process 10
   / efficiency 5), each 0–5 with 2–4 sentences citing files/commits →
   `$EVAL/_eval/run_02/grades/NN-fix-fable1.md`. Notes:
   - Fix correctness: acceptance is uniform (26/26 + 3/3 everywhere) —
     this dimension differentiates only on *how robustly* the pass is
     achieved (e.g. error propagation vs incidental behavior); expect
     high scores, justify any deduction concretely.
   - Fix quality: root cause = strict parse-failure detection at the
     XML layer with faithful error recording; deduct for special-cased
     heuristics, over-broad rewrites, or robustness the bug didn't ask
     for. Note WHERE each fix landed (parser vs fetch loop vs handler).
   - Tests: the bug report did not request a regression test — an
     unprompted, genuine one scores high; none, or a tautological one,
     scores low.
   - Scope: minimal blast radius; commits coherent.
   - Efficiency: wall clock/tokens from the fix runlogs, judged within
     runner (cost units differ) and against the fix-round field
     (1m29s–6m42s). Cite numbers.
2. `$EVAL/_eval/run_02/grades/summary-fix-fable1.md`: one table — rows
   = 6 repos, columns = 5 dimensions + weighted total (/100).
3. Commit: `git add _eval/run_02/grades && git commit -m
   "grades(fable1): run_02.1 fix-round delta pass"`. Report the table
   in chat and stop. No reconciliation, no deliverables, no peeking.
