# Roadmap

> The general plan for harness-eval. Detail lives in the sprint records;
> the eval's own design docs are `_eval/README.md` and
> `_eval/ADDING-A-HARNESS.md`. Lessons driving all of this:
> `_eval/run_01/report/lessons-learned.md`.

## Now — run 02 (complex greenfield: feedhub)

Run 02 is the "do the heavy harnesses pull their weight?" experiment.
Decisions locked 2026-07-16: **Rust** workspace (feedd/feedctl/feedgen),
**SQLite pinned**, **all 7 cells**, **headless** (covariate vs run 1).

Done in setup (sprint 003): spec with pinned edge semantics
(`_eval/run_02/prompts/00-project-spec.md`), 7 prompts, executable sealed
acceptance suite (core C1–C13 + hard H1–H12, hermetic pytest), rubric,
staging repos (controls pre-run-tagged), cargo/rustup fake-HOME
passthrough in run-eval.sh.

Remaining (ordered — details in `_eval/run_02/README.md`):

1. **Harness install refresh** into the 5 harness staging repos +
   profile re-verification (Phoenix + gstack global pieces).
2. **Dry-run shakedown** — throwaway control run, acceptance suite run
   against its output (suite hasn't met a real implementation yet);
   calibrates Rust-task cost/wall-clock.
3. **Freeze** spec + suite at first real contender run; then execute the
   field headless, one cell at a time.

## Next

- **Execute run 02** — same 7 cells as run 1 (5 harnesses + 2 controls,
  native runner each), fresh staging under
  `harness-eval-runs/run_02/`, delta-style grading with a new rubric
  calibration; efficiency anchored to each runner's control (lesson 13).
- **Publish run 02** — import to `run-output/run_02/`, whitepaper v2
  section, README results table gains a per-run section.
- **Task-type matrix, one cell per run group** (v2 design, lessons §v2):
  - run 03: **bug-fix** on a planted-bug repo;
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
