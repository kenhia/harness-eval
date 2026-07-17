# loglens

`loglens` is a Python command-line tool that analyzes web server access logs in
**Combined Log Format (CLF)**.

## Requirements

- Python 3.12+
- [`uv`](https://docs.astral.sh/uv/) for environment and dependency management

## Install

Clone the repository and sync the environment with `uv`:

```bash
uv sync
```

This installs the project (with dev dependencies) into a local virtual
environment. The `loglens` console entry point is then available via `uv run`:

```bash
uv run loglens --help
```

To install the tool onto your `PATH` as a standalone command:

```bash
uv tool install .
loglens --help
```

## Log format

`loglens` expects lines in Combined Log Format:

```
203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"
```

Malformed lines are skipped and counted; the skipped count is reported on
**stderr** and never mixed into the primary output on stdout.

## Global options

- `--format {text,json}` (default `text`) — selects the output format. In JSON
  mode, each command prints a **single valid JSON document** to stdout.

## Subcommands

Every subcommand takes a `LOGFILE` path.

### `summary`

Total requests, unique client IPs, first/last timestamps and the error rate
(percentage of 4xx and 5xx responses).

```bash
uv run loglens summary tests/fixtures/sample.log
```

```
Total requests: 32
Unique client IPs: 4
First timestamp: 2026-07-12T06:25:24+00:00
Last timestamp: 2026-07-12T18:01:49+00:00
Error rate: 37.50%
```

### `top`

Top N values by request count, descending; ties broken by value ascending.

```bash
uv run loglens top tests/fixtures/sample.log --by path -n 5
uv run loglens top tests/fixtures/sample.log --by ip
uv run loglens top tests/fixtures/sample.log --by status
```

`--by` is required and is one of `ip`, `path` or `status`. `-n` defaults to 10.

### `errors`

4xx/5xx requests grouped by `(status, path)` with counts, most frequent first.
Optional `--since` / `--until` ISO 8601 bounds filter by request time.

```bash
uv run loglens errors tests/fixtures/sample.log
uv run loglens errors tests/fixtures/sample.log --since 2026-07-12T14:00:00+00:00
```

### `hourly`

Request count per hour of day (00–23), rendered as a text histogram.

```bash
uv run loglens hourly tests/fixtures/sample.log
```

JSON mode emits an object keyed by zero-padded hour:

```bash
uv run loglens --format json hourly tests/fixtures/sample.log
```

## Exit codes

| Code | Meaning                                   |
|------|-------------------------------------------|
| 0    | Success                                   |
| 1    | The file contains no valid log lines      |
| 2    | `LOGFILE` is missing or unreadable        |

## Development

Run the full check suite (lint + tests) with a single command:

```bash
just check
```

If you don't have [`just`](https://github.com/casey/just), the equivalent is:

```bash
uv run ruff check .
uv run pytest -q
```
