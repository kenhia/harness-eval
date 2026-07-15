# Delta consensus + integration prompt — Fable(2): fold in 06-gstack

You are the consensus-and-integration session for the harness comparative
evaluation. Runs 01–05 were graded and consolidated previously (see
`_eval/grades/final.md`); those consensus scores are FROZEN — do not reopen
them. A new contender, `06-gstack`, has been run and independently graded
by `fable1` and `sol`. You: reconcile 06, integrate it into the standings
and deliverables, and publish.

## Environment

Host `kai`, eval repo `/home/ken/src/ai-agents/harness-eval` (`$EVAL`),
staging repo `/home/ken/src/ai-agents/harness-eval-runs/06-gstack`. You run
in a desktop UI: everything over ssh; kai's default shell is fish — always
wrap: `ssh kai bash -lc '...'`.

Context to read: `_eval/README.md`, `_eval/rubric.md`, `_eval/acceptance.md`,
`_eval/grades/final.md` (existing consensus), `_eval/runs/06-runlog.md`,
`_eval/ADDING-A-HARNESS.md` (esp. §1 runner covariate and §7 publish
import), `_eval/report/whitepaper.md`, `_eval/report/lessons-learned.md`.

## Phase 0 — preconditions

All of: `_eval/grades/06-acceptance-fable1.md`, `06-fable1.md`,
`06-acceptance-sol.md`, `06-sol.md`, and a 06 row in both summary files.
Missing anything → report and stop.

## Phase 1 — factual reconciliation (06 only)

Diff the two 06 acceptance sheets. Pass/fail disagreements are factual —
re-run those checks yourself in a fresh clone
(`git clone <staging> /tmp/grade-consensus/06-gstack`), adjudicate, append
to `grades/acceptance-adjudication.md` under a `## Run 1.5 — 06-gstack`
heading. Adjudicated pass count → final correctness score.

## Phase 2 — rubric consensus (06 only)

Per dimension (excluding correctness): |fable1 − sol| ≤ 1 → mean; ≥2 →
reconciliation prompts in `grades/reconcile/round<R>-06/` (self-contained,
quote both scores + justifications, pose the crux; graders append
`## Reconciliation round <R>` to their 06 sheets). Tell Ken when prompts
are ready and stop; resume when re-prompted. After two unresolved rounds,
decide yourself and log why.

## Phase 3 — integrate the standings

Update `grades/final.md`: add the 06 column/row, recompute the ranking,
and add a "Run 1.5" note stating what changed and the comparability caveats
(runner covariate — Claude Code vs Copilot CLI; later tool builds; delta
graders calibrated from sheets rather than sharing run-1 session context).
Do not alter 01–05 consensus numbers.

## Phase 4 — update deliverables

1. `report/whitepaper.md` — add 06 to the results table, write its
   per-harness narrative (check `git log pre-run..HEAD` in the staging
   repo, gstack artifacts: autoplan output, plan reviews, CLAUDE.md
   routing), extend threats-to-validity with the runner covariate, note the
   ATV-StarterKit overlap (StarterKit bundles gstack as a pillar — 01 and
   06 partially share DNA).
2. `report/infographic.html` — add 06; keep it self-contained HTML; also
   republish as an Artifact.
3. Root `README.md` — update the results table (add 06 with its runner
   noted), adjust the intro ("four harnesses" → "five harnesses", note the
   two runners), keep the POC banner intact.
4. `report/lessons-learned.md` + `notes/lessons.md` — what the expansion
   process revealed (runner covariate handling, HOME-sandbox profile for
   Claude Code, anything that fought you during delta grading).

## Phase 5 — publish import + push

Follow `ADDING-A-HARNESS.md` §7 exactly: secret-scan the staging repo
(tracked files + full history + `_eval/runs/transcripts/06-*` if present),
flatten `06-gstack/` into `$EVAL` main, fetch `history/06-gstack` +
`pre-run/06-gstack` refs, commit everything with
`"grades: run 1.5 — integrate 06-gstack (delta consensus + deliverables)"`,
push main + the new refs to origin.

Summarize in chat: adjudications made, reconciliations needed, final
standings delta, and anything future expansions should do differently.
