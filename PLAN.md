# feedhub — implementation plan

Greenfield Rust workspace: `feedd` (server), `feedctl` (CLI), `feedgen` (fixture
server), plus `feedhub-core` (shared lib: types, feed parsing, date/text handling,
fixture corpus).

## Premises (stated, not assumed)

1. **The spec pins the hard parts.** Dates, text handling, dedupe identity, window
   semantics, and ordering are all specified exactly. This is a build task, not a
   product-discovery task. Planning effort belongs in *how to satisfy the pins
   provably*, not in re-litigating what to build.
2. **Determinism beats background machinery.** Refresh endpoints fetch on demand;
   tests drive those, never the poll loop. The poll loop is thin and reuses the same
   refresh path.
3. **Hand-rolled parsing beats `feed-rs`.** A general feed crate has its own opinions
   about identity and dates (notably substituting fetch time for missing dates) that
   directly contradict the pinned spec. Owning ~300 lines of parser is cheaper than
   fighting a dependency's semantics. This is the main "don't reinvent" exception and
   it is deliberate.
4. **No network in tests.** Everything binds `127.0.0.1:0` and talks to `feedgen`.

## Architecture

```
feedhub/
├── Cargo.toml                  workspace
├── justfile                    `just check` = fmt + clippy + test + build
├── README.md
└── crates/
    ├── feedhub-core/           lib only
    │   ├── model.rs            Feed, Entry, FeedMeta, ParsedFeed
    │   ├── datetime.rs         RFC 822 + RFC 3339 parsing → UTC, storage format
    │   ├── text.rs             entity unescape / CDATA / (untitled)
    │   ├── parser.rs           quick-xml pull parser, RSS 2.0 + Atom
    │   └── fixtures.rs         fixture corpus (shared by feedgen + tests)
    ├── feedd/                  lib + bin
    │   ├── store.rs            rusqlite schema + queries
    │   ├── fetch.rs            reqwest + conditional GET
    │   ├── api.rs              axum routes
    │   └── poll.rs             background poller
    ├── feedctl/                bin (+ thin lib for arg parsing tests)
    └── feedgen/                lib + bin

feedd ─┬─> feedhub-core (parse)     feedgen ──> feedhub-core (fixtures)
       └─> rusqlite, axum, reqwest  feedctl ──> reqwest (API shapes only)
```

`feedd` and `feedgen` carry lib targets so the e2e test spawns both in-process on
ephemeral ports. Binaries are still named exactly `feedd` / `feedctl` / `feedgen`.

## Crate choices

| need | pick | why |
|---|---|---|
| HTTP server | `axum` | standard, tower ecosystem, clean extractors |
| HTTP client | `reqwest` (rustls) | no OpenSSL system dep |
| SQLite | `rusqlite` (`bundled`) | no system sqlite dep; sync API is simpler than sqlx |
| XML | `quick-xml` | pull parser; exposes Text vs CData separately — exactly what the pinned text rule needs |
| dates | `chrono` | RFC 3339 parsing; RFC 822 hand-rolled (see below) |
| CLI | `clap` (derive) | standard |

## Pinned-semantics decisions (the parts that are easy to get wrong)

**D1 — storage format is second-granular `%Y-%m-%dT%H:%M:%SZ`.**
Fixed-width UTC means lexicographic string compare == chronological compare, so
SQLite ordering and range filters are correct without date functions. Mixing
fractional and non-fractional breaks this (`00:00:00.5Z` < `00:00:00Z`
lexicographically, because `.` < `Z` — wrong). Spec requires *accepting* fractional
seconds, not preserving them. Truncate on ingest.

**D2 — `since`/`until` bounds are ceiled to the next whole second when they carry
sub-second precision.** Stored values are integral seconds. For half-open
`since <= t < until`: `since=00:00:00.7` must exclude a stored `00:00:00`, and
`until=00:00:00.7` must include it. Ceiling both bounds yields exactly that. Without
this, sub-second bounds are silently off by one second.

**D3 — ordering is `(published_at IS NULL) ASC, published_at DESC, id ASC`.**
Explicit rather than `NULLS LAST` — same result, no dependency on SQLite version.

**D4 — no `std::sync::Mutex` guard held across `.await`.** Store API is synchronous
and takes `&Connection`; handlers lock, call, drop, *then* await. Network fetches
happen outside the lock. clippy's `await_holding_lock` enforces this at
`-D warnings`, so the invariant is compiler-checked rather than convention.

**D5 — RFC 822 parser is hand-rolled.** chrono's `parse_from_rfc2822` follows RFC
2822, which maps obsolete zone names (`EST`, `PDT`, …) to `-0000`/unknown rather
than to real offsets. The spec pins those zones to actual offsets. ~100 lines,
directly unit-testable.

**D6 — entries with no identity (no guid *and* no link; Atom with no id) are
skipped.** Spec pins identity as the dedupe key; an entry without one has no stable
key and would duplicate on every refresh. Skipping is the honest failure. Documented
in README.

**D7 — dedupe is `INSERT … ON CONFLICT(feed_id, guid) DO UPDATE`** setting
title/link/summary/published_at only. `id` and `fetched_at` are untouched by the
update, satisfying "keeps its internal id and original fetched_at". `new_entries`
counts only true inserts (via `changes()` discrimination — see store.rs).

## Schema

```sql
CREATE TABLE feeds (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  url TEXT NOT NULL UNIQUE,
  title TEXT, last_fetched_at TEXT, last_error TEXT,
  etag TEXT, last_modified TEXT
);
CREATE TABLE entries (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
  guid TEXT NOT NULL,
  title TEXT NOT NULL, link TEXT, summary TEXT,
  published_at TEXT, fetched_at TEXT NOT NULL,
  UNIQUE(feed_id, guid)
);
```
`PRAGMA foreign_keys = ON` per connection (SQLite defaults it off — DELETE cascade
silently no-ops without it).

## Test plan

| layer | covers |
|---|---|
| `core/datetime` unit | every pinned zone name, numeric offsets, 4-digit years, fractional secs, garbage → None, ceil helper |
| `core/text` unit | `&amp;`→`&`, CDATA verbatim, missing title → `(untitled)` |
| `core/parser` unit | RSS + Atom happy path, BOM, guid→link fallback, Atom rel=alternate vs first link, summary vs content, published vs updated, malformed XML → Err, no-identity skip |
| `feedd/store` unit | dedupe updates in place (id + fetched_at preserved), cascade delete, ordering incl. nulls-last + id tiebreak, window half-open, limit/offset vs total, `q` ASCII-insensitive |
| `feedd` api | every endpoint, 409, 422, 404, 204, error shape |
| `feedgen` unit | ETag stable + content-derived, 304 on If-None-Match / If-Modified-Since |
| **e2e** | feedd + feedgen in-process: add → refresh → entries; 304 second refresh reports `new_entries: 0`; malformed feed sets `last_error` and does not disturb a sibling feed |
| `feedctl` bin | exit codes 0 / 1 / 2 via `CARGO_BIN_EXE_feedctl` against a live server + a dead port |

## Failure modes

| mode | handling |
|---|---|
| connection refused / HTTP 5xx / malformed XML | recorded to `feeds.last_error`; other feeds unaffected; server does not crash |
| 304 | success, entries untouched, `new_entries: 0`, `last_error` cleared |
| missing/unparseable date | `published_at = null`, entry still stored, never fetch time |
| feed with no items | success, `new_entries: 0` |
| poll loop panic | fetches are per-feed and error-isolated; loop logs and continues |

## NOT in scope

- Auth, rate limiting, pagination cursors (spec doesn't ask; offset paging is pinned)
- Atom xhtml `content` tree reconstruction (text extraction is sufficient)
- Feed discovery/autodiscovery, OPML import
- Real-network fetching in tests (explicitly forbidden)
