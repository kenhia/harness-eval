# 002 — truncated XML is a fetch error, not a silent empty success

**Goal:** fix a reported bug where refreshing a feed whose body is
well-formed XML *so far* but cut off mid-element (a truncated upstream
response) reported `status: "ok"` with zero new entries and left
`last_error: null`, instead of recording a parse failure.

## Root cause

`feedcore::parse::parse_feed` is a streaming `quick-xml` parser. Its
`check_end_names` setting catches *mismatched* end tags (the existing
`malformed.xml` fixture), but a document that simply *ends* while elements
are still open produces an `Event::Eof` — and both the main loop and the
`read_text` helper treated EOF as a normal, successful end of document. So
a truncated feed parsed into a partial (often empty) `ParsedFeed` and the
refresh path in `feedd` recorded success.

## What shipped

- `parse.rs`: added a `ParseError::Truncated` variant and now track
  open-element nesting depth in the main loop (text elements are balanced
  internally by `read_text`, so they don't affect the counter). If EOF is
  reached with depth > 0, or `read_text` hits EOF before its element
  closes, parsing returns `Truncated`. The existing `feedd` fetch path
  already maps any `ParseError` to `last_error` + `status: "error"`, so no
  change was needed there.
- New unit tests: `truncated_mid_element_errors` (the exact repro from the
  bug report) and `truncated_between_elements_errors`.
- New `feedgen` fixture `truncated.xml` (the repro body) plus README/corpus
  notes, and the `feedd` e2e test now adds and refreshes it, asserting it
  records `last_error`.

## Verification

`just check` (fmt + clippy -D warnings + full test suite) is green.
Correctly-formed feeds are unaffected — all pre-existing parse and e2e
assertions still pass.

## Follow-ups

None. Behavior for valid feeds is unchanged; only the truncation gap is
closed.
