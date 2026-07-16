# loglens

`loglens` is a dependency-free Python CLI that analyzes web server access logs in
**Combined Log Format (CLF)** — the format emitted by Apache and nginx.

Example log lines it understands:

```
203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"
198.51.100.22 - alice [12/Jul/2026:07:01:02 +0000] "POST /api/orders HTTP/1.1" 201 512 "-" "curl/8.5.0"
192.0.2.9 - - [12/Jul/2026:07:15:44 +0000] "GET /missing HTTP/1.1" 404 153 "-" "Mozilla/5.0"
```

## Requirements

- Python 3.12+
- [`uv`](https://docs.astral.sh/uv/) for environment and dependency management

## Install

```bash
# From the project root
uv sync

# Run without installing globally
uv run loglens --help

# Or install the console script into the environment
uv pip install -e .
loglens --help
```

## Usage

All subcommands take a `LOGFILE` path. The global `--format {text,json}` option
(default `text`) may appear before or after the subcommand. JSON output is always
a single valid JSON document printed to stdout.

### `summary` — overall statistics

Total requests, unique client IPs, first/last timestamp, and error rate (percentage
of 4xx and 5xx responses).

```bash
uv run loglens summary tests/fixtures/sample.log
```

```
Total requests:    30
Unique client IPs: 6
First timestamp:   2026-07-12T06:25:24+00:00
Last timestamp:    2026-07-12T13:41:37+00:00
Error rate:        33.33%
```

(Two malformed lines in the sample are skipped; `loglens: skipped 2 malformed lines`
is printed to stderr.)

### `top` — most frequent values

Top N values by request count (descending; ties broken by value ascending).
`-n` defaults to 10.

```bash
uv run loglens top tests/fixtures/sample.log --by ip -n 3
uv run loglens top tests/fixtures/sample.log --by path
uv run loglens top tests/fixtures/sample.log --by status
```

```
7  203.0.113.7
5  192.0.2.9
5  198.51.100.22
```

### `errors` — 4xx/5xx grouped by status and path

Grouped by `(status, path)` with counts, most frequent first. Optionally bounded by
`--since` (inclusive) and `--until` (exclusive) ISO 8601 timestamps.

```bash
uv run loglens errors tests/fixtures/sample.log
uv run loglens errors tests/fixtures/sample.log \
  --since 2026-07-12T09:00:00+00:00 --until 2026-07-12T10:00:00+00:00
```

```
2  404  /missing
2  500  /api/stats
1  401  /api/orders
1  403  /admin
1  403  /admin/users
1  404  /favicon.ico
1  500  /checkout
1  502  /checkout
```

### `hourly` — requests per hour of day

Request count for each hour `00`–`23`, rendered as a text histogram scaled to the
busiest hour.

```bash
uv run loglens hourly tests/fixtures/sample.log
```

```
06  #################                        3
07  #############################            5
08  ######################################## 7
09  #######################                  4
...
```

### JSON output

```bash
uv run loglens --format json summary tests/fixtures/sample.log
```

```json
{"total_requests": 30, "unique_ips": 7, "first_timestamp": "2026-07-12T06:25:24+00:00", "last_timestamp": "2026-07-12T13:41:37+00:00", "error_rate": 33.33}
```

## Error handling

- Malformed lines are skipped and counted; the count is reported on **stderr**,
  never on stdout (so JSON output stays clean).
- Exit codes:
  - `0` — success
  - `1` — the file contains no valid log lines
  - `2` — `LOGFILE` is missing or unreadable

## Development

Run the full check suite (lint + tests) with a single command:

```bash
just check
```

Equivalent without `just`:

```bash
uv run ruff check . && uv run pytest
```
