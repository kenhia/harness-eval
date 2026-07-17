# feedhub

A small feed-aggregation system written in Rust. It fetches and stores
RSS 2.0 and Atom (RFC 4287) feeds in SQLite and exposes a JSON REST API.

The workspace contains three binaries and one shared library:

| crate     | kind   | role |
|-----------|--------|------|
| `feedd`   | binary | Long-running server: fetches/stores feeds, serves the REST API, polls in the background. |
| `feedctl` | binary | Command-line client that drives the `feedd` API. |
| `feedgen` | binary | Fixture tool: serves local test feeds over HTTP and generates a fixture corpus. |
| `feedlib` | library | Shared feed parsing, date handling, and common types. |

No network access happens at runtime or in tests except to the URLs you
explicitly configure. The test suite talks only to a local `feedgen`.

## Requirements

* Rust stable (developed against 1.96).
* [`just`](https://github.com/casey/just) is optional; every recipe maps to a
  plain `cargo` command (see [Checks](#checks)).

SQLite is bundled via the `rusqlite` `bundled` feature, so no system SQLite is
required.

## Build & install

```sh
# Build all three binaries in release mode.
cargo build --release
# Binaries land in target/release/{feedd,feedctl,feedgen}

# Or install them onto your PATH:
cargo install --path crates/feedd
cargo install --path crates/feedctl
cargo install --path crates/feedgen
```

## Quickstart

```sh
# 1. Generate a fixture corpus and serve it locally.
cargo run -q -p feedgen -- make-fixtures ./fixtures
cargo run -q -p feedgen -- serve --dir ./fixtures --listen 127.0.0.1:8700 &

# 2. Start the server against a SQLite database.
cargo run -q -p feedd -- --db ./feedhub.db --listen 127.0.0.1:8600 &

# 3. Drive it with the client.
feedctl add http://127.0.0.1:8700/rss.xml
feedctl refresh
feedctl list
feedctl entries --search hello
```

## `feedd` — the server

```
feedd --db PATH [--listen ADDR:PORT] [--poll-interval SECS]
```

* `--db PATH` — SQLite database file (created if missing). Required.
* `--listen ADDR:PORT` — listen address. Default `127.0.0.1:8600`.
* `--poll-interval SECS` — background refresh interval. Default `300`;
  `0` disables background polling. Refresh endpoints always fetch on demand,
  independent of the poller.

### Behavior

* **Formats.** RSS 2.0 and Atom, UTF-8 with a tolerated leading BOM.
  * RSS `<item>`: identity is `guid` (falling back to `link`); `title`,
    `link`, `description` → summary, `pubDate` (RFC 822).
  * Atom `<entry>`: identity is `id`; `title`, the `rel="alternate"` link
    (or the first `<link>`), `summary` or `content` → summary, `published`
    (falling back to `updated`, RFC 3339).
* **Dates.** `published_at` is normalized to UTC and serialized RFC 3339 with
  a trailing `Z`. RFC 822 accepts 4-digit years, numeric offsets, and the zone
  names `GMT UT Z EST EDT CST CDT MST MDT PST PDT`. RFC 3339 accepts any
  numeric offset and optional fractional seconds. A missing or unparseable
  date stores `published_at = null` (the fetch time is never substituted); the
  entry is still stored.
* **Text.** Titles and summaries are stored after XML entity unescaping; CDATA
  is taken verbatim. A missing title is stored as `(untitled)`.
* **Dedupe/update.** The identity key is (feed, guid-or-id). Re-seeing a known
  key updates `title`/`link`/`summary`/`published_at` in place; the row keeps
  its internal id and original `fetched_at`.
* **Conditional GET.** Each feed's `ETag` and `Last-Modified` are stored and
  sent back as `If-None-Match` / `If-Modified-Since`. A `304` leaves entries
  untouched, counts as a successful fetch, and reports `new_entries: 0`.
* **Failure isolation.** A fetch/parse failure is recorded on that feed's
  `last_error` and never affects other feeds or crashes the server.

### REST API

All responses are JSON. Errors are `{"error": "<message>"}` with a 4xx/5xx
status.

#### `GET /api/health`

Liveness check.

```json
{ "status": "ok" }
```

#### `POST /api/feeds`

Register a feed. Body: `{"url": "https://example.com/feed.xml"}`.

* `201` + the feed object on success.
* `409` if the URL is already registered.
* `422` if the URL is not a valid `http`/`https` URL.

#### `GET /api/feeds`

Array of feed objects.

#### `GET /api/feeds/{id}`

A single feed object, or `404`.

#### `DELETE /api/feeds/{id}`

`204` on success; the feed's entries are deleted too. `404` if unknown.

#### `POST /api/feeds/{id}/refresh`

Fetch the feed now. `404` if unknown, otherwise `200` with a result object:

```json
{ "status": "ok", "new_entries": 3, "feed_id": 1 }
```

On a `304` the result also carries `"not_modified": true`. On failure:
`{ "status": "error", "new_entries": 0, "error": "...", "feed_id": 1 }`.

#### `POST /api/refresh`

Refresh all feeds. Returns an array of the same result objects, one per feed,
each including its `feed_id`. Per-feed failures are isolated.

#### `GET /api/entries`

Query stored entries. All query parameters are optional:

| param     | meaning |
|-----------|---------|
| `feed_id` | Restrict to one feed. |
| `since`   | RFC 3339 instant; lower bound (inclusive). |
| `until`   | RFC 3339 instant; upper bound (exclusive). |
| `q`       | Case-insensitive substring match on the title (ASCII folding). |
| `limit`   | Default `50`, max `500`. |
| `offset`  | Default `0`. |

The time window is **half-open**: `since <= published_at < until`, compared as
UTC instants. When either bound is given, entries with a null `published_at`
are excluded. Results are ordered by `published_at` descending with **nulls
last**, ties broken by internal entry id ascending.

```json
{
  "total": 2,
  "items": [
    {
      "id": 2,
      "feed_id": 1,
      "guid": "post-2",
      "title": "Second Post",
      "link": "http://example.com/posts/2",
      "summary": "Another post & more.",
      "published_at": "2006-01-03T09:00:00Z",
      "fetched_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```

`total` is the full match count, ignoring `limit`/`offset`.

### Feed object

```json
{
  "id": 1,
  "url": "http://example.com/rss.xml",
  "title": "Example RSS Feed",
  "last_fetched_at": "2024-01-01T00:00:00Z",
  "last_error": null,
  "entry_count": 2
}
```

`title` is `null` until the first successful fetch and updates on refresh.
`last_error` is `null` when the last fetch succeeded.

## `feedctl` — the client

```
feedctl [--server URL] [--format text|json] <command>
```

* `--server URL` — base URL of the server. Default `http://127.0.0.1:8600`.
* `--format text|json` — output format. Default `text`. `json` prints the raw
  API response as a single JSON document on stdout.

### Commands

| command | description |
|---------|-------------|
| `add URL` | Register a feed. |
| `list` | List all feeds. |
| `show ID` | Show one feed. |
| `remove ID` | Remove a feed and its entries. |
| `refresh [ID]` | Refresh one feed, or all feeds when `ID` is omitted. |
| `entries [--feed ID] [--since T] [--until T] [--search Q] [--limit N] [--offset N]` | Query entries. |

### Exit codes

* `0` — success.
* `1` — the server answered with an error (message printed to stderr).
* `2` — the server was unreachable, or the command line was invalid.

Examples:

```sh
feedctl add http://127.0.0.1:8700/atom.xml
feedctl --format json list
feedctl entries --since 2005-01-01T00:00:00Z --until 2006-01-01T00:00:00Z
feedctl refresh 1
```

## `feedgen` — the fixture tool

```
feedgen serve --dir DIR [--listen ADDR:PORT]
feedgen make-fixtures DIR
```

* `serve` serves the files in `DIR` over HTTP with the correct `Content-Type`,
  a content-derived `ETag`, and a `Last-Modified` header, honoring
  `If-None-Match` / `If-Modified-Since` with a `304`. Default listen address is
  `127.0.0.1:8700`.
* `make-fixtures` writes the fixture corpus into `DIR` (created if missing).

### Fixture corpus

`make-fixtures` writes these files (also documented in the generated
`DIR/README.md`):

| file            | purpose |
|-----------------|---------|
| `rss.xml`       | A valid RSS 2.0 feed. |
| `atom.xml`      | A valid Atom feed; a second entry uses `content` + `updated` only. |
| `dates.xml`     | Edge-case dates: `PST`/`EDT`/`UT` zone names, a missing date, an unparseable date. |
| `cdata.xml`     | CDATA sections (verbatim) and entity unescaping. |
| `malformed.xml` | Deliberately malformed XML for failure-isolation tests. |
| `README.md`     | Describes the corpus. |

All fixtures are synthetic; no real-world feeds are bundled.

## Checks

A single command runs the full gate (format check, clippy, tests, release
build):

```sh
just check
```

If you don't have `just`, run the equivalent cargo commands:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

The test suite includes an end-to-end test (`crates/feedd/tests/e2e.rs`) that
launches a real `feedd` process and drives it against an in-process `feedgen`
server over local HTTP.

## Storage

A single SQLite file (given by `--db`). Schema:

* `feeds(id, url UNIQUE, title, last_fetched_at, last_error, etag, last_modified)`
* `entries(id, feed_id → feeds ON DELETE CASCADE, guid, title, link, summary,
  published_at, fetched_at, UNIQUE(feed_id, guid))`

Timestamps are stored as UTC epoch seconds and serialized to RFC 3339 `Z` in
the API.

## License

MIT.
