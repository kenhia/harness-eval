# Final grades — run_03 (loglens, Haiku 4.5 tier) — consensus

Produced by the consensus session (fable2, 2026-07-23) from both
graders' rubric sheets (fable1 graded 01→07, sol graded 07→01,
independently). Correctness was mechanical — fed by the executable
acceptance suite via the rubric's tier-calibrated mapping table — and
both graders' correctness rows match on all seven repos (verified
against `runs/NN-acceptance.txt`; see `adjudication.md`). Across the
other 42 dimension cells **no gap reached the ≥2 reconciliation
threshold** (31 of 49 cells agree exactly; 12 differ by 0.5, six by
exactly 1), so no reconciliation round was held and every consensus
cell is the two-grader mean (quarter-points arise where a half met a
whole). One trivial factual discrepancy (cell 01's hard-miss count) was
resolved from the supplied acceptance output; no score was affected.

> **Suite defect S2.** Cells 03 and 06 were re-graded after suite
> defect S2 (the `loglens_json` fixture's `--format` fallback matched
> argparse's error wording only, failing click-based CLIs on JSON
> checks they pass; S2b made the A12 justfile probe case-tolerant).
> Their sheets carry the re-score marker and their scores here derive
> from the **corrected** suite (03: 12/12, 8/9; 06: 9/12, 8/9). The
> other five cells' tallies were unaffected (re-run confirmed). Full
> record: `../DEFECTS.md`.

## Consensus scores

| dim (weight) | 01-atv-starterkit | 02-atv-phoenix | 03-working-skill-repo | 04-kprojects | 05-baseline | 06-gstack | 07-baseline-claude |
|---|---:|---:|---:|---:|---:|---:|---:|
| correctness (30) | 3 | 2 | 5 | 3.5 | 2 | 2.5 | 3 |
| code quality (20) | 3.25 | 2.25 | 3.75 | 3.25 | 3 | 2.75 | 3 |
| tests (15) | 3.5 | 2.25 | 3 | 2.5 | 2.75 | 2.75 | 3 |
| docs (10) | 3.5 | 3.5 | 3 | 4.25 | 4 | 4 | 4 |
| process (10) | 1.5 | 1.75 | 1.75 | 3 | 3.5 | 2.75 | 2 |
| efficiency (10) | 2 | 4 | 5 | 5 | 5 | 2 | 5 |
| autonomy (5) | 4 | 4 | 4 | 4 | 5 | 5 | 5 |
| **Weighted total** | **59.5** | **50.25** | **77.5** | **70** | **62.25** | **56.75** | **66** |

## Final ranking

| rank | repo | harness | runner | total | acceptance (core, hard) |
|---|---|---|---|---:|---|
| 1 | 03-working-skill-repo | working-skill-repo (KB) | Copilot CLI | 77.5 | 12/12, 8/9 |
| 2 | 04-kprojects | kprojects | Copilot CLI | 70 | 11/12, 8/9 |
| 3 | 07-baseline-claude | none (control — Claude runner) | Claude Code | 66 | 11/12, 4/9 |
| 4 | 05-baseline | none (control — Copilot runner) | Copilot CLI | 62.25 | 9/12, 4/9 |
| 5 | 01-atv-starterkit | ATV-StarterKit 2.6.3 | Copilot CLI | 59.5 | 11/12, 4/9 |
| 6 | 06-gstack | gstack | Claude Code | 56.75 | 9/12, 8/9 |
| 7 | 02-atv-phoenix | ATV-Phoenix | Copilot CLI | 50.25 | 9/12, 4/9 |

## Standing notes

- **Tier scale.** Run_03 scores are calibrated to this tier's own
  controls (05 for Copilot cells, 07 for Claude cells) and its own
  tier-calibrated correctness mapping. They are **not comparable
  point-for-point** to run_01/run_02 totals — a 77.5 here is not "worse
  than" run_02's 93.5. The cross-tier statistic is the
  harness-minus-control delta below; acceptance tallies (same
  executable suite semantics) are also comparable across tiers.
- **S1/I1.** The layout nesting in cells 01 and 03 (project one level
  down at `loglens/`) is treated per the S1 adjudication as a
  **process observation**, not a correctness failure — both sheets
  weigh it under Process, and correctness comes from the fixed suite.
  Cell 05 is a **clean rerun** after incident I1 (its first attempt was
  interrupted mid-run by a controller death and voided; partial
  worktree archived, staging reset to `pre-run` — see `../DEFECTS.md`).
- **S2.** 03's and 06's scores derive from the S2-corrected suite (see
  banner above); their non-correctness dimensions were delta re-graded
  by both graders against the corrected tallies.

## Cross-tier statistic — harness-minus-control delta

The deliverable that answers this tier's motivating question (does
harness value grow as model capability drops, or is there a floor where
machinery hurts?). Each harness cell minus its same-runner control, at
this tier and at run_01's frontier tier (Opus). **Do not compare raw
totals across tiers** — only the deltas.

| harness cell | control | run_03 delta (Haiku 4.5) | run_01 delta (frontier) | shift |
|---|---|---:|---:|---:|
| 01-atv-starterkit | 05 | **−2.75** (59.5 − 62.25) | −4 (88 − 92) | +1.25 |
| 02-atv-phoenix | 05 | **−12** (50.25 − 62.25) | −3 (89 − 92) | −9 |
| 03-working-skill-repo | 05 | **+15.25** (77.5 − 62.25) | 0 (92 − 92) | +15.25 |
| 04-kprojects | 05 | **+7.75** (70 − 62.25) | +0.5 (92.5 − 92) | +7.25 |
| 06-gstack | 07 | **−9.25** (56.75 − 66) | −8 (87 − 95) | −1.25 |

**Read:** both pre-registered shapes from the run_03 README appear, and
they split by harness weight. The light convention/skill harnesses grew
in value as capability dropped — working-skill-repo went from delta ~0
at frontier to **+15.25** at Haiku (only 12/12 core in the field, best
hard tier, at 1.05× control cost), and kprojects from +0.5 to **+7.75**
(11/12 + 8/9 at 0.84× control cost). The heavy go-command harnesses hit
the capability floor: phoenix collapsed from −3 to **−12** (its done
gate green-lit checks that fail in any clean environment — machinery
the model couldn't drive became pure tax), and gstack stayed
significantly negative (−8 → −9.25; the plan-driven tz-aware design did
earn a real hard-tier edge, 8/9 vs the control's 4/9, but at ~2.5×
control cost with a worse core tally). StarterKit was mildly negative
at both tiers (−4 → −2.75): its 548-line plan named the timezone risk
its implementation then shipped anyway, at the field's highest burn.

Caveats: N=1 per cell; correctness (weight 30) dominates the deltas,
and 03's swing rests substantially on one design decision (tz-aware
parsing) the aware-parser repos shared. Direction, not measurement.

## Acceptance tallies (post-S1/S2 suite)

Verified against `runs/NN-acceptance.txt` this session. Same executable
suite semantics as the run_01 retro-validation, so these tallies ARE
cross-tier comparable (run_01's frontier field: core 7/7 repos at
12/12; hard-tier failures limited to H9 on 02/03/05).

| repo | core | hard | failures | root causes |
|---|---|---|---|---|
| 01 | 11/12 | 4/9 | A5; H2 H3 H4 H6 H7 | naive parser — aware windows crash |
| 02 | 9/12 | 4/9 | A5 A7 A12; H2 H3 H4 H6 H7 | naive parser; malformed count never on stderr; bare-python `just check` |
| 03 | 12/12 | 8/9 | H9 | aware parser — naive `--since` crashes |
| 04 | 11/12 | 8/9 | A1; H9 | hand-rolled CLI has no `--help`; naive `--since` crashes |
| 05 | 9/12 | 4/9 | A5 A9 A12; H2 H3 H4 H6 H7 | naive parser; dev tools in optional extras break clean-env pytest/`just check` |
| 06 | 9/12 | 8/9 | A7 A9 A12; H9 | json-mode guard suppresses stderr count; optional-extras packaging; naive `--since` |
| 07 | 11/12 | 4/9 | A5; H2 H3 H4 H6 H7 | naive parser — aware windows crash |

**Tier signature (both graders, independently):** no repo normalized
naive/aware datetimes on both sides. The naive-parser repos (01, 02,
05, 07) crash on aware `--since/--until` (A5 + five hard tests); the
aware-parser repos (03, 04, 06) crash on naive `--since` (H9) —
complementary halves of the same missed edge, and no repo's own test
suite covered the mix. Secondary tier patterns: committed `__pycache__`
`.pyc` files (04, 05, 06, 07), dev tooling declared as optional extras
breaking clean-env checks (05, 06), project nested a level down (01,
03 — no frontier cell did this), and dirty trees at agent-done (01–04).

## Raw per-grader scores (pre-consensus, fable1/sol)

| dim | 01 | 02 | 03 | 04 | 05 | 06 | 07 |
|---|---|---|---|---|---|---|---|
| correctness | 3/3 | 2/2 | 5/5 | 3.5/3.5 | 2/2 | 2.5/2.5 | 3/3 |
| code quality | 3/3.5 | 2/2.5 | 4/3.5 | 3/3.5 | 3/3 | 3/2.5 | 3/3 |
| tests | 3/4 | 2/2.5 | 3/3 | 3/2 | 3/2.5 | 3/2.5 | 3/3 |
| docs | 4/3 | 3/4 | 3/3 | 4/4.5 | 4/4 | 4/4 | 4/4 |
| process | 2/1 | 2/1.5 | 2/1.5 | 3/3 | 3/4 | 3/2.5 | 2/2 |
| efficiency | 2/2 | 4/4 | 5/5 | 5/5 | 5/5 | 2/2 | 5/5 |
| autonomy | 4/4 | 4/4 | 4/4 | 4/4 | 5/5 | 5/5 | 5/5 |
| weighted | 59/60 | 48/52.5 | 79/76 | 70/70 | 62/62.5 | 59/54.5 | 66/66 |

## Grader agreement notes

- **Zero reconciliations — third consecutive round** (run_02 build,
  run_02.1 fix, run_03). The executable acceptance suite plus the
  mechanical mapping again removed the entire correctness dispute
  class; the maximum gap anywhere is 1 point.
- **Near-identical pre-consensus rank order.** fable1: 03 > 04 > 07 >
  05 > 01 = 06 > 02; sol: 03 > 04 > 07 > 05 > 01 > 06 > 02. The only
  difference is fable1's 01/06 tie, which the mean resolves in 01's
  favor. Cell 07 is the only repo with exact agreement on all seven
  dimensions (66/66).
- **Full-agreement rows:** correctness (mechanical, 7/7), efficiency
  (7/7 — both graders applied identical control-anchored banding: 01
  and 06 at 2, 02 at 4, 03/04 at 5, controls at 5), and autonomy (7/7 —
  both docked exactly one point on cells 01–04 for the dirty tree at
  agent-done, the same treatment run_02 applied to its repo 01).
- **The six 1-point gaps are emphasis, not shape** — each pair of notes
  cites the same facts: 01 tests (fable1 3 / sol 4: fable1 docks the
  vacuous stderr assertion and zero tz coverage harder than the
  field-best breadth credits), 01 docs (4/3: sol additionally docks the
  wrong `--format` examples and stale `src/` reference), 01 process
  (2/1: sol weighs the unchecked plan checkboxes and undelivered
  planned layout harder), 02 docs (3/4: fable1 docks the README
  claiming stderr reporting the code doesn't do), 04 tests (3/2: sol
  docks the absent CLI-boundary coverage harder), 05 process (3/4:
  sol credits the six-commit narrative harder than the committed-then-
  ignored `.pyc` residue). The mean absorbs all six without moving any
  rank.
- **Shared findings, both sheets independently:** the naive/aware
  datetime split as the tier signature; the CLI boundary as the
  untested layer exactly where failures lived (03's and 04's suites
  never invoke the binary; 05's and 06's suites can't run from a clean
  checkout); and per-repo, both graders picked substantially the same
  best/worst things (01's plan-vs-implementation timezone punt, 02's
  triply-missed malformed-count requirement, 04's hand-rolled argv
  parser, 06's self-unverifiable repo).
