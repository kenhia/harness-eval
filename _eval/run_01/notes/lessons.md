# Lessons learned — running list

Seeded during setup (2026-07-14, before any runs). Add to this after every
run and during grading; this file becomes the "evaluation harness v2" design
input.

## Found during setup

1. **Global-scope harnesses contaminate everything.** kai's live `~/.copilot`
   already had Phoenix (19 skills, agent, MCP server) globally installed —
   the "baseline" run would have had Phoenix active. Environment isolation is
   not optional; it is the first thing an eval harness must provide. POC
   answer: symlink-swapped `~/.copilot` profiles. Real answer: per-run
   containers or throwaway users.
2. **Install scope varies wildly and is itself a finding.** StarterKit and
   working-skill-repo install repo-local; Phoenix is global-only; kprojects
   is repo-local + one optional global skill. Repo-local scope is a
   *feature* for teams (versioned, reviewable, no cross-project bleed).
3. **The registered Phoenix MCP binary was missing** (`target/release`
   cleaned at some point) — a silent global breakage no harness surfaced.
   Eval harness v2 should include a preflight "harness self-check" step
   (Phoenix has `phoenix-mcp doctor`; others have nothing equivalent).
4. **Runner choice is a confound.** All three external harnesses are
   Copilot-first; "give the prompt to Opus" required picking Copilot CLI +
   Opus model as the runner so every harness's machinery actually loads.
   A harness×runner matrix is a future axis.
5. **working-skill-repo's documented install is global**, but the installer
   has an undocumented-ish `--target repo` that is repo-local — reading the
   installer beat reading the README.
6. **Commit-boundary discipline enables grading.** empty-baseline commit →
   install commit → `pre-run` tag makes "agent-authored work" a clean
   `git diff pre-run..HEAD`. Keep this in v2.

## To evaluate after run 1 (fill in)

- Did any harness's ceremony (plans, reviews, handoffs) measurably change the
  outcome vs baseline?
- Token burn per harness (harness overhead = tokens minus baseline tokens).
- Intervention count — which harness needed the most hand-holding?
- Grader agreement — where did Fable and GPT Sol diverge and why?

## v2 orchestration ideas (from the brainstorm)

- **Task-type matrix**, not just greenfield: (a) greenfield build (this run),
  (b) bug-fix session on a planted-bug repo, (c) refactor-without-behavior-
  change on a messy repo, (d) "resume a half-finished project from its own
  docs" — the last one is where handoff-heavy harnesses (KB, kprojects
  sprints) should shine or be exposed.
- **N≥3 repetitions per cell** — single runs are noise; report medians.
- **Scripted runs**: `copilot -p "$(cat prompt.md)"`-style headless
  invocation per run dir, so a justfile can execute the whole matrix.
- **Automatic metrics capture**: wall-clock, session log parse for
  tokens/turns, `git diff --stat`, test/coverage counts — no human notes.
- **Sealed acceptance as executable pytest**, not a checklist a grader
  interprets.
- **Cross-model axis** later: same harness × {Opus, Sonnet, GPT-x} to test
  the "harnesses matter less as models improve" hypothesis explicitly.

## Filled in after run 1 — grading + consensus (2026-07-15)

Answers to the "to evaluate after run 1" questions above; full analysis in
`_eval/report/lessons-learned.md`.

- **Did ceremony change the outcome?** No — all five runs passed all 12
  acceptance checks (after one adjudication), zero interventions everywhere.
  Ceremony changed *cost*, and in two cases bought robustness outside the
  checklist (kprojects' tested tz-normalization; StarterKit's review loop
  catching a real bug pre-done). Baseline tied for 2nd on the weighted rubric.
- **Token burn / harness overhead** (credits, × baseline 143): KB 216
  (1.51×), kprojects 252 (1.76×), Phoenix 265 (1.85×), StarterKit 440
  (3.08×). Wall-clock overhead tracked credits (1.47×–2.40×).
- **Intervention count:** zero, all runs — this dimension didn't
  discriminate either.
- **Grader agreement:** 33 of 35 non-correctness dimension cells within 1
  point. The exception was a 20-point pre-consensus swing on repo 01, root
  cause: the spec never pinned `--until` boundary semantics and the two
  graders' independently generated fixtures differed in whether they probed
  the boundary (honest PASS and FAIL on the same check). Reconciliation
  closed it in one round.

New lessons from the grading phase:

7. **The acceptance checklist did not discriminate at frontier capability.**
   All discriminating signal came from edges *outside* it (offset-less ISO
   8601 bounds crash 3 of 5 repos with a raw `TypeError`). v2 acceptance
   needs a hard tier designed to spread the field.
8. **Sealed acceptance must be one shared executable suite with pinned edge
   semantics.** Grader-interpreted checklists + per-grader fixtures produced
   the run's only dispute; "filter correctly" is not a pass criterion.
   Corollary: spec-pinned edges work — the tie-break rule was pinned and got
   five identical correct implementations.
9. **Efficiency scoring was circular** (anchored to the observed field's
   best/worst). v2: anchor bands to the baseline cell's median across reps.
10. **Ambient MCP services are part of the environment definition.** korg
    being available let run 04 file a work item — a process artifact other
    harnesses don't produce. Decide in/out per cell and record it.

## Filled in after run 1.5 — expansion: 06-gstack + 07-baseline-claude (2026-07-15)

First use of the ADDING-A-HARNESS.md process (staging repos, HOME-sandbox
profiles, delta grading with frozen priors, precedent-based adjudication).
Full analysis in `_eval/report/lessons-learned.md` §run 1.5; headlines:

- **The runner is the biggest effect measured so far.** The bare Claude
  Code control (07) scored 95 and topped the seven-entry field; 05-vs-07
  (same bare model, different CLI) is +3 points, more than any harness's
  margin over its own baseline. gstack (06) landed 8 points *below* its
  same-runner control at ~3× the cost — run 01's robustness-for-spend
  trade replayed, unsurprising given 01 bundles gstack.
- **Claude Code HOME-sandboxes: three sharp edges** now codified in
  ADDING-A-HARNESS.md §2 — profiles need their own `settings.json`
  (permissions don't inherit), `.claude.json` lives at the profile root,
  and profiles must be built under their final name (gstack bakes absolute
  paths into hooks/binaries).
- **Near-miss: gstack's global piece initially landed in `claude-clean`** —
  the control's profile — before being split out to `claude-gstack`. The
  Phoenix contamination lesson generalizes to every new runner: one profile
  per environment flavor, controls never share.
- **The A5 boundary dispute recurred identically on 06** (sol's fixture
  probes the boundary; repo documents half-open; adjudicated PASS by run-1
  precedent). Frozen-delta graders can't see prior adjudications, so
  ambiguity gets re-litigated every expansion until the acceptance suite is
  executable with pinned semantics.
- **Run logs left `claude-code version` unfilled** — with auto-updating
  CLIs that's a lost covariate; v2 preflight captures versions
  mechanically.
- **Delta-grading calibration from written sheets held** (07: 96/94; the
  one ≥2 gap closed in one round with both graders converging on 4).
- **Objective checklist: 7/7 repos at 12/12** — still zero discrimination;
  the hard tier remains v2's highest-value change.
