# Final grades — harness eval POC run 1 (consensus)

Produced by the consensus session (fable2) from: adjudicated acceptance
results (`acceptance-adjudication.md`), both graders' rubric sheets, and one
reconciliation round (`reconcile/round1/`). Rules applied: correctness comes
from the adjudicated acceptance pass count; for every other dimension,
|fable1 − sol| ≤ 1 → consensus = mean (halves allowed); the single ≥2 gap
(repo 01 × tests) went to reconciliation and closed in round 1 with both
graders at 5.

## Consensus scores

| dim (weight) | 01-atv-starterkit | 02-atv-phoenix | 03-working-skill-repo | 04-kprojects | 05-baseline |
|---|---:|---:|---:|---:|---:|
| correctness (30) | 5 | 5 | 5 | 5 | 5 |
| code quality (20) | 4.5 | 4 | 4 | 4.5 | 4 |
| tests (15) | 5 | 4 | 4 | 4.5 | 4 |
| docs (10) | 4.5 | 4.5 | 5 | 5 | 5 |
| process (10) | 3.5 | 5 | 5 | 4.5 | 4.5 |
| efficiency (10) | 2 | 3.5 | 4.5 | 3.5 | 5 |
| autonomy (5) | 5 | 5 | 5 | 5 | 5 |
| **Weighted total** | **88** | **89** | **92** | **92.5** | **92** |

## Final ranking

| rank | repo | harness | total |
|---|---|---|---:|
| 1 | 04-kprojects | kprojects | 92.5 |
| 2= | 03-working-skill-repo | working-skill-repo (KB) | 92 |
| 2= | 05-baseline | none (control) | 92 |
| 4 | 02-atv-phoenix | ATV-Phoenix | 89 |
| 5 | 01-atv-starterkit | ATV-StarterKit 2.x | 88 |

The spread is 4.5 points on a 100-point scale — with n=1 per cell this is
inside the noise floor; treat the ranking as "04 slightly ahead, 01 paid the
most for the least efficiency, everything else indistinguishable" rather
than a strict order.

## Raw per-grader scores (pre-consensus)

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

\* correctness was superseded by adjudication (all repos 12/12 → 5).
ʳ reconciled in round 1: sol revised 3→5; fable1 defended 5.

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
