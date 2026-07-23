# Factual adjudication — run_03 consensus (fable2, 2026-07-23)

## Correctness (mechanical) — verified, no disputes

Both graders' correctness rows match on all seven repos, and every row
was re-verified in this session against the supplied acceptance outputs
(`runs/NN-acceptance.txt`, post-S1/S2 suite) and the rubric's mapping
table:

| cell | core | hard | failures | mapped score (both graders) |
|---|---|---|---|---|
| 01 | 11/12 | 4/9 | A5; H2 H3 H4 H6 H7 | 3 |
| 02 | 9/12 | 4/9 | A5 A7 A12; H2 H3 H4 H6 H7 | 2 |
| 03 | 12/12 | 8/9 | H9 | 5 (post-S2) |
| 04 | 11/12 | 8/9 | A1; H9 | 3.5 |
| 05 | 9/12 | 4/9 | A5 A9 A12; H2 H3 H4 H6 H7 | 2 |
| 06 | 9/12 | 8/9 | A7 A9 A12; H9 | 2.5 (post-S2) |
| 07 | 11/12 | 4/9 | A5; H2 H3 H4 H6 H7 | 3 |

The mapping was applied correctly in all 14 sheets. No adjudication
needed.

## One trivial factual discrepancy — resolved from supplied material

**Cell 01, hard-miss root-cause count.** fable1's sheet says the A5 core
miss "and four hard misses share one root cause"; sol's sheet says
"five hard misses share one cause." The supplied acceptance output
settles it: 01's hard failures are H2/H3/H4/H6/H7 — **five** — all
aware-window crashes from the naive timestamp parser. fable1's own
tier-wide summary note lists the same five tests (A5 + H2/H3/H4/H6/H7),
so this is a count slip in one sheet, not a disagreement about the
repo. No dimension score rests on the count; nothing changes.

No fresh-clone adjudication was required this run: the single
discrepancy resolved from the supplied acceptance output, and every
other fact cited by both sheets — repo test counts (48/22/24/23/29/29/25),
commit counts and messages, tree state at agent-done (01–04 dirty,
05–07 clean), and all efficiency figures — agrees between graders and
was spot-checked against `runs/NN-runlog.md`.
