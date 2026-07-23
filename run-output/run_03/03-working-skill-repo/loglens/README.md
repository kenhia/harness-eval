# loglens - Web Access Log Analyzer

A Python command-line tool for analyzing web server access logs in Combined Log Format (CLF).

## Installation

Install with `uv`:

```bash
uv pip install -e .
```

Or with pip:

```bash
pip install -e .
```

The `loglens` command will be available in your environment.

## Subcommands

### summary

Display summary statistics for a log file: total requests, unique client IPs, first and last timestamps, and error rate (percentage of 4xx and 5xx responses).

```bash
loglens summary access.log
```

Output:
```
Total Requests: 1523
Unique IPs: 127
First Timestamp: 2026-07-12T06:25:24+00:00
Last Timestamp: 2026-07-12T13:40:22+00:00
Error Rate: 12.45%
```

### top

Show the top N values by request count. Values can be grouped by IP, path, or status code.

```bash
loglens top access.log --by ip -n 10
loglens top access.log --by path -n 5
loglens top access.log --by status
```

Output:
```
Top ip:
  1. 203.0.113.7: 45
  2. 192.0.2.9: 32
  3. 198.51.100.22: 28
```

### errors

Display all 4xx and 5xx errors grouped by status code and path, sorted by frequency (most frequent first). Supports optional date range filtering.

```bash
loglens errors access.log
loglens errors access.log --since 2026-07-12T06:00:00+00:00 --until 2026-07-12T12:00:00+00:00
```

Output:
```
Errors (status, path):
  404 /missing.html: 12
  500 /api/error: 5
  403 /forbidden: 3
```

### hourly

Display request count per hour of day (00–23) as a text histogram.

```bash
loglens hourly access.log
```

Output:
```
Requests per Hour:
  00: ██████ (12)
  01: ████ (8)
  02: ██ (4)
  ...
  23: ██████████ (20)
```

## Global Options

### --format

Choose output format (default: `text`).

```bash
loglens summary access.log --format json
loglens top access.log --by ip --format json
```

JSON output is a single valid JSON document on stdout:

```json
{
  "total_requests": 1523,
  "unique_ips": 127,
  "first_timestamp": "2026-07-12T06:25:24+00:00",
  "last_timestamp": "2026-07-12T13:40:22+00:00",
  "error_rate": 12.45
}
```

## Error Handling

- **Malformed log lines** are skipped and the count is reported on stderr
- **Missing or unreadable file** exits with code 2
- **No valid log lines** exits with code 1
- **Success** exits with code 0

Example:
```bash
$ loglens summary bad-log.log
Skipped 2 malformed lines
Total Requests: 245
...
```

## Development

### Run Tests

```bash
uv run pytest tests -v
```

With coverage:
```bash
uv run pytest tests --cov=loglens --cov-report=term-missing
```

### Lint

```bash
uv run ruff check loglens tests
```

### Run All Checks

Use the justfile:
```bash
just check
```

Or manually:
```bash
uv run ruff check loglens tests && uv run pytest tests -v
```

## Project Structure

```
loglens/
├── loglens/
│   ├── __init__.py        # Package root
│   ├── parser.py          # CLF log parsing
│   ├── commands.py        # Subcommand implementations
│   └── cli.py             # Click CLI and entry point
├── tests/
│   ├── fixtures/
│   │   └── sample.log     # Test data
│   └── test_loglens.py    # Comprehensive test suite
├── pyproject.toml         # Project metadata and dependencies
├── justfile               # Check commands
└── README.md              # This file
```

## Example Workflow

Parse a log file and analyze it:

```bash
# Get summary
loglens summary /var/log/nginx/access.log

# Find top IPs
loglens top /var/log/nginx/access.log --by ip -n 20

# Investigate errors
loglens errors /var/log/nginx/access.log

# See traffic distribution
loglens hourly /var/log/nginx/access.log

# Export to JSON for further processing
loglens summary /var/log/nginx/access.log --format json | jq .
```
