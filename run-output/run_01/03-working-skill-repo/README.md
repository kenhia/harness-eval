# loglens

`loglens` is a Python command-line tool that analyzes web server access logs in
**Combined Log Format (CLF)** — the format emitted by Apache, nginx, and most
HTTP servers.

Example log lines:

```text
203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"
198.51.100.22 - alice [12/Jul/2026:07:01:02 +0000] "POST /api/orders HTTP/1.1" 201 512 "-" "curl/8.5.0"
192.0.2.9 - - [12/Jul/2026:07:15:44 +0000] "GET /missing HTTP/1.1" 404 153 "-" "Mozilla/5.0"
```

## Requirements

- Python 3.12+
- [uv](https://docs.astral.sh/uv/) for environment and dependency management

## Install

Clone the repository, then create the environment and install the project
(including the `loglens` console entry point) with `uv`:

```bash
uv sync
```

Run the tool through uv:

```bash
uv run loglens --help
```

Or activate the environment and call it directly:

```bash
source .venv/bin/activate
loglens --help
```

## Usage

Every subcommand takes a `LOGFILE` path. A global `--format {text,json}` option
(default `text`) may appear before or after the subcommand. In JSON mode, output
is a single valid JSON document on stdout.

```
loglens [--format {text,json}] <command> LOGFILE [options]
```

### `summary` — overall statistics

Total requests, unique client IPs, first/last timestamp, and error rate
(percentage of 4xx + 5xx responses).

```bash
uv run loglens summary tests/fixtures/sample.log
```

```text
Total requests:  32
Unique IPs:      8
First timestamp: 2026-07-12T06:25:24+00:00
Last timestamp:  2026-07-12T15:40:10+00:00
Error rate:      31.25% (10 errors)
```

JSON:

```bash
uv run loglens --format json summary tests/fixtures/sample.log
```

### `top` — most frequent values

Top `N` values by request count (default `N=10`), descending; ties broken by
value ascending. Choose the dimension with `--by {ip,path,status}`.

```bash
uv run loglens top tests/fixtures/sample.log --by ip -n 5
uv run loglens top tests/fixtures/sample.log --by path
uv run loglens top tests/fixtures/sample.log --by status --format json
```

### `errors` — 4xx/5xx breakdown

Error requests grouped by `(status, path)` with counts, most frequent first.
Optionally bound the window with ISO-8601 `--since` / `--until` (inclusive).

```bash
uv run loglens errors tests/fixtures/sample.log
uv run loglens errors tests/fixtures/sample.log --since 2026-07-12T09:00:00+00:00 --until 2026-07-12T10:00:00+00:00
uv run loglens errors tests/fixtures/sample.log --format json
```

### `hourly` — requests per hour of day

Request count per hour of day (00–23) rendered as a text histogram (or a JSON
object keyed by hour).

```bash
uv run loglens hourly tests/fixtures/sample.log
```

```text
06 #################                        3
07 #######################                  4
...
```

## Error handling

- **Malformed lines** are skipped and counted; the count is reported on
  **stderr**, never on stdout.
- **Exit codes:**
  - `0` — success
  - `1` — the file contains no valid log lines
  - `2` — the `LOGFILE` is missing or unreadable

## Development

All checks (lint + tests) run with a single command:

```bash
just check
```

Equivalent without `just`:

```bash
uv run ruff check .
uv run pytest
```

Other recipes:

```bash
just install   # uv sync
just lint      # uv run ruff check .
just test      # uv run pytest
```
