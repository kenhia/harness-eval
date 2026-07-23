# loglens

A Python command-line tool for analyzing web server access logs in Combined Log Format (CLF).

## Installation

### Prerequisites

- Python 3.12 or higher
- `uv` package manager

### Using uv

```bash
uv pip install -e .
```

Or, to install with development dependencies:

```bash
uv pip install -e ".[dev]"
```

## Usage

`loglens` provides four subcommands for analyzing web access logs.

### Summary

Display overall statistics for a log file:

```bash
loglens summary access.log
```

Output includes:
- Total number of requests
- Number of unique client IPs
- First and last timestamps
- Error rate (percentage of 4xx and 5xx responses)

With JSON output:
```bash
loglens summary access.log --format json
```

### Top

Show the top N values by request count for a given metric:

```bash
loglens top access.log --by ip -n 10
```

Group by one of:
- `ip` — client IP addresses (default)
- `path` — requested paths
- `status` — HTTP status codes

Options:
- `-n N` — show top N results (default: 10)
- `--format {text,json}` — output format (default: text)

Examples:
```bash
loglens top access.log --by path -n 5
loglens top access.log --by status --format json
```

### Errors

Show all 4xx and 5xx errors grouped by (status, path):

```bash
loglens errors access.log
```

Filter by timestamp with:
```bash
loglens errors access.log --since 2026-07-12T10:00:00 --until 2026-07-12T20:00:00
```

Both `--since` and `--until` accept ISO8601 timestamps.

With JSON output:
```bash
loglens errors access.log --format json
```

### Hourly

Show request distribution across hours of the day (00–23) as a text histogram:

```bash
loglens hourly access.log
```

Example output:
```
00:00 ███ 42
01:00 ██ 28
02:00 █████████ 120
...
```

With JSON output:
```bash
loglens hourly access.log --format json
```

## Log Format

`loglens` expects logs in Combined Log Format (CLF):

```
IP - USER [TIMESTAMP] "METHOD PATH PROTOCOL" STATUS SIZE "REFERRER" "USER_AGENT"
```

Example:
```
203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"
198.51.100.22 - alice [12/Jul/2026:07:01:02 +0000] "POST /api/orders HTTP/1.1" 201 512 "-" "curl/8.5.0"
```

## Error Handling

- Malformed log lines are skipped; a count of skipped lines is reported to stderr
- Exit codes:
  - `0`: Success
  - `1`: File contains no valid log lines
  - `2`: File is missing or unreadable

## Development

### Running Tests

```bash
pytest
```

### Linting

```bash
ruff check .
```

### Running All Checks

```bash
just check
```

Or without `just`:

```bash
pytest && ruff check .
```
