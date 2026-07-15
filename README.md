# loglens

`loglens` is a Python command-line tool that analyzes web server access logs in
**Combined Log Format (CLF)** — the format emitted by Apache, nginx and most web
servers.

An example of the log lines it understands:

```
203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"
198.51.100.22 - alice [12/Jul/2026:07:01:02 +0000] "POST /api/orders HTTP/1.1" 201 512 "-" "curl/8.5.0"
192.0.2.9 - - [12/Jul/2026:07:15:44 +0000] "GET /missing HTTP/1.1" 404 153 "-" "Mozilla/5.0"
```

## Requirements

- Python 3.12+
- [`uv`](https://docs.astral.sh/uv/) for environment and dependency management

## Install

Clone the repository and let `uv` create the environment and install the project:

```bash
uv sync
```

This installs `loglens` into the project virtual environment. Run it via `uv run`:

```bash
uv run loglens --help
```

To install the console script onto your `PATH` as a standalone tool:

```bash
uv tool install .
loglens --help
```

## Usage

Every subcommand takes a `LOGFILE` path. The global `--format {text,json}` option
(default `text`) selects the output format; in `json` mode a single valid JSON
document is written to stdout.

```bash
loglens [--format {text,json}] <command> LOGFILE [options]
```

### `summary`

Total requests, unique client IPs, first and last timestamp, and the error rate
(percentage of 4xx + 5xx responses).

```bash
loglens summary tests/fixtures/sample.log
loglens --format json summary tests/fixtures/sample.log
```

### `top`

Top `N` values by request count, descending. Ties are broken by value ascending.
`--by` chooses the dimension; `-n/--number` sets how many rows (default 10).

```bash
loglens top tests/fixtures/sample.log --by ip
loglens top tests/fixtures/sample.log --by path -n 5
loglens top tests/fixtures/sample.log --by status
```

### `errors`

All 4xx/5xx requests grouped by `(status, path)` with counts, most frequent first.
Optionally restrict to a time window with `--since` / `--until` (ISO 8601).

```bash
loglens errors tests/fixtures/sample.log
loglens errors tests/fixtures/sample.log --since 2026-07-12T07:00:00+00:00
loglens errors tests/fixtures/sample.log --since 2026-07-12T07:00:00 --until 2026-07-12T08:00:00
```

### `hourly`

Request count per hour of day (`00`–`23`), rendered as a text histogram (or a
JSON object of hour → count under `--format json`).

```bash
loglens hourly tests/fixtures/sample.log
loglens --format json hourly tests/fixtures/sample.log
```

## Error handling & exit codes

- **Malformed lines** are skipped and counted. The count is reported on **stderr**
  (never on stdout), so piping stdout to a JSON parser stays clean.
- Exit codes:
  - `0` — success
  - `1` — the file contains no valid log lines
  - `2` — `LOGFILE` is missing or unreadable

## Development

All checks (lint + tests) run with a single command:

```bash
just check
```

Individual tasks:

```bash
just lint    # ruff check .
just test    # pytest
just fix     # ruff check --fix .
```

Without `just` installed, the equivalent is:

```bash
uv run ruff check . && uv run pytest
```
