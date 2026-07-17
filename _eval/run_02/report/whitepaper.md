# One dependency to rule them all — Harness eval run 02 + fix round 02.1

*harness-eval, 2026-07-17. Seven identical Rust builds of a
three-binary feed-aggregation service: five harnesses, two bare-model
controls, two runners, same model, one frozen prompt — then a second
act in which six of the seven received a bug report against their own
repo and fixed their own code. Executable sealed acceptance, two
independent LLM graders, consensus. **Preliminary: N=1 per cell** — see
Variance.*

## The question

Run 1 asked whether harnesses earn their keep on a small greenfield CLI
and couldn't separate the field (7 of 7 passed everything; the spread
lived in style). Run 02 raises the stakes twice:

1. **Build**: a genuinely multi-component task — `feedd` (REST server,
   SQLite, RSS 2.0 + Atom ingest, conditional GET), `feedctl` (API
   client CLI), `feedgen` (hermetic fixture server) in a Cargo
   workspace, with every edge semantic pinned in the spec — does
   heavier machinery pull its weight when there's real planning to do?
2. **Fix (02.1)**: the build round left six implementations with the
   *same organic bug*. Each got a realistic bug report and fixed its
   own codebase — the first cell in this eval where an agent resumes
   its own work. Does harness machinery (sprints, handoffs, self-verify
   spines) show up when the task is debugging rather than greenfield?

## Method

**Task.** The feedhub spec pins its own edge semantics — half-open
`[since, until)` windows, UTC normalization incl. RFC 822 zone names,
null for unparseable dates, dedupe/update-in-place, ordering
tie-breaks, entity/CDATA handling — because run 1's only grading
dispute came from a spec silence (lesson 11). Rust + pinned SQLite by
design: a stack change defeats boilerplate reflexes and the compiler
forces real iterate-fix loops.

**Acceptance is executable and sealed.** One shared hermetic pytest
suite (26 tests) run identically against every repo: a **core tier**
(C1–C13: build, API contract, ingest, dedupe, failure isolation, exit
codes, the repo's own gates) and a **hard tier** (H1–H12: RFC 822 zone
names, null-date ordering, offset normalization, boundary probes at
exactly `since`/`until`, BOM/CDATA/entities, update-in-place identity,
conditional GET observed via the fixture server's request log,
pagination math, search, refresh-all isolation). The suite serves its
own fixture corpus; the contender's `feedgen` is itself under test,
never trusted infrastructure. Graders receive the results and score
only subjective dimensions. The fix round adds a sealed addendum
(F1–F3) gated behind `FIX_ROUND=1`, including the F2 trap: a
well-formed feed with zero items must still refresh "ok" — designed to
catch "no items = error" symptom patches.

**Contenders and conditions.** Same seven cells as run 1 (StarterKit
2.6.3, Phoenix, working-skill-repo/KB, kprojects, gstack, plus a
control per runner), same per-harness "go" lines. New this run: fully
**headless** execution (`claude -p` / `copilot -p`, recorded as a
covariate vs run 1's interactive mode), fake-HOME profiles for both
runners, and mechanical capture — `run-eval.sh` preflights the
environment, times the run, and auto-fills run logs from session logs;
`run-acceptance.sh` archives suite output per repo. Zero interventions
in all thirteen runs (7 build + 6 fix).

**Fix round design.** Boundary tag `pre-fix` at each post-build HEAD;
each agent got the same **bug report** (reproduction with a *different*
truncation than the sealed fixture, observed vs expected, spec quote —
deliberately NOT the sealed tests, and deliberately not asking for a
regression test, so adding one is signal). 03 received nothing: its
build passed 26/26, so there was nothing to fix — the absence is a
result, not a gap (see The dependency finding).

**Grading.** Two independent graders (fable1, GPT Sol), calibrated
against their own run-1-era sheets, opposite grading orders; consensus
session reconciles ≥2-point gaps and adjudicates factual disputes in
fresh clones. Build round: run-1 rubric weights. Fix round: fix-delta
rubric (correctness 35 / fix quality 30 / tests 20 / scope 10 /
efficiency 5) on `pre-fix..HEAD` only.

## Build results (run_02)

| rank | repo | harness | runner | total | core | hard | wall | cost† |
|---|---|---|---|---:|---|---|---|---|
| 1 | 03 | working-skill-repo (KB) | Copilot | **93.5** | 14/14 | 12/12 | 18m 01s | 647.7 cr |
| 2 | 07 | none (control) | Claude Code | 84.5 | 13/14 | 11/12 | 36m 09s | 144.9k out-tok |
| 3 | 06 | gstack | Claude Code | 82.5 | 13/14 | 11/12 | 48m 11s | 284.3k out-tok |
| 4 | 05 | none (control) | Copilot | 77.5 | 13/14 | 11/12 | 19m 35s | 703.0 cr |
| 5 | 04 | kprojects | Copilot | 75 | 13/14 | 11/12 | 20m 38s | 735.8 cr |
| 6 | 02 | ATV-Phoenix | Copilot | 73.5 | 13/14 | 11/12 | 17m 16s | 540.3 cr |
| 7 | 01 | ATV-StarterKit 2.6.3 | Copilot | 71 | 13/14 | 11/12 | 16m 22s | 596.7 cr |

† Cost units are per-runner and NOT comparable to each other or to run
1 (see Threats: the credit unit itself drifted ~5× between CLI
versions; premium requests — 15 for every Copilot cell, build and fix
alike — are the stabler unit).

The 22.5-point spread (run 1: 4.5 among the same seven) is real
discrimination, and it is dominated by one thing: **six of seven repos
share a single root-cause correctness failure** (acceptance C9 + H12).
Every hand-rolled quick-xml event loop treats `Event::Eof` as normal
termination, so truncated XML parses as a valid empty feed and refresh
reports success instead of recording `last_error`. Each of those six
also authored its own `malformed.xml` test fixture — in the
mismatched-tag flavor its parser *does* catch — so six test suites
certified the failing behavior. The hard tier's other eleven probes
(zone names, boundaries, BOM/CDATA, conditional GET, pagination…) were
passed by everyone: at frontier capability, pinned-spec semantics get
implemented correctly; what discriminates is what nobody thought to
doubt.

**The runner effect flipped.** In run 1 the bare Claude Code control
topped the whole field; here both Claude cells again beat every Copilot
cell *except* KB's 03 (93.5 > 84.5). A harness cell out-scoring the
other runner's control is a first for this eval. Also unlike run 1,
gstack (06) landed *above* its own baseline in run 1 terms — but still
2 points under the bare control 07 at 2× output tokens and +12 minutes:
its robustness-margin spending habits persist on a bigger task.

**Efficiency didn't discriminate within the Copilot sub-field** (all
five cells 0.77–1.05× of control, identical premium requests) — and
docs didn't discriminate anywhere (uniform 5s: complete READMEs are
table stakes for this model generation).

## Fix results (run_02.1)

| rank | repo | harness | total | wall | cost |
|---|---|---|---:|---|---|
| 1 | 06 | gstack | **99** | 3m 44s | 12.8k out-tok |
| 2 | 07 | none (Claude control) | 98 | 2m 57s | 8.5k out-tok |
| 3 | 04 | kprojects | 96 | 3m 43s | 151.7 cr |
| 4 | 01 | ATV-StarterKit | 95 | 1m 53s | 86.2 cr |
| 5 | 02 | ATV-Phoenix | 94.5 | 6m 42s | 261.6 cr |
| 6 | 05 | none (Copilot control) | 94 | 1m 29s | 67.4 cr |

Acceptance was uniform — every cell 14/14 core, 12/12 hard, 3/3 fix
addendum. Nobody special-cased the bug report's sample; nobody fell
into the F2 empty-valid-feed trap (a pre-registered prediction,
falsified). **Five of six produced literally the same fix** — consult
the open-element stack the parser already maintained in the
`Event::Eof` arm; 04, whose parser had no stack, added a depth counter
plus a second EOF guard and the field's only typed error
(`ParseError::Truncated`).

With correctness flat, the 5-point spread is test depth, scope
hygiene, and cost:

- **Tests separated the field.** 02, 04, 06, 07 wired a `truncated.xml`
  fixture plus an end-to-end test asserting the user-visible failure
  path; 02 uniquely added a well-formed-empty-feed guard that
  independently anticipates the sealed F2 trap; 05's entire armor is
  one unit test copying the bug report's repro verbatim. The bug
  report never asked for tests — this dimension is pure unprompted
  discipline, and the two harness-free controls sit at the bottom of
  it (05: 3.5, and even 07's multi-layer tests never left the repro
  sample).
- **Cost tracked test depth almost linearly** on the Copilot runner:
  67.4 → 86.2 → 151.7 → 261.6 credits for 05 → 01 → 04 → 02. You buy
  fix thoroughness with tokens — the fix round's version of run 1's
  robustness-for-tokens trade, but this time the spending bought
  scoreable artifacts.
- **Harness residue is the main scope blemish**: Phoenix's fix commit
  bundled harness-state edits (a narrowed done-check, a committed
  trace file) and ran slowest at 2.2× the Copilot-cell median; 01
  repeated its dirty-tree-at-done habit (`.atv` runtime state, the
  only dirty tree of the round).

**Read as a pair, the two rounds invert.** Greenfield build: the two
Claude cells and KB dominate, heavier Copilot harnesses trail their own
control. Fix-your-own-bug: *every* harness cell beats its same-runner
control — machinery that was ceremony on a half-hour build (sprint
records, done-checks, self-verification spines) turns into regression
tests and verification passes when the task is surgical. The effect is
small (5 points, N=1) but its *direction* is consistent across all
four harness/control pairs, and it is the first empirical support in
this eval for the hypothesis the harnesses exist to serve.

## The dependency finding

The single most consequential decision in thirteen runs was made in
the first minute of each build, before any test existed: **which XML
crate to type into Cargo.toml.** Two of eight implementations (03, and
the ungraded 99 shakedown rep) chose strict whole-document parsing
(roxmltree) and were immune to the truncation bug *by construction*.
The other six chose quick-xml — a streaming parser whose `Eof` event
arrives as a normal value, not an error — hand-rolled the same event
loop, wrote the same inadequate fixture, and shipped the same bug.
No harness influenced the choice in either direction; no agent
documented *why* it picked its parser; and six independent frontier
trajectories converged on the identical trap. The eval's sharpest
discriminator wasn't reasoning depth or process discipline — it was a
dependency default. (03's win is still a win: its KB machinery also
produced the field's cleanest workspace split per both graders. But
the 22.5-point gap to last place is mostly one line of Cargo.toml.)

## What the executable acceptance bought

Run 1's grading needed an adjudication and a reconciliation round;
its only acceptance dispute came from two graders' sealed fixtures
probing an unpinned boundary differently, and run 1.5 re-litigated
that exact question. Run 02: **zero reconciliations in both rounds**
(49 build cells: max gap 1 point; 30 fix cells: same), identical
pre-consensus rank order in the build round, correctness mechanical
everywhere. One suite defect (S1, a case-sensitive header lookup)
was caught by the first *live* implementation, fixed under the
documented adjudication path, and every completed repo re-run. The
sealed-fixture dispute class is simply gone. Grader disagreement
that remains is a stable one-point severity prior (Sol docks parser
precision shortcuts harder), absorbed by the mean without moving a
rank in the build round.

## Variance — why these results are preliminary

The control cell effectively ran twice at build (the 99 shakedown and
cell 07, identical frozen prompt): 52m/206.9k output tokens vs
36m/144.9k — **44% wall-clock spread at identical ~4k tok/min
throughput**, and the shakedown rep passed the very C9 check that 07
(and five others) failed. Forensics ruled out leakage (separate
project slugs, no memory, fresh session, zero cross-references): this
is pure trajectory variance, and it brackets everything above. The
decisive correctness behavior of this field sits *inside* single-run
variance; the build ranking's broad strokes (03's clean pass, the
six-way shared bug, the fix round's direction) are the findings we
trust, the point-level ordering is not. N ≥ 3 reps per cell is the
top roadmap item; at ~0.5–1.5h and real dollars per cell it
accumulates over weeks. We publish N=1 with the caveat prominent
rather than sit on the data.

## Threats to validity

- **N=1 per cell**, single task type per round, single model.
  Variance is measured, not hypothetical (above).
- **Cost units drifted mid-eval**: Copilot's credit unit changed ~5×
  between run 1 and run 02 CLI versions (same cell, same premium
  requests); credits are not comparable across runs, and Claude
  dollar/token units never were comparable to credits. Premium
  requests and wall clock travel best.
- **Runner versions drifted mid-field** (Claude Code 2.1.209 → 2.1.211
  between cells 06 and 07; Copilot 1.0.70 → 1.0.71 before the field,
  1.0.71 throughout it). Recorded per run log; not controllable with
  auto-updating CLIs.
- **The E1 incident** (fix round): Copilot CLI's MCP approval gate
  activated mid-round, initially misread as an org policy. Net
  handling: ambient MCP (klams/korg) uniformly absent for all Copilot
  fix cells; Phoenix's own MCP spine re-approved and verified present
  for its graded rerun (a first spineless attempt was voided unread);
  Claude cells' profile MCP unaffected. Runner-level ambient-MCP
  asymmetry stands as a covariate. Full forensics in FIX-ROUND.md.
- **Headless vs run 1's interactive** mode is a covariate when
  comparing across runs.
- **Author conflicts**: the eval author authored one contender
  (kprojects); harness artifacts self-identify, so grader blinding is
  imperfect. Both unchanged from run 1.
- **StarterKit/gstack shared DNA weakened**: 2.6.3 no longer vendors
  gstack (3 textual mentions only).

## Per-harness verdicts (one line each, build + fix)

- **working-skill-repo (KB)**: the result of the run — only full pass,
  cleanest workspace, then sat out the fix round with nothing to fix.
- **gstack**: still spends big (2× tokens, +33% wall vs control) but
  bought real quality this time (best tests of the build round; won
  the fix round outright).
- **kprojects**: mid-field build, strong disciplined fix (best Copilot
  fix cell); its sprint machinery finally paid on the resume task.
- **ATV-Phoenix**: build indistinguishable from control at extra
  process weight; solid fix undermined by harness-state leakage into
  the fix commit and 2.2× median cost.
- **ATV-StarterKit**: last on build (weakest tests of the field,
  dirty-tree habit), yet a crisp minimal fix — its machinery neither
  hurt nor helped a two-minute surgical task.
- **Controls**: the Copilot control is the build sub-field's
  second-best value and the fix round's floor (verbatim-repro test
  only) — bare models fix bugs fine but don't armor them; the Claude
  control remains the strongest bare configuration.

## What's next

N≥3 reps (headless matrix driver), the behavior-preserving refactor
and resume-from-handoff cells, and a harness × runner axis for the
dual-target harnesses. Process debts paid this run that v3 keeps:
executable sealed acceptance with a hard tier, precedents that travel,
mechanical run logs, per-run environment manifests (E1's lesson:
ambient services can change under you mid-field — capture them, per
run, mechanically).
