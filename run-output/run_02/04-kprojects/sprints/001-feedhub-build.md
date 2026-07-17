# 001 — feedhub end-to-end build

**Goal:** build feedhub start to finish — a Rust feed-aggregation service
with a server (`feedd`), a CLI client (`feedctl`), and a local fixture tool
(`feedgen`), all offline-testable.

## What shipped

A four-crate Cargo workspace:

- **`feedcore`** (lib): shared `ParsedFeed`/`ParsedItem` types, a `quick-xml`
  streaming parser for RSS 2.0 and Atom, and the pinned date/text
  normalization (`date.rs`, `text.rs`).
- **`feedd`** (bin+lib): `rusqlite` storage, an `axum` REST API, on-demand
  fetching with conditional GET, dedupe/update, failure isolation, and
  optional background polling. Exposed as a library so the integration test
  can drive it in-process.
- **`feedctl`** (bin): blocking `reqwest` client with `text`/`json` output
  and the specified `0/1/2` exit-code contract.
- **`feedgen`** (bin+lib): a minimal hand-rolled HTTP/1.1 static server
  (content-derived ETag, Last-Modified, conditional GET → 304) plus a
  deterministic fixture corpus generator.

## Key decisions

- **Stack:** `axum` + `reqwest` (rustls) + `rusqlite` (bundled SQLite) +
  `quick-xml` + `chrono` + `clap`. This mirrors the house `axum`/`reqwest`
  style recorded in klams, swapped to SQLite per spec.
- **Concurrency:** a single SQLite `Connection` behind a `std::Mutex` in the
  app state. The lock is only ever held across synchronous DB work — never
  across an `await` — so network fetches stay concurrent while writes are
  serialized. Simple and correct at this scale.
- **Custom RFC 822 parser** rather than leaning on chrono's rfc2822, to
  guarantee the pinned named-zone set (`EST/EDT/.../PDT`, `GMT/UT/Z`) and the
  2-digit-year pivot.
- **Date comparison via normalized strings:** everything is stored as
  fixed-width `YYYY-MM-DDTHH:MM:SSZ`, so lexicographic ordering equals
  chronological ordering. The entries window and `ORDER BY published_at IS
  NULL, published_at DESC, id ASC` fall straight out of SQL.
- **`feedgen` HTTP by hand** (no axum dep) to keep total control of ETag /
  `If-None-Match` / `If-Modified-Since` / 304 semantics that feedd's
  conditional-GET path is tested against.
- **Synthetic identity fallback** for items with neither guid/id nor link:
  a deterministic content-derived key so dedupe stays stable across refetches.

## Verification

- `cargo test` green: 15 feedcore unit tests, feedgen + feedd unit tests, and
  a full end-to-end test that generates the corpus, serves it via `feedgen`,
  and drives every `feedd` endpoint over local HTTP (register/dup/refresh/
  304/dedupe/window/order/search/delete-cascade/error-isolation).
- `cargo fmt --all --check` and `cargo clippy --all-targets -- -D warnings`
  clean; `cargo build --release` produces all three binaries.
- Manual smoke test of the real binaries confirmed named-zone dates
  (EST→+5h, PST→+8h, `+0530`→−5.5h), nulls-last ordering, null exclusion
  under a window, and the `0/1/2` exit codes.

## Follow-ups / ideas

- Concurrent (fan-out) refresh in `POST /api/refresh` instead of sequential.
- `feedd` graceful-shutdown already wired to Ctrl-C; could add SIGTERM.
- Optional entry body/content storage and richer `feedctl` table formatting.
