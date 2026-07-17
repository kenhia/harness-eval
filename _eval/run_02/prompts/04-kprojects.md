Build the feedhub service described below, start to finish, following this repository's project conventions.

## Project: feedhub — a feed aggregation service

Build this project start to finish in the current repository: working code,
tests, documentation, all committed to git.

### What it is

`feedhub` is a small feed-aggregation system written in Rust: a long-running
server (`feedd`) that fetches and stores RSS/Atom feeds and exposes a REST
API, a CLI client (`feedctl`) that drives that API, and a fixture tool
(`feedgen`) that serves test feeds over local HTTP so everything can be
developed and tested without touching the real internet.

### Requirements — general

- Rust (stable), organized as a **Cargo workspace** with three binary
  crates named exactly `feedd`, `feedctl`, and `feedgen`, plus at least one
  shared library crate (name your choice) for feed parsing and common types.
- `cargo build --release` builds all three binaries; `cargo test` passes;
  `cargo fmt --check` is clean; `cargo clippy --all-targets -- -D warnings`
  is clean.
- Storage is **SQLite**, in a single database file whose path is given on
  the command line. Choice of crates is otherwise yours.
- No network access at runtime or in tests except to explicitly configured
  URLs (in tests: only `feedgen` or equivalent local servers). Fetching
  crates during build is fine.

### `feedd` — the server

`feedd --db PATH [--listen ADDR:PORT] [--poll-interval SECS]`

- `--listen` defaults to `127.0.0.1:8600`; `--poll-interval` defaults to
  `300` and `0` disables background polling. Independent of background
  polling, refresh endpoints fetch on demand (tests rely on these for
  determinism).
- Feed formats: **RSS 2.0** and **Atom (RFC 4287)**, UTF-8 (tolerate a
  leading BOM). Item mapping:
  - RSS `<item>`: identity = `guid` (fall back to `link`); `title`,
    `link`, `description` → summary, `pubDate` (RFC 822/1123).
  - Atom `<entry>`: identity = `id`; `title`, the `rel="alternate"` link
    (or the first `<link>`), `summary` or `content` → summary,
    `published` falling back to `updated` (RFC 3339).
- **Date handling (pinned).** Store `published_at` normalized to **UTC**,
  serialized RFC 3339 with `Z`. RFC 822 dates must accept 4-digit years,
  numeric offsets (`+0000`, `-0500`), and the zone names
  `GMT UT Z EST EDT CST CDT MST MDT PST PDT`. RFC 3339 dates must accept
  any numeric offset and optional fractional seconds. A missing or
  unparseable date stores `published_at = null` — never substitute fetch
  time. The entry is still stored.
- **Text handling (pinned).** Titles and summaries are stored after XML
  entity unescaping (`&amp;` → `&`); CDATA content is taken verbatim. A
  missing title is stored as `(untitled)`.
- **Dedupe/update (pinned).** Identity key is (feed, guid-or-id). Seeing a
  known key again **updates** title/link/summary/published_at in place —
  no duplicate row; the entry keeps its internal id and original
  `fetched_at`.
- **Conditional GET.** Store each feed's `ETag` and `Last-Modified` and
  send `If-None-Match` / `If-Modified-Since` on refetch. A `304` response
  leaves entries untouched, counts as a successful fetch, and reports
  `new_entries: 0`.
- **Failure isolation.** A fetch/parse failure (connection refused, HTTP
  error, malformed XML) is recorded on that feed (`last_error`) and never
  affects other feeds or crashes the server.

REST API (JSON; errors are `{"error": "<message>"}` with an appropriate
4xx/5xx status):

| method + path | behavior |
|---|---|
| `GET /api/health` | `{"status":"ok"}` (extra fields allowed) |
| `POST /api/feeds` | body `{"url": "..."}`; 201 + feed object; 409 if the URL is already registered; 422 if the URL is not valid http(s) |
| `GET /api/feeds` | array of feed objects |
| `GET /api/feeds/{id}` | feed object or 404 |
| `DELETE /api/feeds/{id}` | 204; the feed's entries are deleted too |
| `POST /api/feeds/{id}/refresh` | fetch now; returns `{"status":"ok"\|"error", "new_entries": N, ...}` |
| `POST /api/refresh` | refresh all feeds; returns per-feed results (array of the same result objects, each including the feed id) |
| `GET /api/entries` | query entries, see below |

Feed object fields (at minimum): `id`, `url`, `title` (null until first
successful fetch; updates on refresh), `last_fetched_at`, `last_error`
(null when the last fetch succeeded), `entry_count`.

`GET /api/entries` — query parameters, all optional:

- `feed_id` — restrict to one feed.
- `since`, `until` — RFC 3339 instants (any offset). **Window semantics
  (pinned): half-open, `since <= published_at < until`, compared as UTC
  instants.** When either bound is given, entries with null `published_at`
  are excluded.
- `q` — case-insensitive substring match on title (ASCII case folding is
  sufficient).
- `limit` (default 50, max 500), `offset` (default 0).
- Response: `{"total": N, "items": [...]}` — `total` is the match count
  ignoring limit/offset. **Ordering (pinned): `published_at` descending,
  nulls last, ties broken by internal entry id ascending.**
- Entry object fields (at minimum): `id`, `feed_id`, `guid`, `title`,
  `link`, `summary`, `published_at`, `fetched_at`.

### `feedctl` — the client

`feedctl [--server URL] [--format text|json] <command>`

- `--server` defaults to `http://127.0.0.1:8600`; `--format` defaults to
  `text` (human-readable); `json` prints the raw API response as a single
  valid JSON document on stdout.
- Commands: `add URL`, `list`, `show ID`, `remove ID`, `refresh [ID]`
  (no ID = refresh all), `entries [--feed ID] [--since T] [--until T]
  [--search Q] [--limit N] [--offset N]`.
- Exit codes: `0` success; `1` the server answered with an error (message
  on stderr); `2` the server is unreachable or the usage is invalid.

### `feedgen` — the fixture tool

- `feedgen serve --dir DIR [--listen ADDR:PORT]` — serve the files in DIR
  over HTTP with correct `Content-Type`, a content-derived `ETag`, and
  `Last-Modified`; honor `If-None-Match` / `If-Modified-Since` with `304`.
- `feedgen make-fixtures DIR` — write a fixture corpus into DIR covering
  at least: a valid RSS 2.0 feed, a valid Atom feed, a feed with
  edge-case dates (zone names, missing date), a feed exercising
  CDATA/entities, and a malformed-XML file. Document the corpus (README
  or `--help`).

### Definition of done

- All three binaries implemented per spec; `cargo test` passes and
  includes at least one end-to-end test that drives `feedd` against
  `feedgen` over local HTTP; `cargo fmt --check` and
  `cargo clippy --all-targets -- -D warnings` are clean.
- `README.md` covering build/install, usage of all three binaries, and an
  API reference for every endpoint.
- Meaningful git commits as you go; final state committed.
- A single command to run all checks (`just check` or a documented
  equivalent).

### Operating rules

- Work autonomously start to finish; do not stop to ask questions unless
  truly blocked.
- Use whatever workflow, planning, and review machinery your environment
  provides.
- Researching feed-format or HTTP specifics on the web is expected and
  allowed; fetching real-world feeds is not.
