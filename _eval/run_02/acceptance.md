# Acceptance — run_02 (feedhub)

**SEALED. Never shown to working agents.** Unlike run 1's prose checklist,
run 02's acceptance is **executable**: one shared pytest suite
([acceptance/](acceptance/)) run identically against every repo. Graders
do not derive functional results themselves (lesson 10) — they receive
this suite's output. Boundary and edge semantics are pinned in the spec
itself (lesson 11), so there is nothing to interpret.

## Running it

```bash
FEEDHUB_REPO=~/src/ai-agents/harness-eval-runs/run_02/NN-<name> \
    uv run --with pytest pytest _eval/run_02/acceptance -v
```

Requires the Rust toolchain (the suite runs `cargo build --release` plus
the repo's own gates). Fully hermetic: fixture feeds are served by the
suite's own in-process HTTP server (with ETag/304 and a request log) —
the contender's `feedgen` is itself under test (C13), never trusted as
infrastructure.

## Tiers and scoring

**Core (C1–C13, `test_core.py`)** — the gate tier; every competent run
should pass all of it. Feeds the correctness dimension the way run 1's
12-check list did:

- C1 workspace builds; `feedd`/`feedctl`/`feedgen` binaries exist
- C2 health endpoint
- C3 feed registration (201 / duplicate 409 / invalid 422)
- C4 RSS 2.0 ingest: fields, UTC normalization, ordering, feed metadata
- C5 Atom ingest
- C6 refetch does not duplicate
- C7 entries query: feed filter, limit, total-ignores-limit
- C8 delete cascades, then 404
- C9 malformed feed: recorded error, no crash, siblings unaffected
- C10 feedctl add/refresh/list/entries; `--format json` is valid JSON
- C11 feedctl exit codes (1 API error / 2 unreachable)
- C12 repo's own gates: `cargo test`, `fmt --check`, `clippy -D warnings`
- C13 feedgen serves with ETag and honors If-None-Match with 304

**Hard (H1–H12, `test_hard.py`)** — the spread tier (lessons 9, 21).
Failures expected; report the pass count, not pass/fail:

- H1 RFC 822 named zones (EST/GMT) → correct UTC instants
- H2 missing date → null, stored, sorted last
- H3 garbage date → null, never fetch-time
- H4 RFC 3339 offsets + fractional seconds → UTC
- H5 Atom `updated` fallback
- H6 half-open window incl. an offset-form bound (the run 1 dispute class,
  now pinned and mechanical)
- H7 BOM + CDATA verbatim + entity unescaping
- H8 guid reuse → update in place (stable id, stable fetched_at)
- H9 conditional GET: If-None-Match actually sent, 304 handled
- H10 pagination math + cross-page ordering
- H11 case-insensitive title search
- H12 refresh-all failure isolation with per-feed results

## Reporting

Per repo: `core: N/14` pytest tests (13 checks — C12 expands to three
gate tests; C1 is enforced by the build fixture), `hard: N/12`, plus the
pytest output archived to
`_eval/run_02/runs/NN-acceptance.txt`. Core feeds the correctness
dimension; hard is reported alongside it and drives the robustness
narrative.

## Status

Written during run 02 prep; **not yet validated against a real
implementation** — the planned dry-run control run doubles as the suite's
shakedown. Expected suite-bug fixes before the field runs are process
fixes, not spec changes; log them in the run_02 README. The suite freezes
when the first real contender runs.
