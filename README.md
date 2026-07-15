# loglens

A command-line tool for analyzing web server access logs in **Combined Log
Format (CLF)**.

```
203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"
```

`loglens` parses CLF access logs and reports summaries, top-N rankings, error
breakdowns, and hourly request histograms — as human-readable text or JSON.

## Requirements

- Python 3.12+
- [`uv`](https://docs.astral.sh/uv/) for environment management

## Install

Clone the repo and sync the environment:

```bash
uv sync
```

This installs the project and its console entry point, `loglens`. Run it via
`uv run`:

```bash
uv run loglens --help
```

Or install it into your environment / tool path:

```bash
uv tool install .
loglens --help
```

## Usage

Every subcommand takes a `LOGFILE` path. The global `--format {text,json}`
option (default `text`) selects the output format and must come **before** the
subcommand.

### `summary` — overall statistics

Total requests, unique client IPs, first/last timestamp, and error rate (the
percentage of 4xx and 5xx responses).

```bash
uv run loglens summary tests/fixtures/sample.log
```

```
Total requests:  32
Unique IPs:      6
First timestamp: 2026-07-12T06:25:24+00:00
Last timestamp:  2026-07-12T16:31:38+00:00
Error responses: 8
Error rate:      25.00%
```

### `top` — top values by request count

Rank the top N (default 10) IPs, paths, or statuses by request count, descending.
Ties are broken by value ascending.

```bash
uv run loglens top tests/fixtures/sample.log --by path -n 5
uv run loglens top tests/fixtures/sample.log --by ip
uv run loglens top tests/fixtures/sample.log --by status
```

```
/api/orders  4
/dashboard   4
/index.html  4
/products    3
/api/cart    2
```

### `errors` — 4xx/5xx requests grouped by (status, path)

Most frequent first. Optionally restrict to a time window with `--since` and
`--until` (ISO 8601; a value without a timezone offset is treated as UTC).

```bash
uv run loglens errors tests/fixtures/sample.log
uv run loglens errors tests/fixtures/sample.log --since 2026-07-12T10:00:00+00:00
uv run loglens errors tests/fixtures/sample.log --until 2026-07-12T12:00:00
```

```
404  /missing           2
401  /api/cart          1
404  /favicon.ico       1
404  /products/42       1
500  /api/cart          1
500  /search?q=gadgets  1
503  /api/stats         1
```

The `--since`/`--until` window is inclusive; `--since 2026-07-12T10:00:00+00:00`
keeps only errors at or after 10:00 UTC.

### `hourly` — requests per hour of day

A text histogram of request counts bucketed by hour (00–23).

```bash
uv run loglens hourly tests/fixtures/sample.log
```

```
06  ########################                  3
07  ################################          4
08  ########################                  3
...
11  ########################################  5
...
```

### JSON output

Pass `--format json` before any subcommand to emit a single JSON document on
stdout:

```bash
uv run loglens --format json summary tests/fixtures/sample.log
uv run loglens --format json top tests/fixtures/sample.log --by path
```

## Behavior & exit codes

- **Malformed lines** are skipped and counted; the count is reported on
  **stderr**, never on stdout. Blank lines are ignored.
- Exit codes:
  - `0` — success
  - `1` — the file contains no valid log lines
  - `2` — `LOGFILE` is missing or unreadable (or bad arguments)

Because malformed-line reports go to stderr, `--format json` output on stdout is
always a single valid JSON document safe to pipe into `jq`.

## Development

Common tasks are wired into the [`justfile`](./justfile):

```bash
just check   # ruff lint + pytest (the CI gates)
just test    # pytest only
just lint    # ruff check only
```

Or run the tools directly:

```bash
uv run ruff check .
uv run pytest
```

## License

MIT
