# loglens — Web Access Log Analyzer

A Python CLI tool that analyzes web server access logs in [Combined Log Format (CLF)](https://httpd.apache.org/docs/2.4/logs.html#combined).

## Installation

Requires Python 3.12+.

Using `uv` (recommended):
```bash
uv sync
uv run loglens --help
```

Or after installing with `uv sync`, activate the environment:
```bash
source .venv/bin/activate
loglens --help
```

## Quick Start

Analyze a sample log file:
```bash
loglens summary tests/fixtures/sample.log
```

## Subcommands

### summary
Print summary statistics: total requests, unique client IPs, first/last timestamps, error rate.

```bash
loglens summary LOGFILE
loglens --format json summary LOGFILE
```

**Text output example:**
```
Total requests: 30
Unique IPs: 4
First timestamp: 2026-07-12 06:25:24
Last timestamp: 2026-07-12 21:35:44
Error rate: 20.00%
```

**JSON output example:**
```json
{
  "total_requests": 30,
  "unique_ips": 4,
  "first_timestamp": "2026-07-12T06:25:24",
  "last_timestamp": "2026-07-12T21:35:44",
  "error_rate": 20.0
}
```

### top
Show top N values grouped by IP, request path, or HTTP status.

```bash
loglens top LOGFILE --by {ip|path|status} [-n N]
loglens --format json top LOGFILE --by ip -n 5
```

**Text output example (top 10 IPs):**
```
203.0.113.7: 11
198.51.100.22: 10
192.0.2.9: 6
192.0.2.1: 3
```

**JSON output example:**
```json
{
  "ip": [
    {"value": "203.0.113.7", "count": 11},
    {"value": "198.51.100.22", "count": 10}
  ]
}
```

### errors
List 4xx and 5xx errors grouped by status code and request path, sorted by frequency.

Optional filters: `--since` and `--until` (ISO8601 timestamps).

```bash
loglens errors LOGFILE
loglens errors LOGFILE --since 2026-07-12T14:00:00
loglens errors LOGFILE --until 2026-07-12T20:00:00
loglens --format json errors LOGFILE
```

**Text output example:**
```
404 /missing: 4
500 /api/data: 2
403 /admin: 2
401 /api/login: 1
```

**JSON output example:**
```json
{
  "errors": [
    {"status": 404, "path": "/missing", "count": 4},
    {"status": 500, "path": "/api/data", "count": 2}
  ]
}
```

### hourly
Show request distribution by hour of day (00–23) as a text histogram or JSON.

```bash
loglens hourly LOGFILE
loglens --format json hourly LOGFILE
```

**Text output example:**
```
00: ███████ (8)
01: 
02: 
03: 
04: 
05: 
06: ███████ (8)
07: ███████ (8)
08: ███████ (8)
09: ███████ (8)
10: ███ (3)
11: ███ (3)
12: ███ (3)
13: ███ (3)
14: ███ (3)
15: ███ (3)
16: ███ (3)
17: ███ (3)
18: ███ (3)
19: ███ (3)
20: ███ (3)
21: ███ (3)
22: 
23: 
```

**JSON output example:**
```json
{
  "hourly": [
    {"hour": 0, "count": 8},
    {"hour": 1, "count": 0},
    ...
    {"hour": 23, "count": 0}
  ]
}
```

## Global Options

- `--format {text,json}` — Output format (default: `text`). JSON output is valid JSON on stdout.

## Error Handling

- Malformed log lines are silently skipped; the count is reported on stderr.
- **Exit codes:**
  - `0` — Success
  - `1` — No valid log lines found
  - `2` — Log file not found or unreadable

## Testing

Run the test suite:
```bash
source .venv/bin/activate
pytest tests/ -v
```

Check code style (ruff):
```bash
ruff check .
```

Run all checks (tests + lint):
```bash
pytest tests/ -q && ruff check . && loglens --help
```

## Development

Install dev dependencies:
```bash
uv sync
```

Run tests:
```bash
uv run pytest tests/ -v
```

Format and lint:
```bash
uv run ruff check . --fix
```
