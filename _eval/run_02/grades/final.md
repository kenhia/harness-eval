# Final grades — run_02 (feedhub) — consensus

Produced by the consensus session (fable2) from both graders' rubric
sheets (fable1 graded 01→07, sol graded 07→01, independently).
Correctness was mechanical this run — fed by the executable acceptance
suite (post-S1 fix) — and both graders' correctness rows match on all
seven repos. Across the other 42 dimension cells **no gap reached the
≥2 reconciliation threshold** (35 of 49 cells agree exactly; the other
14 differ by exactly 1), so no reconciliation round was held and every
consensus cell is the two-grader mean (halves allowed). One factual
discrepancy (06-gstack's test count) was adjudicated in a fresh clone —
`cargo test --workspace` runs 165 tests; see `adjudication.md`. No
score was affected.

## Consensus scores

| dim (weight) | 01-atv-starterkit | 02-atv-phoenix | 03-working-skill-repo | 04-kprojects | 05-baseline | 06-gstack | 07-baseline-claude |
|---|---:|---:|---:|---:|---:|---:|---:|
| correctness (30) | 3 | 3 | 5 | 3 | 3 | 3 | 3 |
| code quality (20) | 3.5 | 3.5 | 4.5 | 3.5 | 3.5 | 4.5 | 4.5 |
| tests (15) | 2 | 3.5 | 3.5 | 3 | 3.5 | 4.5 | 4.5 |
| docs (10) | 5 | 5 | 5 | 5 | 5 | 5 | 5 |
| process (10) | 4.5 | 3 | 5 | 4.5 | 5 | 5 | 5 |
| efficiency (10) | 5 | 5 | 5 | 5 | 5 | 4 | 5 |
| autonomy (5) | 4 | 5 | 5 | 5 | 5 | 5 | 5 |
| **Weighted total** | **71** | **73.5** | **93.5** | **75** | **77.5** | **82.5** | **84.5** |

## Final ranking

| rank | repo | harness | runner | total | acceptance (core, hard) |
|---|---|---|---|---:|---|
| 1 | 03-working-skill-repo | working-skill-repo (KB) | Copilot CLI | 93.5 | 14/14, 12/12 |
| 2 | 07-baseline-claude | none (control — Claude runner) | Claude Code | 84.5 | 13/14, 11/12 |
| 3 | 06-gstack | gstack | Claude Code | 82.5 | 13/14, 11/12 |
| 4 | 05-baseline | none (control — Copilot runner) | Copilot CLI | 77.5 | 13/14, 11/12 |
| 5 | 04-kprojects | kprojects | Copilot CLI | 75 | 13/14, 11/12 |
| 6 | 02-atv-phoenix | ATV-Phoenix | Copilot CLI | 73.5 | 13/14, 11/12 |
| 7 | 01-atv-starterkit | ATV-StarterKit 2.6.3 | Copilot CLI | 71 | 13/14, 11/12 |

Read with the README's variance caveat (N=1 per cell; the 99/07 rep
pair measured ~44% single-run wall-clock spread; the identical frozen
prompt produced 26/26 in the shakedown rep and 24/26 in cell 07 — the
decisive C9 behavior itself sits inside trajectory variance). The
22.5-point spread is dominated by the correctness gate: six of seven
repos share one root-cause failure — a hand-driven quick-xml event loop
that treats `Event::Eof` as normal termination, so truncated XML parses
as a valid empty feed and refresh reports success instead of recording
`last_error` (C9 + H12). Every one of those six also authored its own
`malformed.xml` in the mismatched-tag flavor its parser *does* catch,
so their suites certified the failing behavior. 03's full pass traces
to a dependency choice made before any test ran: roxmltree's strict
whole-document parsing makes truncation an error by construction.

## Acceptance tallies (post-S1 suite)

| repo | core | hard | failures |
|---|---|---|---|
| 01 | 13/14 | 11/12 | C9, H12 — shared Eof-truncation root cause |
| 02 | 13/14 | 11/12 | C9, H12 — same |
| 03 | 14/14 | 12/12 | — |
| 04 | 13/14 | 11/12 | C9, H12 — same |
| 05 | 13/14 | 11/12 | C9, H12 — same |
| 06 | 13/14 | 11/12 | C9, H12 — same (H9 flipped to PASS under the S1 fix; see README defect log) |
| 07 | 13/14 | 11/12 | C9, H12 — same |

## Raw per-grader scores (pre-consensus, fable1/sol)

| dim | 01 | 02 | 03 | 04 | 05 | 06 | 07 |
|---|---|---|---|---|---|---|---|
| correctness | 3/3 | 3/3 | 5/5 | 3/3 | 3/3 | 3/3 | 3/3 |
| code quality | 4/3 | 4/3 | 5/4 | 4/3 | 4/3 | 5/4 | 5/4 |
| tests | 2/2 | 4/3 | 4/3 | 3/3 | 4/3 | 5/4 | 5/4 |
| docs | 5/5 | 5/5 | 5/5 | 5/5 | 5/5 | 5/5 | 5/5 |
| process | 5/4 | 3/3 | 5/5 | 5/4 | 5/5 | 5/5 | 5/5 |
| efficiency | 5/5 | 5/5 | 5/5 | 5/5 | 5/5 | 4/4 | 5/5 |
| autonomy | 4/4 | 5/5 | 5/5 | 5/5 | 5/5 | 5/5 | 5/5 |
| weighted | 74/68 | 77/70 | 97/90 | 78/72 | 81/74 | 86/79 | 88/81 |

## Grader agreement notes

- **Zero reconciliations — a first for this eval.** Run 1 and run 1.5
  each needed a round; run_02's maximum gap is 1 point. The executable
  acceptance suite removed the entire dispute class that produced run
  1's 20-point repo-01 swing (divergent sealed fixtures on ambiguous
  spec points): correctness matched mechanically everywhere.
- **Identical pre-consensus rank order.** Both graders independently
  ordered the field 03 > 07 > 06 > 05 > 04 > 02 > 01. All remaining
  disagreement is level, not shape.
- **The level disagreement is one systematic offset.** Sol scored code
  quality exactly one point below fable1 on all seven repos (and tests
  one below on five), consistently docking parser-precision shortcuts
  (namespace stripping, synthetic entry identities, lossy UTF-8
  decoding) that fable1 noted without docking — the same "stricter
  prior on precision/machinery" flavor recorded in run 1's notes. The
  mean absorbs the offset without moving any rank.
- **Full-agreement rows:** correctness (mechanical), docs (both graders
  gave all seven repos 5 — the field uniformly wrote complete READMEs),
  efficiency (all five Copilot cells landed 0.77–1.05× of the control
  with identical premium-request counts, so the dimension did not
  discriminate; both graders put 06 at 4 as the only signal), and
  autonomy (both gave 01 a 4 for the dirty-tree handoff — uncommitted
  `.atv` runtime state needing the evaluator snapshot commit `c950337`,
  a treatment pre-specified in the grader prompts — and 5s elsewhere).
- Both graders also independently identified the same single worst
  thing on nearly every repo: the malformed-fixture blind spot (own
  fixture in the catchable flavor) and missing feedctl exit-code tests
  (present only in the two Claude-runner cells).

## Fix round (run_02.1)

Consensus by the fix-round consensus session (fable2, 2026-07-17) from
both graders' fix sheets (`NN-fix-fable1.md` / `NN-fix-sol.md`, written
independently). Six repos participated; 03 sat out absent-by-passing
(context notes below). Run_02 build grades above are frozen and
untouched — this section is the fix-round delta only, on the fix rubric
(weights 35/30/20/10/5; see `../FIX-ROUND.md`).

Acceptance was uniform: every cell 14/14 core, 12/12 hard, 3/3 fix
addendum (`runs/NN-fix-acceptance.txt`), and all six fixes landed at
the XML parse layer with generic truncation detection (no sample- or
zero-items-shaped heuristics), so fix correctness carried no spread.
Across the 30 dimension cells **no gap reached the ≥2 reconciliation
threshold** (24 cells agree exactly; 6 differ by exactly 1), so no
reconciliation round was held and every consensus cell is the
two-grader mean (halves allowed). No factual disputes arose — both
graders cite identical acceptance results and runlog metrics (see
`adjudication.md`, fix-round section).

### Consensus scores

| dim (weight) | 01-atv-starterkit | 02-atv-phoenix | 04-kprojects | 05-baseline | 06-gstack | 07-baseline-claude |
|---|---:|---:|---:|---:|---:|---:|
| fix correctness (35) | 5 | 5 | 5 | 5 | 5 | 5 |
| fix quality (30) | 5 | 5 | 4.5 | 5 | 5 | 5 |
| tests (20) | 4 | 5 | 5 | 3.5 | 5 | 4.5 |
| scope & process (10) | 4.5 | 3.5 | 5 | 5 | 5 | 5 |
| efficiency (5) | 5 | 2.5 | 4 | 5 | 4 | 5 |
| **Weighted total** | **95** | **94.5** | **96** | **94** | **99** | **98** |

### Fix-round ranking

| rank | repo | harness | runner | total |
|---|---|---|---|---:|
| 1 | 06-gstack | gstack | Claude Code | 99 |
| 2 | 07-baseline-claude | none (control — Claude runner) | Claude Code | 98 |
| 3 | 04-kprojects | kprojects | Copilot CLI | 96 |
| 4 | 01-atv-starterkit | ATV-StarterKit 2.6.3 | Copilot CLI | 95 |
| 5 | 02-atv-phoenix | ATV-Phoenix | Copilot CLI | 94.5 |
| 6 | 05-baseline | none (control — Copilot runner) | Copilot CLI | 94 |

A 5-point spread — the tightest field of any round (run_02 build:
22.5). With correctness uniform, the spread is entirely test depth,
mechanism cleanliness, scope hygiene, and cost.

### Raw per-grader scores (pre-consensus, fable1/sol)

| dim | 01 | 02 | 04 | 05 | 06 | 07 |
|---|---|---|---|---|---|---|
| fix correctness | 5/5 | 5/5 | 5/5 | 5/5 | 5/5 | 5/5 |
| fix quality | 5/5 | 5/5 | 4/5 | 5/5 | 5/5 | 5/5 |
| tests | 4/4 | 5/5 | 5/5 | 3/4 | 5/5 | 4/5 |
| scope & process | 4/5 | 4/3 | 5/5 | 5/5 | 5/5 | 5/5 |
| efficiency | 5/5 | 3/2 | 4/4 | 5/5 | 4/4 | 5/5 |
| weighted | 94/96 | 96/93 | 93/99 | 92/96 | 99/100 | 96/100 |

### Reconciliation notes

- **Zero reconciliations again** — every gap is exactly 1 point, each
  an emphasis difference on agreed facts, absorbed by the mean:
  - **01 scope (4/5):** fable1 docked the repeated `.atv` dirty tree
    at done (only dirty tree of the round, same blemish as run_02);
    sol judged the fix commit itself, which both call exemplary.
  - **02 scope (4/3) and efficiency (3/2):** both graders flag the
    same two facts — harness-state edits bundled into the fix commit
    (`.phoenix-ralph/done-check.json` narrowed from `just check` to a
    single e2e test, `.phoenix/trace.jsonl` committed) and the field's
    slowest, costliest run (6m42s, 261.6 credits, 2.2× copilot-cell
    median) — sol weighs both harder.
  - **04 fix quality (4/5):** fable1 docked the mechanism (a mutable
    depth counter threaded through four match arms — the most invasive
    production change of the six, vs the others' one-place stack
    check); sol credited the field's only typed error
    (`ParseError::Truncated`) and the second EOF hole closed in
    `read_text`. Same facts, different weighting of thoroughness vs
    fragility.
  - **05 tests (3/4) and 07 tests (4/5):** both graders note the same
    gap — regression bodies that never stray from the bug report's own
    repro sample (05's single unit test verbatim; 07's unit + e2e both
    exact-repro-only, though multi-layer) — fable1 docks it one point
    harder in each case.
- **Rank shape diverged for the first time** (run_02's pre-consensus
  orders were identical): fable1 ordered 06 > 02=07 > 01 > 04 > 05,
  sol 07 > 04=06 > 01=05 > 02. In a 7-point-wide field, single-point
  emphasis differences reorder the middle; the consensus mean lands
  06 > 07 > 04 > 01 > 02 > 05. Both graders independently put 06 at or
  tied for the top of their sheets and both put a Claude cell first.
- **Shared findings, both sheets:** five of six fixes are literally the
  same fix (consulting the open-element stack the parser already
  maintained, in the `Event::Eof` arm); 04, whose parser had no stack,
  added a depth counter plus a second EOF guard in `read_text` and the
  only typed error variant. Tests are where the field spread: 02, 04,
  06, 07 wired a `truncated.xml` fixture + e2e asserting the
  user-visible failure path; 02 uniquely authored a well-formed-empty
  guard that independently anticipates the F2 addendum trap; 05's
  armor is one verbatim copy of the repro. On the copilot runner, cost
  tracked test depth almost linearly (67.4 → 86.2 → 151.7 → 261.6
  credits for 05 → 01 → 04 → 02).

### Standing context notes

- **03 absent-by-passing.** 03-working-skill-repo received no bug
  report: its run_02 build passed 26/26 because roxmltree's strict
  whole-document parsing made truncation an error by construction —
  the absence is itself a result. With the ungraded 99 shakedown rep
  also passing, 2 of 8 run_02 implementations chose strict parsing up
  front; the other six converged on the identical quick-xml
  Eof-leniency bug and all six then fixed it the same way.
- **E1 environment note.** Ambient MCP (klams/korg) was uniformly
  absent for all copilot fix cells (Copilot CLI MCP approval gate; full
  incident and corrections in `../FIX-ROUND.md`). The phoenix harness
  spine WAS present and verified for 02's graded rerun (the earlier
  spineless attempt is void on branch `void/fix-e1`, unread by
  graders). Claude cells' profile MCP was unaffected. The runner-level
  ambient-MCP asymmetry is recorded as an environment covariate, not
  scored.
