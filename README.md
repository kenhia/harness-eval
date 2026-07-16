# feedhub

A small feed-aggregation system in Rust.

- **`feedd`** — a long-running server that fetches and stores RSS 2.0 and Atom
  feeds in SQLite, and exposes a REST API.
- **`feedctl`** — a command line client for that API.
- **`feedgen`** — a fixture tool that generates a corpus of test feeds and serves
  them over local HTTP, so the whole system can be developed and tested without
  touching the real internet.

`feedhub-core` is the shared library: feed parsing, date and text normalization,
and the types that cross binary boundaries.

## Contents

- [Build and install](#build-and-install)
- [Quick start](#quick-start)
- [`feedd`](#feedd) · [`feedctl`](#feedctl) · [`feedgen`](#feedgen)
- [API reference](#api-reference)
- [Behavior worth knowing](#behavior-worth-knowing)
- [Development](#development)

## Build and install

Requires stable Rust (1.75+) and a C toolchain — SQLite is compiled from source
via `rusqlite`'s `bundled` feature, which is what lets feedhub run with no
system SQLite installed.

```sh
git clone <this repo> && cd feedhub
cargo build --release
```

The three binaries land in `target/release/`:

```sh
./target/release/feedd --help
./target/release/feedctl --help
./target/release/feedgen --help
```

To install them onto your `PATH`:

```sh
cargo install --path crates/feedd
cargo install --path crates/feedctl
cargo install --path crates/feedgen
```

## Quick start

No real feeds required. In three terminals:

```sh
# 1. Generate the fixture corpus and serve it on 127.0.0.1:8601.
feedgen make-fixtures /tmp/fixtures
feedgen serve --dir /tmp/fixtures

# 2. Run the server against a scratch database.
feedd --db /tmp/feedhub.db

# 3. Drive it.
feedctl add http://127.0.0.1:8601/rss-basic.rss
feedctl refresh
feedctl list
feedctl entries --limit 5
```

```
$ feedctl refresh
feed 1: ok, 3 new entries

$ feedctl list
  ID  ENTRIES  TITLE                           URL
   1        3  Example RSS Feed                http://127.0.0.1:8601/rss-basic.rss

$ feedctl entries --search post --limit 2
2003-06-12T12:00:00Z      [feed 1]  Third post, no guid
                          https://example.com/posts/3
2003-06-11T14:30:00Z      [feed 1]  Second post
                          https://example.com/posts/2

Showing 2 of 3 matching entries.
```

## `feedd`

```
feedd --db PATH [--listen ADDR:PORT] [--poll-interval SECS]
```

| flag | default | meaning |
|---|---|---|
| `--db PATH` | *required* | SQLite database file. Created if absent. |
| `--listen ADDR:PORT` | `127.0.0.1:8600` | Address to listen on. |
| `--poll-interval SECS` | `300` | Seconds between background polls. `0` disables background polling. |

Background polling and on-demand refresh are independent: `--poll-interval 0`
turns off the loop, and `POST /api/feeds/{id}/refresh` still fetches
immediately. The poll loop reuses the same refresh path the API does, so there
is no second code path to keep correct.

Logging is via `RUST_LOG` (e.g. `RUST_LOG=feedd=debug`).

## `feedctl`

```
feedctl [--server URL] [--format text|json] <command>
```

| flag | default | meaning |
|---|---|---|
| `--server URL` | `http://127.0.0.1:8600` | Base URL of the feedd server. |
| `--format text\|json` | `text` | `json` prints the API response as a single JSON document on stdout. |

| command | does |
|---|---|
| `add URL` | Register a feed. |
| `list` | List registered feeds. |
| `show ID` | Show one feed. |
| `remove ID` | Unregister a feed and delete its entries. |
| `refresh [ID]` | Fetch now. With no ID, refreshes every feed. |
| `entries [--feed ID] [--since T] [--until T] [--search Q] [--limit N] [--offset N]` | Query stored entries. |

`--search` is the CLI spelling of the API's `q` parameter. `--since` / `--until`
take RFC 3339 instants in any offset.

### Exit codes

| code | meaning |
|---|---|
| `0` | Success. |
| `1` | The server answered with an error. Its message goes to stderr. |
| `2` | The server is unreachable, or the usage is invalid. |

The distinction between 1 and 2 is *whether the server answered at all*. A feed
URL that feedd rejects with a 422 is exit **1** — feedd answered, and said no.
A closed port is exit **2**.

`--format json` always emits exactly one valid JSON document. `remove` is the
one case where the API has no body to pass through (it returns `204`), so
feedctl synthesizes `{"status":"ok","id":N}`.

## `feedgen`

```
feedgen serve --dir DIR [--listen ADDR:PORT] [--no-conditional]
feedgen make-fixtures DIR
```

| flag | default | meaning |
|---|---|---|
| `--dir DIR` | *required* | Directory of feed files to serve. |
| `--listen ADDR:PORT` | `127.0.0.1:8601` | Address to listen on. Sits next to feedd's 8600. |
| `--no-conditional` | off | Ignore `If-None-Match` / `If-Modified-Since` and always answer `200`. |

`serve` reads from the filesystem, so edits to a file take effect on the next
request — which is how the tests simulate an upstream change.

Every response carries a content-derived `ETag` (FNV-1a over the file bytes, so
it is stable across restarts) and a `Last-Modified` from the file's mtime.
`feedgen` answers `304` to a matching `If-None-Match`, or to an
`If-Modified-Since` at or after the mtime.

`--no-conditional` exists for testing: a `304` short-circuits a client's parse
and dedupe path, so forcing a full `200` on unchanged content is the only way to
exercise it.

### Content types

| extension | content type |
|---|---|
| `.rss` | `application/rss+xml; charset=utf-8` |
| `.atom` | `application/atom+xml; charset=utf-8` |
| `.xml` | `application/xml; charset=utf-8` |
| `.md` | `text/markdown; charset=utf-8` |
| `.txt` | `text/plain; charset=utf-8` |
| anything else | `application/octet-stream` |

### The fixture corpus

`feedgen make-fixtures DIR` writes these, plus a `README.md` cataloging them:

| file | pins |
|---|---|
| `rss-basic.rss` | A well-formed RSS 2.0 feed. The third item has no `guid`, so identity falls back to `link`. |
| `atom-basic.atom` | A well-formed Atom feed: `rel="alternate"` link selection, a bare `<link>`, `summary`-vs-`content`, and `published`-vs-`updated` fallback. |
| `dates-edge.rss` | RFC 822 zone names (`EST`, `PDT`, `UT`), a numeric offset, a missing date, and an unparseable date. The last two must store `published_at: null`. |
| `cdata-entities.rss` | Entity unescaping (`&amp;` → `&`), CDATA taken verbatim, and an item with no title (stored as `(untitled)`). |
| `malformed.xml` | An unclosed `<item>`. Must set `last_error` on its feed and leave other feeds untouched. |

## API reference

All responses are JSON. Errors are `{"error": "<message>"}` with a 4xx/5xx
status — including malformed request bodies and bad query parameters.

### `GET /api/health`

`200`. Extra fields are allowed and present.

```json
{ "status": "ok", "version": "0.1.0" }
```

### `POST /api/feeds`

Register a feed. Body: `{"url": "https://example.com/feed.rss"}`.

| status | when |
|---|---|
| `201` | Created. Returns the feed object. |
| `409` | That URL is already registered. |
| `422` | The URL is not a valid `http`/`https` URL, or the body is malformed. |

### `GET /api/feeds`

`200`. An array of feed objects, ordered by `id`.

### `GET /api/feeds/{id}`

`200` with the feed object, or `404`.

### `DELETE /api/feeds/{id}`

`204` with no body. The feed's entries are deleted too. `404` if no such feed.

### `POST /api/feeds/{id}/refresh`

Fetch that feed now, regardless of the poll interval. `404` if no such feed.
Otherwise `200` with a refresh result — note that a *fetch* failure is still a
`200`, carrying `"status": "error"`; the 404 is only about the feed not
existing.

```json
{ "feed_id": 1, "status": "ok", "new_entries": 3, "updated_entries": 0, "not_modified": false }
```

```json
{ "feed_id": 2, "status": "error", "new_entries": 0, "error": "malformed XML: ..." }
```

| field | meaning |
|---|---|
| `feed_id` | The feed refreshed. |
| `status` | `"ok"` or `"error"`. |
| `new_entries` | Entries newly inserted. `0` for a 304, an error, or unchanged content. |
| `updated_entries` | Known entries **re-seen** and written back, not entries whose values actually changed. Re-fetching identical content reports every item here. Absent on error. |
| `not_modified` | `true` when the origin answered `304`. Absent on error. |
| `error` | Present only when `status` is `"error"`. |

### `POST /api/refresh`

Refresh every feed, sequentially. `200` with an array of the same result
objects, each including its `feed_id`. One feed's failure never truncates the
array or affects another feed's result.

### `GET /api/entries`

All parameters are optional.

| parameter | default | meaning |
|---|---|---|
| `feed_id` | — | Restrict to one feed. |
| `since` | — | RFC 3339 instant, any offset. Inclusive lower bound. |
| `until` | — | RFC 3339 instant, any offset. Exclusive upper bound. |
| `q` | — | Case-insensitive (ASCII) substring of the title. |
| `limit` | `50` | Max entries. Values above `500` are clamped to `500`. |
| `offset` | `0` | Entries to skip. |

Returns `200`:

```json
{ "total": 42, "items": [ /* entry objects */ ] }
```

`total` is the match count ignoring `limit`/`offset`. `422` for an unparseable
or negative value.

**Window semantics.** Half-open, `since <= published_at < until`, compared as
UTC instants. When either bound is given, entries with a null `published_at` are
excluded.

**Ordering.** `published_at` descending, nulls last, ties broken by ascending
internal entry id.

### Objects

**Feed**

| field | type | notes |
|---|---|---|
| `id` | integer | |
| `url` | string | |
| `title` | string \| null | Null until the first successful fetch; updates on refresh. |
| `last_fetched_at` | string \| null | RFC 3339 UTC. Set on every completed fetch attempt, including failures. |
| `last_error` | string \| null | Null when the most recent fetch succeeded. |
| `entry_count` | integer | |

**Entry**

| field | type | notes |
|---|---|---|
| `id` | integer | Stable across updates. |
| `feed_id` | integer | |
| `guid` | string | RSS `guid` (or `link`), Atom `id`. |
| `title` | string | `(untitled)` when the source had none. |
| `link` | string \| null | |
| `summary` | string \| null | |
| `published_at` | string \| null | RFC 3339 UTC with `Z`. Null when the source date was missing or unparseable. |
| `fetched_at` | string | RFC 3339 UTC, never null. The entry's *first* fetch; unchanged by later updates. |

## Behavior worth knowing

**Dates.** `published_at` is normalized to UTC and serialized RFC 3339 with `Z`.
RSS `pubDate` is parsed as RFC 822/1123, accepting 4-digit years (2- and 3-digit
are tolerated), numeric offsets (`+0000`, `-0500`), and the zone names
`GMT UT Z EST EDT CST CDT MST MDT PST PDT`. Atom dates are RFC 3339 with any
numeric offset and optional fractional seconds.

The RFC 822 parser is deliberately a little more tolerant than that list: it
also accepts `UTC`, treats a missing zone as GMT (RFC 2822 §4.3 reads an unknown
zone as `-0000`), and accepts an optional day-of-week which it does not check
against the date. Zone names are matched case-sensitively, as the specs write
them — `gmt` does not parse. `:60` is clamped to `:59`, identically on both
grammars, so a leap second can never make the stored text and the sort key
disagree.

Instants carry millisecond precision at most; anything finer is truncated.

A missing or unparseable date stores `published_at: null`. **Fetch time is never
substituted** — an undated entry is undated, and still stored.

Instants are stored at millisecond precision, alongside an integer epoch-ms sort
key. Ordering and window filters compare that integer, so they compare true
instants rather than depending on the text form sorting correctly. (It doesn't:
`'.'` sorts before `'Z'`, so `00:00:00.5Z` would compare *less than* `00:00:00Z`
despite being later.)

**Text.** Titles and summaries are stored after XML entity unescaping
(`&amp;` → `&`). HTML entities (`&nbsp;`, `&mdash;`, `&rsquo;`) resolve too:
strictly they are undefined in XML without a DTD, but they are pervasive in real
feeds, and rejecting them would discard the entire document — every entry — over
one character. CDATA is taken verbatim, so `&amp;` inside CDATA stays literal.
A missing title is stored as `(untitled)`.

Element matching is namespace-aware. Only elements in the feed's own namespace
supply fields, so a `<atom:link>` or `<media:description>` sitting inside an RSS
`<item>` is ignored rather than mistaken for the item's own `<link>` or
`<description>`. An empty element never claims a field either, so a placeholder
`<link/>` ahead of the real one is harmless.

**Dedupe.** Identity is `(feed, guid-or-id)`. Seeing a known key again updates
`title`, `link`, `summary`, and `published_at` in place — no duplicate row. The
entry keeps its internal `id` and its original `fetched_at`. The same guid in two
different feeds is two entries.

Items with no usable identity (an RSS item with neither `guid` nor `link`, or an
Atom entry with no `id`) are skipped — they have no stable dedupe key, so
storing them would duplicate on every refresh. Skips are logged at `WARN` rather
than swallowed, so a feed of identity-less items can't quietly look like an
empty one.

**Conditional GET.** feedd stores each feed's `ETag` and `Last-Modified` and
sends `If-None-Match` / `If-Modified-Since` on refetch. A `304` leaves entries
untouched, counts as a successful fetch (so it clears `last_error`), and reports
`new_entries: 0`.

**Failure isolation.** A fetch or parse failure — connection refused, HTTP
error, malformed XML — is recorded to that feed's `last_error` and goes no
further. Other feeds are unaffected, the feed keeps its existing title and
entries, and the server does not crash. Any subsequent successful fetch clears
`last_error`. A refresh that reports `"status": "error"` always leaves a
matching `last_error` on the feed row; the response and the row never disagree.

**Concurrency.** Refreshes of the same feed are not serialized against each
other, so a poll tick racing a manual refresh can apply responses out of order
and briefly store stale entry fields and validators. It self-corrects: the stale
validator draws a `200` on the next refresh, which re-applies. Counts stay
honest throughout, because the store's mutex serializes the two transactions and
identity dedupes them.

**Network.** feedd only ever fetches URLs you explicitly register, and only over
`http`/`https`. Requests carry a 30-second timeout and a 10-second connect
timeout, follow at most 5 redirects, and refuse bodies over 32 MiB. The test
suite fetches nothing but `feedgen` on loopback.

## Development

One command runs every gate:

```sh
just check      # or: ./check   (same four commands, no `just` needed)
```

Which is:

```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
cargo build --release
```

### Layout

```
crates/
├── feedhub-core/   types, RSS/Atom parsing, date + text normalization
├── feedd/          store, fetcher, REST API, poll loop   (lib + bin)
├── feedctl/        the CLI                               (lib + bin)
└── feedgen/        fixture corpus + static server        (lib + bin)
```

`feedd`, `feedctl`, and `feedgen` each carry a lib target alongside the binary,
and each `main.rs` is a thin shim over it. That is what lets the tests drive the
same code the binaries run, in-process, on ephemeral ports.

### Tests

`cargo test --workspace` runs everything, including an end-to-end suite that
drives a real `feedd` against a real `feedgen` over local HTTP. Both bind
`127.0.0.1:0` and the test reads the port before the server task spawns, so
there are no sleeps and no readiness polling anywhere in the suite.

The one thing worth knowing if you touch storage: `new_entries` uses
`INSERT OR IGNORE` followed by an `UPDATE`, *not* `INSERT ... ON CONFLICT DO
UPDATE` with `changes()`. SQLite reports one row changed for both branches of an
upsert, so a `changes()`-based count would silently report the feed's entire
item count as "new" on every refresh. `refetching_identical_content_without_a_304_still_adds_nothing`
in `crates/feedd/tests/e2e.rs` is the test that holds that line — the 304 path
cannot catch it, because a 304 never reaches the upsert at all.

## License

MIT.
