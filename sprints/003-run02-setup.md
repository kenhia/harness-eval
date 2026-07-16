# 003 — run 02 setup (feedhub)

*Started 2026-07-16. In progress — this record tracks setup through field
execution.*

## Goal

Stand up everything run 02 needs short of pressing go: spec, prompts,
executable acceptance, rubric, staging repos.

## Decisions (Ken, 2026-07-16)

- **Rust** (Cargo workspace; Go was the fallback) — stack change tests
  harness generality, compiler pressure should spread the field.
- **SQLite pinned** in the spec — removes a comparison-muddying degree of
  freedom.
- **All 7 cells** — full comparability with run 1.
- **Headless** via `run-eval.sh --headless` — reproducible; recorded as a
  covariate vs run 1's interactive mode.

## Shipped so far

- **Spec** (`_eval/run_02/prompts/00-project-spec.md`): feedhub — `feedd`
  (REST + SQLite + RSS/Atom ingest + conditional GET), `feedctl` (API
  client CLI), `feedgen` (fixture server, makes tests hermetic). Every
  edge semantic that burned run 1 is pinned: half-open windows, UTC
  normalization incl. RFC 822 zone names, null-date handling, dedupe/
  update-in-place, ordering + tie-breaks, entity/CDATA handling (lesson
  11: specs pin their own edges).
- **Prompts**: prefixes.txt + 7 generated prompt files, go-lines carried
  over from run 1 verbatim (only "loglens CLI" → "feedhub service").
- **Executable sealed acceptance** (`_eval/run_02/acceptance/`): hermetic
  pytest, 26 tests — core C1–C13 gate tier (build, API contract, ingest,
  dedupe, failure isolation, feedctl exit codes, repo's own gates,
  feedgen ETag/304) + hard H1–H12 spread tier (zone names, null dates,
  offsets, half-open boundaries incl. offset-form bounds, BOM/CDATA/
  entities, update-in-place, conditional GET actually observed via the
  fixture server's request log, pagination math, search, refresh-all
  isolation). Suite serves its OWN fixture corpus — the contender's
  feedgen is under test, never trusted infrastructure.
- **Rubric** (`_eval/run_02/rubric.md`): run 1 weights kept; correctness
  fed by the suite (core gates it, hard tops it out); efficiency banded
  against the same-runner control (lesson 13).
- **Staging repos**: all 7 created under `harness-eval-runs/run_02/`
  (controls pre-run-tagged).
- **Tooling fix**: rustup/cargo break under a bare fake HOME —
  `run-eval.sh` now passes `CARGO_HOME`/`RUSTUP_HOME` through to the real
  installs (verified).

## Remaining

1. Harness install refresh (5 repos) + profile re-verification.
2. Dry-run shakedown: throwaway control run + acceptance suite against
   its output (suite is untested against a real implementation).
3. Freeze at first contender run; execute field; grade.
