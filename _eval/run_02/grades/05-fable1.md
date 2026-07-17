# 05-baseline — grader: fable1

Control run: no harness, Copilot CLI runner. This cell is the efficiency
anchor for the Copilot cells (01–04); its own efficiency is scored 5 by
definition of the band ("control-comparable = 5").

| dim | score /5 | note |
|---|---|---|
| correctness | 3 | Mapping applied: core 13/14 → core-failure cap ≤3 → 3 (hard 11/12 noted; cannot lift past the cap). Failures C9+H12, one root cause: `Ok(Event::Eof) => break` (parse.rs:113) with only a format-detection post-check (parse.rs:181), so between-tags truncation parses as "ok, 0 entries" and `record_success` clears last_error. Verified empirically; mismatched tags, mid-tag truncation, and non-XML garbage all correctly error |
| code quality | 4 | No panics on IO or request paths; per-feed error recording covers even the error path's own errors (fetch.rs:74-83); doc comments on every module; FNV fallback identity for guid-less items; title COALESCE keeps prior title on update. Docked: a hand-rolled percent-decoder (api.rs:246-277) despite the `url` crate being a declared dependency, and the FNV hash duplicated verbatim in feedlib and feedgen — reinvention and copy-paste where reuse was available |
| tests | 4 | 22 tests: store units pin the important semantics directly — `upsert_dedupes_and_updates` (asserts fetched_at preserved), `ordering_nulls_last_then_id`, `window_half_open_excludes_null`, `search_case_insensitive` — plus a real e2e spawning the feedd binary via CARGO_BIN_EXE against an in-process feedgen server, covering add/409/422, refresh, 304, malformed→last_error, refresh-all, cascade delete. Docked: feedctl (346 lines) has zero tests incl. its spec'd exit codes, and limit clamp/pagination are untested |
| docs | 5 | README (297 lines) alone suffices: crate table, build, quickstart, all 8 endpoints with status codes and JSON shapes, entries param table with half-open/nulls-last/clamp semantics, feedctl commands + exit codes, feedgen corpus table, `just check` with plain-cargo fallback, storage schema |
| process | 5 | Six commits, one per component in a sensible sequence (lib → feedgen → feedd → feedctl → e2e/lints → docs), subjects accurate; no ceremony (control cell — none expected, none produced). The e2e commit's "fix fmt and clippy lints" records a real gate loop |
| efficiency | 5 | Anchor cell: 703.0 credits, 19m35s, 15 premium requests, 85.5k output tokens — the reference the other Copilot cells are banded against |
| autonomy | 5 | Zero interventions; declared done; working tree clean |

**Weighted total:** 81/100
(3/5×30 + 4/5×20 + 4/5×15 + 5/5×10 + 5/5×10 + 5/5×10 + 5/5×5)

**Best thing:** Discipline of the error taxonomy: zero panics on any server/IO path, and `refresh_feed` records per-feed errors even when its own error-recording path fails — the failure-isolation contract is honored end-to-end (except for the one parser leniency the whole field shared).

**Worst thing:** The parser accepts truncated XML via the silent `Eof` break (parse.rs:113) — and the repo's two malformed fixtures (unit and feedgen corpus) both use the mismatched-tag flavor quick-xml catches, so its own passing tests certified the exact behavior the acceptance suite failed.

**Narrative (≤150 words):** The Copilot control again sets a demanding reference: with no harness at all it produced a complete, well-documented feedhub with clean layering, store-level unit tests that pin dedupe, ordering, window, and search semantics directly, and a genuine binary-spawning end-to-end test. Its acceptance failure is the field's shared one — quick-xml's silence at EOF turned truncated feeds into successful empty fetches, and the repo's own malformed fixtures happened to exercise only the flavor the parser catches. Beyond that, the artifact's warts are small but telling: feedctl ships 346 lines with zero tests, a percent-decoder was hand-rolled while the `url` crate sat in the dependency list, and an FNV hash is copy-pasted across two crates. Six tidy commits, zero interventions, clean tree. Four of five harness cells failed to beat this on acceptance, and none beat it decisively on cost.
