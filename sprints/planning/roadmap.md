# Roadmap

> The general plan for harness-eval. Detail lives in the sprint records;
> the eval's own design docs are `_eval/README.md` and
> `_eval/ADDING-A-HARNESS.md`. Lessons driving all of this:
> `_eval/run_01/report/lessons-learned.md`.

## Now — after run 02: reps and the combined report is shipped

Run 02 + fix round 02.1 are COMPLETE and published (2026-07-17): spec,
executable acceptance, 13 headless runs, zero-reconciliation grading,
combined whitepaper/infographic/lessons, publish import. Records:
sprint 004, `_eval/run_02/`, `run-output/run_02/`.

Next up (pick per budget):

1. **Reps toward N≥3** on the run_02 field — the top-value spend
   (variance is measured at ±44% wall / rank-affecting; see lessons
   29). Needs the headless matrix driver (loop run-eval.sh over
   prefixes.txt), a small tooling sprint.
2. **v3 preflight hardening**: per-run MCP/tool availability manifest
   (lesson 30), void protocol codified in ADDING-A-HARNESS (lesson 33).
3. **Run 03 design**: behavior-preserving refactor or
   resume-from-handoff cell (lesson 27 says press the resume axis —
   that's where machinery paid); dependency-decision task idea from
   lesson 25.

## Next

- **Execute run 02** — same 7 cells as run 1 (5 harnesses + 2 controls,
  native runner each), fresh staging under
  `harness-eval-runs/run_02/`, delta-style grading with a new rubric
  calibration; efficiency anchored to each runner's control (lesson 13).
- **Publish run 02 + 02.1 as ONE combined report** (decided 2026-07-16):
  the story is the field's shared quick-xml failure paired with how each
  implementation fixed its own bug. Whitepaper, infographic, and README
  results wait for 02.1 grading; the infographic gets an explicit blurb
  that 03 sits out the fix round *because it passed* (strict roxmltree —
  the same choice as the ungraded 99 shakedown control, i.e. 2 of 8
  implementations dodged the trap). Import to `run-output/run_02/`
  happens after 02.1 so final trees include the fix commits.
- **run_02.1 — organic bug-fix round** (after run_02 grading is locked):
  every non-control implementation failed C9/H12 identically (quick-xml
  streaming leniency: malformed XML → "ok" empty feed). Give each agent
  a **bug report** against its own repo (fixture + observed vs expected +
  spec quote — NOT the sealed pytest source, which must never enter the
  published repos) and have it fix its own codebase. Mechanics: tag
  `pre-fix`, same prompt template for all, grade the `pre-fix..HEAD`
  delta (root-cause vs symptom patch, regression test, scope
  discipline), re-run the full sealed suite for the flip + no
  regressions. This is the task-matrix bug-fix + resume-own-work cell
  arriving organically — handoff machinery (sprints, KB handoffs, gstack
  context) finally gets tested against bare controls. Cheap (~7 short
  runs). Optional bonus probe: hand 99 (which passes C9) the same bug
  report and observe non-reproducing-bug behavior.
- **Task-type matrix, one cell per run group** (v2 design, lessons §v2):
  - run 03: **bug-fix** on a planted-bug repo — largely superseded by
    run_02.1 if it lands; keep only if a planted, multi-file bug adds
    signal the organic one didn't;
  - run 04: **behavior-preserving refactor** on a messy repo;
  - run 05: **resume-from-handoff** — the cell where handoff-heavy
    harnesses (KB, kprojects) should shine or be exposed. Candidate
    input: a deliberately half-finished run 02 repo.

## Later / Ideas

- **N ≥ 3 reps per cell**; medians + spread, not single runs. Variance
  is now measured, not hypothetical: the run 02 control cell ran twice
  (99-shakedown vs 07) at 52m/207k vs 36m/145k output tokens — 44%
  wall-clock spread, identical throughput, leakage forensically ruled
  out. Personal-project budget means ≈21+ runs will take **weeks**:
  publish run 02 as preliminary N=1 (caveat stated prominently), then
  accumulate reps as time/money allow and refine. Needs the headless
  matrix driver (a justfile/script looping run-eval.sh `--headless`) —
  mechanics exist, orchestration doesn't.
- **Harness × runner axis** where dual-target harnesses exist (kprojects,
  possibly others) — lesson 15 showed the runner effect is as large as
  the harness effect.
- Third grader identity, or CI-run acceptance so graders never touch
  objective checks at all.
- Model axis (same harness, different model) once per-cell costs are
  understood.
- Automated secret-scan step folded into the publish-import tooling.

---

## Run 02 design sketch (decide before spec-writing)

**Shape**: a workspace producing **three cooperating executables** —
proposed: a small **feed aggregation service**:

- `feedd` — long-running server: polls configured RSS 2.0 + Atom feeds on
  an interval, stores normalized entries (SQLite or flat files), exposes a
  **REST API** (list/search entries, feed CRUD, health/stats).
- `feedctl` — CLI client that talks to `feedd`'s REST API (add/remove
  feeds, query entries, tail new items).
- `feedgen` — fixture/load tool: serves synthetic feeds over local HTTP
  (valid, subtly-invalid, and adversarial), enabling hermetic acceptance
  tests with no external network.
- shared library crate/module for feed parsing + the API types.

Why this project: it forces **web research** (RSS 2.0 vs Atom RFC 4287
differences, RFC 3339/822 date handling, conditional GET with
ETag/If-Modified-Since), **real planning** (three binaries + shared lib +
storage + API contract), and it's rich in hard-tier acceptance material
(encoding, malformed XML, tz-naive dates — the exact edge class that
discriminated in run 1). Everything tests hermetically via `feedgen`.

**Stack — recommendation: Rust** (Cargo workspace, e.g. axum + reqwest +
clap). Rationale: all run-1 output was Python, so a stack change tests
harness generality and defeats boilerplate reflexes; the compiler forces
real iterate-fix loops, which should spread the efficiency and robustness
dimensions; a Cargo workspace models the multi-binary shape naturally.
Cost: longer wall clocks — expect run 1's per-run cost to grow 2–4×,
times 7 cells. **Fallback: Go** if we want the complexity to live
entirely in the architecture rather than partly in the language. Python
is the do-not: it would measure run 1 again, just bigger.

**Open decisions** (Ken):
- Stack: Rust (recommended) vs Go.
- Storage: pin it in the spec (SQLite recommended — pins acceptance
  queries) vs leave storage as a design decision to score.
- Contender set: all 5 harnesses again, or drop any that run 1 showed to
  be pure overhead?
- Interactive (paste + hands-off, as run 1) vs `--headless` for the real
  runs — headless is more reproducible; interactive matches run 1's
  conditions. (Suggest: headless, and note it as a covariate.)
