# Delta consensus + integration prompt — Fable(2): fold in 06-gstack and 07-baseline-claude

You are the consensus-and-integration session for the harness comparative
evaluation. Runs 01–05 were graded and consolidated previously
(`_eval/grades/final.md`); those consensus scores are FROZEN — do not
reopen them. Two new runs have been completed and independently graded by
`fable1` and `sol`:

- `06-gstack` — new contender (Claude-Code-native harness)
- `07-baseline-claude` — control: no harness, Claude Code runner. Its
  analytical purpose: 05-vs-07 isolates the runner effect on a bare
  baseline; 06-vs-07 isolates gstack's contribution on its native runner.

You: reconcile both, integrate them into the standings and deliverables,
and publish.

## Environment

Host `kai`, eval repo `/home/ken/src/ai-agents/harness-eval` (`$EVAL`),
staging repos `/home/ken/src/ai-agents/harness-eval-runs/{06-gstack,07-baseline-claude}`.
You run in a desktop UI: everything over ssh; kai's default shell is fish —
always wrap: `ssh kai bash -lc '...'`.

Context to read: `_eval/README.md`, `_eval/rubric.md`, `_eval/acceptance.md`,
`_eval/grades/final.md`, `_eval/runs/06-runlog.md`, `_eval/runs/07-runlog.md`,
`_eval/ADDING-A-HARNESS.md` (esp. §1 runner covariate, §7 publish import),
`_eval/report/whitepaper.md`, `_eval/report/lessons-learned.md`.

## Phase 0 — preconditions

Under `$EVAL/_eval/grades/`, for NN in {06, 07}: `NN-acceptance-fable1.md`,
`NN-fable1.md`, `NN-acceptance-sol.md`, `NN-sol.md`, and NN rows in both
summary files. Missing anything → report and stop.

## Phase 1 — factual reconciliation (06 and 07)

Diff the graders' acceptance sheets per run. Pass/fail disagreements are
factual — re-run those checks yourself in fresh clones
(`/tmp/grade-consensus/NN-…`), adjudicate, and append to
`grades/acceptance-adjudication.md` under a `## Run 1.5` heading.
Adjudicated pass counts → final correctness scores.

## Phase 2 — rubric consensus (06 and 07)

Per run × dimension (excluding correctness): |fable1 − sol| ≤ 1 → mean;
≥2 → reconciliation prompts in `grades/reconcile/round<R>-r15/`
(self-contained: quote both scores + justifications verbatim, pose the
crux; graders append `## Reconciliation round <R>` to their own sheets).
Tell Ken when a round's prompts are ready and stop; resume when
re-prompted. After two unresolved rounds, decide yourself and log why.

## Phase 3 — integrate the standings

Update `grades/final.md`:

- Add 06 and 07 columns/rows; 07 is annotated **(control — Claude runner)**
  just as 05 is the Copilot-runner control.
- Recompute the ranking with all seven entries.
- Add a "Run 1.5" section: what changed, the runner-effect readout
  (05 vs 07 on every dimension) and the gstack readout (06 vs 07),
  plus comparability caveats (runner covariate; later tool builds — check
  the `claude-code version` fields in both run logs; delta graders
  calibrated from sheets rather than sharing run-1 session context;
  ATV-StarterKit bundles gstack, so 01 and 06 share DNA).
- Do not alter 01–05 consensus numbers.

## Phase 4 — update deliverables

1. `report/whitepaper.md` — results table gains 06 and 07; per-harness
   narrative for 06 (inspect `git log pre-run..HEAD` in staging, gstack
   artifacts: autoplan output, plan reviews, CLAUDE.md routing) and a short
   07 note; NEW subsection under results: "Runner effect" (05 vs 07) and
   "gstack vs its own baseline" (06 vs 07); extend threats-to-validity
   (runner covariate, 01/06 shared DNA).
2. `report/infographic.html` — add both entries (07 visually marked as a
   control, like 05); keep self-contained; republish as an Artifact.
3. Root `README.md` — update results table (both rows, runner column or
   footnote), adjust intro (five harnesses + two baselines, two runners),
   keep the POC banner intact.
4. `report/lessons-learned.md` + `notes/lessons.md` — what the expansion
   process revealed (runner covariate handling, HOME-sandbox profiles,
   the profile-contamination near-miss that motivated splitting
   claude-clean/claude-gstack, anything that fought you in delta grading).

## Phase 5 — publish import + push

Follow `ADDING-A-HARNESS.md` §7 for BOTH staging repos: secret-scan
(tracked files + full history + any `_eval/runs/transcripts/0{6,7}-*`),
flatten each into `$EVAL` main, fetch `history/NN-…` + `pre-run/NN-…`
refs, commit with
`"grades: run 1.5 — integrate 06-gstack + 07-baseline-claude"`,
push main + the four new refs to origin.

Summarize in chat: adjudications, reconciliations, final standings, the
runner-effect and gstack-vs-baseline readouts, and anything future
expansions should do differently.
