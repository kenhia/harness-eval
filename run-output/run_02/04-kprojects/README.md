# feedhub

A small feed-aggregation system written in Rust. It fetches and stores
RSS 2.0 and Atom feeds in SQLite and exposes a REST API.

It is a Cargo workspace of three binaries plus a shared library:

| crate | kind | role |
|---|---|---|
| `feedd` | binary | long-running server: fetches/stores feeds, serves the REST API |
| `feedctl` | binary | command-line client that drives the API |
| `feedgen` | binary + lib | fixture tool: serves test feeds over local HTTP and generates a corpus |
| `feedcore` | library | shared feed parsing, date/text normalization, and common types |

Everything can be developed and tested offline — `feedgen` stands in for
the real internet, and nothing fetches URLs other than those you configure.

## Build & install

Requires a stable Rust toolchain (1.80+).

```sh
cargo build --release        # builds feedd, feedctl, feedgen
# binaries land in target/release/{feedd,feedctl,feedgen}

cargo install --path crates/feedd     # optional: install into ~/.cargo/bin
cargo install --path crates/feedctl
cargo install --path crates/feedgen
```

## Running the checks

A single command runs every CI gate (format check, clippy with warnings as
errors, and the full test suite, including an end-to-end test that drives
`feedd` against `feedgen` over local HTTP):

```sh
just check
# equivalent to:
#   cargo fmt --all --check
#   cargo clippy --all-targets -- -D warnings
#   cargo test
```

## Quick start

```sh
# 1. Generate a corpus of test feeds and serve it.
cargo run -p feedgen -- make-fixtures .scratch/corpus
cargo run -p feedgen -- serve --dir .scratch/corpus --listen 127.0.0.1:8700 &

# 2. Start the server.
cargo run -p feedd -- --db feedhub.sqlite --poll-interval 0 &

# 3. Drive it with the client.
feedctl add http://127.0.0.1:8700/rss.xml
feedctl refresh
feedctl list
feedctl entries --search hello
feedctl --format json entries --since 2024-01-01T00:00:00Z
```

## `feedd` — the server

```
feedd --db PATH [--listen ADDR:PORT] [--poll-interval SECS]
```

- `--db PATH` (required): SQLite database file, created if missing.
- `--listen` (default `127.0.0.1:8600`): REST API bind address.
- `--poll-interval` (default `300`): background poll interval in seconds;
  `0` disables background polling. Refresh endpoints always fetch on demand
  regardless of this setting.

Set `RUST_LOG` (e.g. `RUST_LOG=feedd=debug`) to control logging.

### Behavior notes

- **Formats:** RSS 2.0 and Atom (RFC 4287), UTF-8 with an optional leading
  BOM.
- **Dates:** `published_at` is normalized to UTC and serialized as RFC 3339
  with a `Z` suffix. RFC 822 dates accept 4-digit years, numeric offsets
  (`+0000`, `-0500`), and the zone names `GMT UT Z EST EDT CST CDT MST MDT
  PST PDT`. RFC 3339 dates accept any numeric offset and optional fractional
  seconds. A missing or unparseable date stores `published_at = null` (the
  fetch time is never substituted); the entry is still stored.
- **Text:** titles and summaries are stored after XML entity unescaping
  (`&amp;` → `&`); CDATA is taken verbatim; a missing title is stored as
  `(untitled)`.
- **Dedupe/update:** identity is `(feed, guid-or-id)` — RSS `guid` (falling
  back to `link`), Atom `id`. Re-seeing a known key updates the title, link,
  summary, and published_at in place, keeping the entry's internal id and
  original `fetched_at`.
- **Conditional GET:** each feed's `ETag` and `Last-Modified` are stored and
  sent as `If-None-Match` / `If-Modified-Since` on refetch. A `304` leaves
  entries untouched, counts as a successful fetch, and reports
  `new_entries: 0`.
- **Failure isolation:** a fetch/parse failure is recorded on that feed as
  `last_error` and never affects other feeds or crashes the server.

## REST API reference

All responses are JSON. Errors are `{"error": "<message>"}` with an
appropriate 4xx/5xx status.

### `GET /api/health`

Liveness probe.

```json
{ "status": "ok" }
```

### `POST /api/feeds`

Register a feed. Body: `{"url": "https://example.com/feed.xml"}`.

- `201` + feed object on success.
- `409` if the URL is already registered.
- `422` if the URL is not a valid `http(s)` URL.

### `GET /api/feeds`

Array of feed objects.

### `GET /api/feeds/{id}`

A single feed object, or `404`.

### `DELETE /api/feeds/{id}`

`204` on success; the feed's entries are deleted too. `404` if unknown.

### `POST /api/feeds/{id}/refresh`

Fetch the feed now. `404` if the feed is unknown. Returns a refresh result:

```json
{ "feed_id": 1, "status": "ok", "new_entries": 2 }
```

On a `304`, the result additionally includes `"not_modified": true` with
`"new_entries": 0`. On failure, `"status": "error"` with an `"error"` field.

### `POST /api/refresh`

Refresh all feeds. Returns an array of refresh result objects (each includes
its `feed_id`).

### `GET /api/entries`

Query stored entries. All query parameters are optional:

| param | meaning |
|---|---|
| `feed_id` | restrict to one feed |
| `since` | RFC 3339 lower bound (inclusive), compared as a UTC instant |
| `until` | RFC 3339 upper bound (exclusive), compared as a UTC instant |
| `q` | case-insensitive substring match on the title |
| `limit` | page size, default 50, max 500 |
| `offset` | page offset, default 0 |

Window semantics are half-open: `since <= published_at < until`. When either
bound is given, entries with a null `published_at` are excluded. Results are
ordered by `published_at` descending (nulls last), ties broken by internal
entry id ascending. `total` is the match count ignoring `limit`/`offset`.

```json
{
  "total": 42,
  "items": [
    {
      "id": 7,
      "feed_id": 1,
      "guid": "urn:example:hello",
      "title": "Hello RSS",
      "link": "http://example.invalid/hello",
      "summary": "First post.",
      "published_at": "2024-01-02T20:04:05Z",
      "fetched_at": "2024-06-01T12:00:00Z"
    }
  ]
}
```

### Object fields

**Feed:** `id`, `url`, `title` (null until first successful fetch),
`last_fetched_at`, `last_error` (null when the last fetch succeeded),
`entry_count`.

**Entry:** `id`, `feed_id`, `guid`, `title`, `link`, `summary`,
`published_at`, `fetched_at`.

## `feedctl` — the client

```
feedctl [--server URL] [--format text|json] <command>
```

- `--server` (default `http://127.0.0.1:8600`).
- `--format` (default `text`): `json` prints the raw API response as a single
  JSON document on stdout.

Commands:

| command | description |
|---|---|
| `add URL` | register a feed |
| `list` | list feeds |
| `show ID` | show one feed |
| `remove ID` | delete a feed |
| `refresh [ID]` | refresh one feed, or all feeds if no ID |
| `entries [--feed ID] [--since T] [--until T] [--search Q] [--limit N] [--offset N]` | query entries |

Exit codes: `0` success; `1` the server answered with an error (message on
stderr); `2` the server is unreachable or the usage is invalid.

## `feedgen` — the fixture tool

```
feedgen serve --dir DIR [--listen ADDR:PORT]     # default 127.0.0.1:8700
feedgen make-fixtures DIR
```

`serve` exposes the files in `DIR` over HTTP/1.1 with the correct
`Content-Type`, a content-derived `ETag`, and a `Last-Modified` header, and
honors `If-None-Match` / `If-Modified-Since` with a `304`.

`make-fixtures` writes a documented corpus (see the generated `README.md` in
the target directory) covering:

- `rss.xml` — a valid RSS 2.0 feed.
- `atom.xml` — a valid Atom feed (alternate links, summary/content).
- `dates.xml` — edge-case dates: zone names (EST/PST/GMT), a numeric offset,
  and a missing/unparseable date (stored as null).
- `cdata.xml` — CDATA (verbatim) and XML entities (unescaped).
- `malformed.xml` — malformed XML (recorded as a feed error, never a crash).
- `truncated.xml` — well-formed XML cut off mid-element (a truncated upstream
  response); recorded as a feed error, not a successful empty fetch.

## Project layout

```
crates/
  feedcore/   shared types, date/text normalization, RSS+Atom parser
  feedd/      server: SQLite storage, REST API, fetch, polling
  feedctl/    CLI client
  feedgen/    fixture HTTP server + corpus generator
```

## License

MIT — see [LICENSE](LICENSE).
