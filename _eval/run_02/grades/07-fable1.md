# 07-baseline-claude — grader: fable1

Control run: no harness, Claude Code runner; the efficiency anchor for
the Claude cells. Note for the variance record: the identical frozen
prompt produced 26/26 in the 99-shakedown rep and 24/26 here — the C9
truncation behavior itself sits inside single-run trajectory variance.

| dim | score /5 | note |
|---|---|---|
| correctness | 3 | Mapping applied: core 13/14 → core-failure cap ≤3 → 3 (hard 11/12 noted; cannot lift past the cap). Failures C9+H12, one root cause: `parse_feed` errors only when quick-xml itself errors, and `Ok(Event::Eof) => break` (parse.rs:204) plus a format-only post-check (parse.rs:267-269) lets truncated documents parse as empty feeds; `apply_fetch` then sets last_error = NULL. The error-recording plumbing itself (refresh.rs:64-90) is correct. Verified empirically |
| code quality | 5 | No dead code or speculative abstraction; comments consistently explain why; `TextAccum` preserves CDATA verbatim while trimming plain text via segment tagging (parse.rs:88-125); unknown entities like `&nbsp;` kept verbatim per spec's unescaping pin; search via `instr(lower(..))` deliberately avoiding LIKE-wildcard escaping. The self-review commit d06e2a3 fixed five real defects: unbounded body read → 16 MiB streaming cap, parse moved outside the DB mutex, poll ticker Burst→Delay, a genuine window-bound rounding bug (floor→ceil on fractional `until`, with a written proof at dates.rs:20-31), feedctl timeout. Nits, noted not docked: `updated_entries` counts re-seen-unchanged rows; feedctl's client builder swallows failure with `unwrap_or_default()` |
| tests | 5 | 58 tests hitting every rubric edge: full exit-code matrix with the real feedctl binary spawned via CARGO_BIN_EXE; 13 zone/offset date variants driven end-to-end; dedupe update-in-place; failure isolation incl. an oversized-feed refusal; conditional GET both sides; nulls-last + id-tiebreak ordering; half-open offset-aware window incl. sub-second rounding; limit clamp asserted at 100000→500; nothing tautological. E2e runs real feedd against real feedgen over local HTTP on port 0 |
| docs | 5 | README (342 lines) alone suffices: build, checks (`just check` + spelled-out non-just equivalents), a two-terminal no-internet walkthrough with expected output, per-binary sections with exit codes, all 8 endpoints as individual headings plus object schemas, layout, storage |
| process | 5 | Six commits — one per component, then docs, then a genuine self-review commit (d06e2a3) that found five real defects including a subtle correctness bug, with design-rationale bodies throughout. The run-1 pattern of an unprompted review loop that changes the outcome repeats on this runner with nothing prescribing it |
| efficiency | 5 | Anchor cell: 36m09s, 144.9k output tokens, single linear pass, review loop included. Sets the bar the Claude-runner harness cell is banded against |
| autonomy | 5 | Zero interventions; declared done; working tree clean |

**Weighted total:** 88/100
(3/5×30 + 5/5×20 + 5/5×15 + 5/5×10 + 5/5×10 + 5/5×10 + 5/5×5)

**Best thing:** The self-review commit (d06e2a3): five real fixes including the sub-second window-bound rounding bug — `until=12:00:00.5` silently dropping an entry at 12:00:00 — with a correct written proof in dates.rs and a regression test in the e2e; the exact class of unprompted review-that-changes-the-outcome this cell showed in run 1.

**Worst thing:** The hand-rolled parser's malformed-XML guarantee holds only for the malformation classes quick-xml reports: truncated documents parse as empty feeds (parse.rs:204,267-269), and the repo's own malformed fixture — a mismatched close tag — is the one class its parser does catch, so 58 otherwise-sharp tests certified the failing behavior.

**Narrative (≤150 words):** The Claude control repeats its run-1 shape: a linear six-commit build ending in an unprompted self-review that caught five real defects, one of them a subtle correctness bug most suites would never probe. Tests are the second-strongest of the field and the most efficient per line — 58 tests covering the full exit-code matrix (real binary), thirteen date edge cases, clamp, tie-break, and sub-second window rounding. Code is clean throughout, with the field's most careful text handling (CDATA-verbatim via segment-tagged accumulation). Its single blemish is the field's: quick-xml's silence at EOF let truncated feeds count as successful empty fetches, certified by a fixture that only exercised the catchable malformation flavor. At 36 minutes and 144.9k output tokens with zero thrash, this is again the bar: on this task the harness cell above it bought a better test suite, not a better acceptance result.
