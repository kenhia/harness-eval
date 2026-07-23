# Harnesses matter more when the model is weaker — but only the light ones

*harness-eval run 03, 2026-07-23. The same seven cells as runs 1 and 02
— five harnesses, two bare controls, two runners — on run 1's frozen
loglens spec, with **one variable changed: the model is Claude Haiku
4.5 instead of a frontier model.** Harness versions, prompts, spec, and
acceptance semantics all held constant. Preliminary: N=1 per cell.*

## The question

Mark Rytting (author of working-skill-repo) asked the obvious question
after run 02: harnesses looked like ceremony at the frontier — but
frontier models don't need much help. **Does harness value grow as
model capability drops?** Runs 1 and 02 couldn't answer it: every cell
used the same frontier model.

Run 03 adds the **model-capability axis**. Everything else is pinned so
the comparison is single-variable, and this tier is graded on its own
scale against its own controls — because a tier's raw totals are not
comparable to another tier's. The cross-tier statistic is the
**harness-minus-control delta**.

## Method (what changed, what didn't)

Same frozen loglens spec and prompts as run 1, byte-identical. Same five
harnesses at the **same versions** installed for run 02 (StarterKit
2.6.3, KB @ 34804ea, kprojects, gstack 1.60.1.0, Phoenix profile). Same
runners, headless, zero interventions. Model: `claude-haiku-4.5`
(Copilot cells) / `claude-haiku-4-5-20251001` (Claude cells), verified
in every run's session metrics — no silent fallback.

**Acceptance is the executable loglens suite** built for this tier: run
1's A1–A12 checklist ported to hermetic pytest (core, 12 checks) plus a
hard tier (9 probes) mechanizing the robustness edges run 1's graders
found by hand. It was **retro-validated against the seven graded run_01
trees**: all seven pass core 12/12, matching their adjudicated results,
and the hard tier's H9 probe reproduces run 1's naive-datetime finding
on exactly the repos its graders flagged. Correctness maps from the
tallies through a **tier-calibrated mechanical table** — run 02's
"12/12 → 4" rule would have flattened this entire field.

## Results — this tier, on its own scale

| rank | repo | harness | runner | total | core | hard | cost |
|---|---|---|---|---:|---|---|---|
| 1 | 03 | working-skill-repo (KB) | Copilot | **77.5** | **12/12** | 8/9 | 59.7 cr |
| 2 | 04 | kprojects | Copilot | 70 | 11/12 | 8/9 | 48.1 cr |
| 3 | 07 | none (control) | Claude Code | 66 | 11/12 | 4/9 | 22.2k tok |
| 4 | 05 | none (control) | Copilot | 62.25 | 9/12 | 4/9 | 57.1 cr |
| 5 | 01 | ATV-StarterKit 2.6.3 | Copilot | 59.5 | 11/12 | 4/9 | 153.8 cr |
| 6 | 06 | gstack | Claude Code | 56.75 | 9/12 | 8/9 | 34.0k tok |
| 7 | 02 | ATV-Phoenix | Copilot | 50.25 | 9/12 | 4/9 | 64.0 cr |

**One repo (03) achieved a full core pass**; the frontier field went
7-for-7. The suite that could not separate frontier models separates
this one cleanly — core 9–12/12, hard 4–8/9, totals spanning 27 points.

## The cross-tier statistic — the headline

Each harness cell minus its **same-runner control**, at this tier and at
run 1's frontier tier. Raw totals are not comparable across tiers; these
deltas are.

| harness | frontier delta | Haiku delta | shift |
|---|---:|---:|---:|
| working-skill-repo (KB) | 0 | **+15.25** | **+15.25** |
| kprojects | +0.5 | **+7.75** | +7.25 |
| ATV-StarterKit | −4 | −2.75 | +1.25 |
| gstack | −8 | −9.25 | −1.25 |
| ATV-Phoenix | −3 | **−12** | −9 |

**The answer to Mark's question is yes — with a decisive caveat. It
splits by harness weight.**

**Light convention/skill harnesses gained enormously.** KB went from
*exactly zero* delta at the frontier to **+15.25** at Haiku: the only
full core pass in the field, the best hard tier, at 1.05× control cost.
kprojects went +0.5 → +7.75 (11/12 + 8/9 at 0.84× control cost — it
beat its control *and* was cheaper). These harnesses supply conventions
and review habits the frontier model already had, and the weaker model
did not.

**Heavy go-command harnesses hit a capability floor and got worse.**
Phoenix collapsed −3 → **−12**: its done-gate green-lit checks that fail
in any clean environment — an autonomous verification loop the model
couldn't actually drive became pure tax. gstack stayed firmly negative
(−8 → −9.25); its plan-driven design did earn a real hard-tier edge
(8/9 vs the control's 4/9) but at ~2.5× control cost with a worse core
tally. StarterKit was mildly negative at both tiers, and its failure is
almost poetic: a 548-line plan that **named the timezone risk its
implementation then shipped anyway**, at the field's highest burn.

The mechanism worth stating plainly: **machinery that encodes knowledge
helps a weaker model; machinery that demands the model drive an
autonomous process needs capability the weaker model doesn't have, and
the process becomes overhead it must service.** That's a sharper claim
than "harnesses help small models," and it's actionable — it predicts
which harness to reach for at which capability tier.

## The tier signature — a failure the frontier didn't have

Both graders independently identified the same defining pattern: **no
repo normalized naive/aware datetimes on both sides.** The field split
into complementary halves:

- **Naive-parser repos (01, 02, 05, 07)** crash on timezone-aware
  `--since/--until` — A5 plus five hard probes.
- **Aware-parser repos (03, 04, 06)** crash on a *naive* `--since` (H9).

Every repo solved exactly one half of the problem and no repo's own test
suite covered the mix. At the frontier this class of edge produced a
handful of scattered misses; here it is the organizing failure of the
entire field. Secondary tier patterns, none seen at the frontier:
committed `.pyc` files (04, 05, 06, 07), dev tooling declared as
optional extras so clean-environment checks break (05, 06), the project
nested a directory down (01, 03), and dirty trees at agent-done (01–04).

## What the method bought this run

- **Zero grader reconciliations — third consecutive round.** 31 of 49
  dimension cells agreed exactly; the maximum gap anywhere was 1 point;
  pre-consensus rank order was near-identical (one tie apart).
  Correctness matched 7/7 mechanically.
- **Two suite defects, both caught and adjudicated, neither silent.**
  **S1**: two repos nested the project a level down and the suite scored
  them 0/12 — a working CLI reported as total failure. The spec doesn't
  pin the directory level, so nesting became a scoreable *process*
  observation instead. **S2**: the JSON fallback matched argparse's
  error wording only, so click-based CLIs failed JSON checks they
  actually pass — **found by a grader mid-grading, who flagged it for
  adjudication rather than self-adjudicating**, and independently
  corroborated by the other grader observing the same symptom with the
  opposite attribution. Fixing S2 moved 03 from 10/12 to a full pass and
  changed the field's shape; both affected cells were delta re-graded by
  both graders against corrected results.
- **An interrupted run (I1)** — a controller disconnect killed the
  matrix mid-cell — was detected by its signature (uncommitted worktree,
  no runlog), voided, and cleanly resumed.

Every one of those would have silently corrupted a hand-graded field.

## Threats to validity

- **N=1 per cell.** Run 02 measured 44% wall-clock variance between two
  reps of one cell, with the reps landing on opposite sides of a
  decisive check. Treat direction, not magnitude, as the finding.
- **Correctness (weight 30) dominates the deltas**, and 03's swing rests
  substantially on one design decision — timezone-aware parsing — shared
  by all three aware-parser repos. A single different dependency choice
  could move it.
- **Cross-tier deltas inherit both tiers' noise**, and the frontier
  numbers come from run 1's prose-checklist grading, a weaker instrument
  than this run's executable suite.
- **Runner drift**: Copilot 1.0.73 / Claude Code 2.1.217, both later
  than run 02's builds; auto-updating CLIs make this unavoidable.
- **Cost units** aren't comparable across runners or across CLI
  versions; premium requests (0.33/cell here vs 15 at the frontier) and
  wall clock travel best.
- Unchanged from prior runs: the eval author wrote one contender
  (kprojects), and harness artifacts self-identify, so blinding is
  imperfect.

## What's next

The obvious follow-up is the third rung: **a local open-weight model**
(Gemma via BYOK to a vLLM endpoint), where the same seven cells cost
nothing per run — making N≥3 reps affordable for the first time, and
extending the capability ladder far enough to test whether the
light-harness gain keeps growing or itself hits a floor. If the pattern
here holds, the practical guidance writes itself: **the weaker your
model, the more you want conventions — and the less you want ceremony.**
