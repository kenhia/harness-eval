# Roadmap

> The general plan for harness-eval. Detail lives in the sprint records;
> the eval's own design docs are `_eval/README.md` and
> `_eval/ADDING-A-HARNESS.md`. Lessons driving all of this:
> `_eval/run_01/report/lessons-learned.md`.

## Now — prepare run 02 (complex greenfield)

Run 02 is the "do the heavy harnesses pull their weight?" experiment: a
bigger, multi-component project that genuinely needs planning and web
research, where run 1's task was too small to reward machinery.

1. **Author the run 02 spec** (`_eval/run_02/prompts/00-project-spec.md`)
   — see design sketch below. Spec must pin its own edge semantics
   (lesson 11) and stay frozen once the first contender runs.
2. **Executable sealed acceptance suite** (`_eval/run_02/acceptance/`) —
   the single highest-value v2 change (lessons 9, 10, 21): one shared
   pytest suite run identically against every repo, black-box (CLI + REST
   against the built binaries), two tiers:
   - **core** — every competent run should pass (parity with run 1's
     checklist role);
   - **hard** — adversarial/edge inputs designed to spread the field
     (malformed feeds, encoding traps, timezone/date pathologies,
     boundary semantics *as pinned by the spec*).
   Graders then score only subjective dimensions.
3. **Refresh contenders + profiles** — re-verify each harness's install
   under current versions; preflight per §2 of ADDING-A-HARNESS; harness
   versions recorded in run logs (auto-captured runner version already is).
4. **Dry-run the mechanics** — one throwaway control run end-to-end with
   `run-eval.sh` on the new spec before burning the real field.

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

- **N ≥ 3 reps per cell**; medians + spread, not single runs. Needs the
  headless matrix driver (a justfile/script looping run-eval.sh
  `--headless`) — mechanics exist, orchestration doesn't.
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
