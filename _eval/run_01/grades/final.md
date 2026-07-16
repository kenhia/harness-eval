# Final grades — harness eval POC (consensus)

Produced by the consensus session (fable2) from: adjudicated acceptance
results (`acceptance-adjudication.md`), both graders' rubric sheets, and one
reconciliation round (`reconcile/round1/`). Rules applied: correctness comes
from the adjudicated acceptance pass count; for every other dimension,
|fable1 − sol| ≤ 1 → consensus = mean (halves allowed); the single ≥2 gap
(repo 01 × tests) went to reconciliation and closed in round 1 with both
graders at 5.

**Run 1.5 (2026-07-15):** the field was expanded with `06-gstack` and
`07-baseline-claude` (see the Run 1.5 section below). Runs 01–05's consensus
numbers are frozen and unchanged; the delta pass added one adjudication
(`acceptance-adjudication.md` § Run 1.5) and one reconciliation round
(`reconcile/round1-r15/`), both resolved.

## Consensus scores

| dim (weight) | 01-atv-starterkit | 02-atv-phoenix | 03-working-skill-repo | 04-kprojects | 05-baseline | 06-gstack | 07-baseline-claude |
|---|---:|---:|---:|---:|---:|---:|---:|
| correctness (30) | 5 | 5 | 5 | 5 | 5 | 5 | 5 |
| code quality (20) | 4.5 | 4 | 4 | 4.5 | 4 | 4 | 4 |
| tests (15) | 5 | 4 | 4 | 4.5 | 4 | 5 | 5 |
| docs (10) | 4.5 | 4.5 | 5 | 5 | 5 | 4.5 | 5 |
| process (10) | 3.5 | 5 | 5 | 4.5 | 4.5 | 4.5 | 5 |
| efficiency (10) | 2 | 3.5 | 4.5 | 3.5 | 5 | 1.5 | 4.5 |
| autonomy (5) | 5 | 5 | 5 | 5 | 5 | 5 | 5 |
| **Weighted total** | **88** | **89** | **92** | **92.5** | **92** | **87** | **95** |

## Final ranking (runs 1 + 1.5)

| rank | repo | harness | runner | total |
|---|---|---|---|---:|
| 1 | 07-baseline-claude | none **(control — Claude runner)** | Claude Code | 95 |
| 2 | 04-kprojects | kprojects | Copilot CLI | 92.5 |
| 3= | 03-working-skill-repo | working-skill-repo (KB) | Copilot CLI | 92 |
| 3= | 05-baseline | none **(control — Copilot runner)** | Copilot CLI | 92 |
| 5 | 02-atv-phoenix | ATV-Phoenix | Copilot CLI | 89 |
| 6 | 01-atv-starterkit | ATV-StarterKit 2.x | Copilot CLI | 88 |
| 7 | 06-gstack | gstack | Claude Code | 87 |

The run-1 read stands: within the Copilot field the spread is 4.5 points and
inside the n=1 noise floor. The 1.5 expansion widens the total spread to 8
points, but the two extremes are both Claude-runner entries — the honest
headline is "both bare baselines beat or match every harness on their own
runner," not a strict seven-way order.

## Run 1.5 — delta consensus (06-gstack, 07-baseline-claude)

**What changed.** Two runs added on the Claude Code runner: `06-gstack`
(Claude-native harness; `/autoplan` go command; global piece sandboxed in
the `claude-gstack` HOME-profile) and `07-baseline-claude` (no harness,
`claude-clean` profile — the Claude-runner control mirroring 05's role on
Copilot). Delta grading by fresh `fable1` and `sol` sessions calibrated from
their predecessors' sheets. One factual dispute (06 × A5, the window-boundary
record again) was adjudicated PASS by direct application of run 1's repo-01
precedent — documented, tested half-open bounds are spec-permissible. One
rubric gap ≥2 (06 × code quality, 5 vs 3) went to reconciliation round 1 and
converged: fable1 revised 5→4 (the README's memory claim contradicts
`read_entries`' full materialization — a real dock, not a nit), sol revised
3→4 (the layering earns its keep; the catch-all is an exit policy, not
swallowing). Runs 01–05 were not reopened.

**Runner effect (05 vs 07 — same bare model, no harness, different CLI):**

| dim | 05-baseline (Copilot) | 07-baseline-claude (Claude Code) | Δ |
|---|---:|---:|---:|
| correctness | 5 | 5 | 0 |
| code quality | 4 | 4 | 0 |
| tests | 4 | 5 | **+1** |
| docs | 5 | 5 | 0 |
| process | 4.5 | 5 | +0.5 |
| efficiency | 5 | 4.5 | −0.5 |
| autonomy | 5 | 5 | 0 |
| **total** | **92** | **95** | **+3** |

The Claude-runner control shipped 77 tests to 05's 23, added an unprompted
self-review commit (00f8445) that fixed two real bugs, and handled the
naive-timestamp edge that crashes 05 — the robustness signal that separated
run 1's field appeared *in the bare control* on this runner. It paid +4
minutes of wall-clock for it (8m38s vs 4m32s; cost units are not comparable
across runners). Caveat: n=1 per cell and a possibly later model/tool build,
so "+3 points" is a direction, not a measurement.

**gstack vs its own baseline (06 vs 07 — same runner, harness on/off):**

| dim | 07 (control) | 06-gstack | Δ |
|---|---:|---:|---:|
| correctness | 5 | 5 | 0 |
| code quality | 4 | 4 | 0 |
| tests | 5 | 5 | 0 |
| docs | 5 | 4.5 | −0.5 |
| process | 5 | 4.5 | −0.5 |
| efficiency | 4.5 | 1.5 | **−3** |
| autonomy | 5 | 5 | 0 |
| **total** | **95** | **87** | **−8** |

On its native runner, gstack's machinery improved no dimension. It bought
real margins — the field's most hardened parser (escaped quotes, locale,
hostile bytes), 137 tests to the control's 77, and TODOS.md, a genuinely
useful deferred-work record — at 2.2× the control's wall-clock (19m05s vs
8m38s) and ~3× its tokens/dollars ($10.84 vs $3.61), for a task the control
already delivered at 12/12. This is run 01's robustness-for-spend trade
replayed on the other runner, and consistent with their shared DNA (see
caveats).

**Comparability caveats (restated per ADDING-A-HARNESS.md):**

1. **Runner covariate.** 06/07 ran under Claude Code CLI with
   claude-opus-4-8; 01–05 under Copilot CLI with Claude Opus selected.
   Efficiency units (Copilot credits vs `/cost` dollars/tokens) are not
   comparable across runners — 06 is scored against 07, not against 01–05's
   credit numbers.
2. **Tool builds.** Both 1.5 run logs left the `claude-code version` field
   unfilled. kai's binary reported 2.1.209 at consensus time the same day —
   matching the run-log template's "run-1 era" note — so both runs almost
   certainly shared one build, but the version is formally unrecorded, and
   the model build may differ from run 1's (both CLIs auto-update).
3. **Grader continuity.** Delta graders were new sessions calibrating from
   their predecessors' written sheets, not the original graders' full
   context. Correctly, they could not read `final.md`/`reconcile/` — which
   is why sol re-litigated the already-adjudicated A5 boundary semantics
   with no way to know the precedent existed.
4. **Shared DNA.** ATV-StarterKit bundles gstack (run 01's install includes
   `.gstack`), so 01 and 06 are not independent observations of unrelated
   harnesses — and, notably, they display the same failure pattern
   (deep planning, top-tier robustness, worst-in-field efficiency).

## Raw per-grader scores (pre-consensus)

Run 1:

| dim | 01 f1/sol | 02 f1/sol | 03 f1/sol | 04 f1/sol | 05 f1/sol |
|---|---|---|---|---|---|
| correctness* | 5/4 | 5/5 | 5/5 | 5/5 | 5/5 |
| code quality | 5/4 | 4/4 | 4/4 | 5/4 | 4/4 |
| tests | 5/3 → 5/5ʳ | 4/4 | 4/4 | 5/4 | 4/4 |
| docs | 5/4 | 4/5 | 5/5 | 5/5 | 5/5 |
| process | 4/3 | 5/5 | 5/5 | 5/4 | 4/5 |
| efficiency | 2/2 | 3/4 | 4/5 | 3/4 | 5/5 |
| autonomy | 5/5 | 5/5 | 5/5 | 5/5 | 5/5 |
| weighted | 92/72 | 87/91 | 91/93 | 96/89 | 91/93 |

Run 1.5:

| dim | 06 f1/sol | 07 f1/sol |
|---|---|---|
| correctness* | 5/4 | 5/5 |
| code quality | 5/3 → 4/4ʳ | 4/4 |
| tests | 5/5 | 5/5 |
| docs | 5/4 | 5/5 |
| process | 5/4 | 5/5 |
| efficiency | 2/1 | 5/4 |
| autonomy | 5/5 | 5/5 |
| weighted | 94/74 → 90/78ʳ | 96/94 |

\* correctness was superseded by adjudication (all repos 12/12 → 5).
ʳ reconciled: run 1 round 1 — repo 01 tests, sol revised 3→5, fable1
defended 5; run 1.5 round 1 — repo 06 code quality, fable1 revised 5→4 and
sol revised 3→4.

## Grader agreement notes

- **Repo 01 was the only real divergence (92 vs 72), and it was one root
  cause, not four.** Sol's sealed fixture placed an error record exactly at
  the `--until` bound; repo 01's documented half-open window excluded it.
  Sol read that as a spec violation and propagated it into correctness (4),
  tests (3), docs (4), and process (3). Fable1's fixture had no boundary
  record, so it never saw the issue. Adjudication ruled the semantics
  spec-permissible (see `acceptance-adjudication.md`); reconciliation then
  closed the tests gap in one round. Lesson: one ambiguous spec point ×
  divergent fixture design produced a 20-point swing — sealed acceptance
  must pin boundary semantics, and ideally be a shared executable test.
- **Everywhere else the graders were within 1 point** (33 of 35 dimension
  cells), with mild systematic flavors: fable1 scored code quality/tests a
  bit higher on repo 04 (crediting the naive-timestamp discipline), sol
  scored efficiency a bit higher mid-field (02/03/04) and process lower
  where artifacts read as post-hoc (04's roadmap, 01's plan).
- **Rank order before consensus:** fable1 had 04 > 01 > 03 = 05 > 02; sol
  had 03 = 05 > 02 > 04 > 01. Both independently placed 04 at or near the
  top on artifact quality and agreed exactly on autonomy (5s across) and on
  repo 05's efficiency ceiling.
- **Run 1.5 replayed run 1's pattern in miniature.** The only ≥2 gap (06 ×
  code quality) plus sol's lower 06 marks on docs/process trace to the same
  two roots as run 1: the boundary-semantics reading (sol's A5 FAIL, later
  adjudicated PASS by precedent sol had no access to) and sol's stricter
  prior on machinery-heaviness. 11 of 12 non-correctness cells were within
  1 point; the reconciliation converged in one round with both graders
  moving to 4 from opposite directions — the protocol's cheapest possible
  outcome. On 07 the graders essentially agreed everywhere (96 vs 94).
