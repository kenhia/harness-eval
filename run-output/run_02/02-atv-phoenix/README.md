# feedhub

A small feed-aggregation system written in Rust. It fetches and stores RSS 2.0
and Atom feeds, exposes a JSON REST API, and ships a CLI client and a fixture
server so everything can be developed and tested without touching the real
internet.

It is a Cargo workspace with three binaries and one shared library:

| crate | kind | role |
|-------|------|------|
| `feedd` | binary | long-running server: fetch + store feeds, serve the REST API |
| `feedctl` | binary | command-line client that drives the `feedd` API |
| `feedgen` | binary | fixture tool: serves and generates test feeds over local HTTP |
| `feedcore` | library | shared feed parsing, date/text handling, SQLite storage, fetching |

Storage is a single SQLite database file. At runtime and in tests, the only
network access is to explicitly configured URLs (in tests, only `feedgen`).

## Build & install

Requires a stable Rust toolchain (`cargo`, `rustfmt`, `clippy`).

```sh
# Build all three binaries (optimized).
cargo build --release
# Binaries land in target/release/{feedd,feedctl,feedgen}

# Optionally install them onto your PATH.
cargo install --path crates/feedd
cargo install --path crates/feedctl
cargo install --path crates/feedgen
```

### Run all checks

A single command runs formatting, build, lint, and the full test suite
(including an end-to-end test that drives `feedd` against `feedgen` over local
HTTP):

```sh
just check
```

If you don't have [`just`](https://github.com/casey/just), run the equivalent
directly:

```sh
cargo fmt --check
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test
```

## Quick start

```sh
# 1. Generate a fixture corpus and serve it locally.
cargo run -p feedgen -- make-fixtures ./fixtures
cargo run -p feedgen -- serve --dir ./fixtures --listen 127.0.0.1:8700 &

# 2. Start the server against a fresh database.
cargo run -p feedd -- --db feedhub.db --listen 127.0.0.1:8600 --poll-interval 0 &

# 3. Drive it with the client.
cargo run -p feedctl -- add http://127.0.0.1:8700/rss.xml
cargo run -p feedctl -- refresh
cargo run -p feedctl -- entries
cargo run -p feedctl -- --format json list
```

## `feedd` — the server

```
feedd --db PATH [--listen ADDR:PORT] [--poll-interval SECS]
```

- `--db PATH` — SQLite database file (required; created if missing).
- `--listen` — default `127.0.0.1:8600`.
- `--poll-interval` — background poll interval in seconds; default `300`,
  `0` disables background polling. Independent of polling, the refresh
  endpoints always fetch on demand.

### Feed handling

- **Formats:** RSS 2.0 and Atom (RFC 4287), UTF-8, tolerating a leading BOM.
  - RSS `<item>`: identity = `guid` (falling back to `link`); `title`, `link`,
    `description` → summary, `pubDate`.
  - Atom `<entry>`: identity = `id`; `title`, the `rel="alternate"` link (or the
    first `<link>`), `summary` or `content` → summary, `published` (falling back
    to `updated`).
- **Dates:** stored as UTC, serialized RFC 3339 with `Z`. RFC 822 accepts
  4-digit years, numeric offsets (`+0000`, `-0500`), and the zone names
  `GMT UT Z EST EDT CST CDT MST MDT PST PDT`. RFC 3339 accepts any numeric
  offset and optional fractional seconds. A missing or unparseable date stores
  `published_at = null` (the entry is still stored) — fetch time is never
  substituted.
- **Text:** titles/summaries are stored after XML entity unescaping
  (`&amp;` → `&`); CDATA is taken verbatim; a missing title becomes
  `(untitled)`.
- **Dedupe/update:** identity is (feed, guid-or-id). Seeing a known key again
  updates title/link/summary/published_at in place — no duplicate row; the
  entry keeps its internal id and original `fetched_at`.
- **Conditional GET:** each feed's `ETag` and `Last-Modified` are stored and
  sent as `If-None-Match` / `If-Modified-Since` on refetch. A `304` leaves
  entries untouched, counts as a successful fetch, and reports `new_entries: 0`.
- **Failure isolation:** a fetch/parse failure is recorded on that feed's
  `last_error` and never affects other feeds or crashes the server.

## REST API reference

All responses are JSON. Errors are `{"error": "<message>"}` with a 4xx/5xx
status.

### `GET /api/health`
Liveness probe.
- **200** → `{"status": "ok"}` (extra fields allowed).

### `POST /api/feeds`
Register a feed.
- Body: `{"url": "http://..."}`.
- **201** → the feed object.
- **409** if the URL is already registered.
- **422** if the URL is not a valid http(s) URL (or the body has no `url`).

### `GET /api/feeds`
- **200** → array of feed objects.

### `GET /api/feeds/{id}`
- **200** → the feed object.
- **404** if no such feed.

### `DELETE /api/feeds/{id}`
Delete a feed and all of its entries.
- **204** on success.
- **404** if no such feed.

### `POST /api/feeds/{id}/refresh`
Fetch one feed now.
- **200** → a refresh result: `{"feed_id": N, "status": "ok"|"error",
  "new_entries": N, "error"?: "...", "not_modified"?: true}`.
- **404** if no such feed.

### `POST /api/refresh`
Refresh every feed.
- **200** → an array of the refresh result objects above (one per feed, each
  including its `feed_id`).

### `GET /api/entries`
Query stored entries. All query parameters are optional:

| param | meaning |
|-------|---------|
| `feed_id` | restrict to one feed |
| `since` | RFC 3339 instant; lower bound (inclusive) |
| `until` | RFC 3339 instant; upper bound (exclusive) |
| `q` | case-insensitive substring match on title |
| `limit` | default 50, max 500 |
| `offset` | default 0 |

- **Window semantics:** half-open, `since <= published_at < until`, compared as
  UTC instants. When either bound is given, entries with a null `published_at`
  are excluded.
- **Ordering:** `published_at` descending, nulls last, ties broken by internal
  entry id ascending.
- **200** → `{"total": N, "items": [...]}` — `total` is the match count
  ignoring `limit`/`offset`.
- **400** if a parameter is malformed (e.g. an unparseable date).

### Object shapes

Feed object:

```json
{
  "id": 1,
  "url": "http://example.com/rss.xml",
  "title": "Example RSS Feed",
  "last_fetched_at": "2024-07-16T00:00:00Z",
  "last_error": null,
  "entry_count": 2
}
```

`title` is `null` until the first successful fetch; `last_error` is `null` when
the last fetch succeeded.

Entry object:

```json
{
  "id": 10,
  "feed_id": 1,
  "guid": "urn:example:post:1",
  "title": "Hello RSS & World",
  "link": "http://example.com/posts/1",
  "summary": "First post in the RSS fixture.",
  "published_at": "2002-10-02T13:00:00Z",
  "fetched_at": "2024-07-16T00:00:00Z"
}
```

## `feedctl` — the client

```
feedctl [--server URL] [--format text|json] <command>
```

- `--server` — default `http://127.0.0.1:8600`.
- `--format` — `text` (default, human-readable) or `json` (prints the raw API
  response as a single JSON document on stdout).

Commands:

| command | description |
|---------|-------------|
| `add URL` | register a feed |
| `list` | list all feeds |
| `show ID` | show one feed |
| `remove ID` | delete a feed and its entries |
| `refresh [ID]` | refresh one feed, or all feeds when no id is given |
| `entries [--feed ID] [--since T] [--until T] [--search Q] [--limit N] [--offset N]` | query entries |

Exit codes:

- `0` — success.
- `1` — the server answered with an error (message printed on stderr).
- `2` — the server is unreachable, or usage was invalid.

## `feedgen` — the fixture tool

```
feedgen serve --dir DIR [--listen ADDR:PORT]
feedgen make-fixtures DIR
```

- `serve` — serve the files in `DIR` over HTTP with correct `Content-Type`, a
  content-derived `ETag`, and `Last-Modified`, honoring `If-None-Match` /
  `If-Modified-Since` with `304`. Default listen `127.0.0.1:8700`.
- `make-fixtures` — write a documented fixture corpus into `DIR`:

  | file | purpose |
  |------|---------|
  | `rss.xml` | valid RSS 2.0 feed |
  | `atom.xml` | valid Atom feed |
  | `dates.xml` | edge-case dates: zone names and a missing date |
  | `cdata.xml` | CDATA content and XML entities |
  | `malformed.xml` | intentionally malformed XML (drives `last_error`) |
  | `truncated.xml` | a document cut off mid-element (drives `last_error`) |
  | `README.md` | describes the corpus |

## Project layout

```
crates/
  feedcore/   shared library (parse, dates, text, store, fetch, service)
  feedd/      server binary + end-to-end integration test (tests/e2e.rs)
  feedctl/    client binary
  feedgen/    fixture tool binary
justfile      `just check` runs fmt + build + clippy + test
```

## License

MIT.
