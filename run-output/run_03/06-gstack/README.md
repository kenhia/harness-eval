# loglens

A fast, lightweight CLI tool for analyzing web server access logs in Combined Log Format (CLF).

## Installation

### Prerequisites

- Python 3.12 or later
- [uv](https://github.com/astral-sh/uv) package manager

### Install from source

```bash
# Clone or download the repository
cd loglens

# Create a virtual environment and install
uv venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate
uv pip install -e .

# Verify installation
loglens --help
```

### Alternative: Install with pip

```bash
pip install -e .
loglens --help
```

## Usage

loglens provides four subcommands for analyzing access logs:

### Summary

Show overview statistics: total requests, unique IPs, time range, and error rate.

```bash
loglens summary /var/log/apache2/access.log
```

**Text output:**
```
Total Requests: 1000
Unique IPs: 42
First Timestamp: 2026-07-12T00:15:24+00:00
Last Timestamp: 2026-07-12T23:45:44+00:00
Error Rate: 5.2%
```

**JSON output:**
```bash
loglens --format json summary /var/log/apache2/access.log
```

```json
{
  "total_requests": 1000,
  "unique_ips": 42,
  "first_timestamp": "2026-07-12T00:15:24+00:00",
  "last_timestamp": "2026-07-12T23:45:44+00:00",
  "error_rate_percent": 5.2
}
```

### Top

Show the top N most frequent values by request count. Supports ranking by IP, path, or HTTP status.

```bash
# Top 10 IPs (default)
loglens top /var/log/apache2/access.log

# Top 5 paths
loglens top --by path -n 5 /var/log/apache2/access.log

# Top 10 status codes
loglens top --by status /var/log/apache2/access.log
```

**Text output:**
```
Top IP:
 1. 203.0.113.7                           234
 2. 198.51.100.22                         167
 3. 192.0.2.9                             145
```

**JSON output:**
```bash
loglens --format json top --by path -n 5 /var/log/apache2/access.log
```

```json
{
  "by": "path",
  "items": [
    {"value": "/api/users", "count": 234},
    {"value": "/api/orders", "count": 189},
    {"value": "/index.html", "count": 145}
  ]
}
```

### Errors

Show 4xx and 5xx error responses grouped by status code and path. Supports optional date filtering.

```bash
# Show all errors
loglens errors /var/log/apache2/access.log

# Filter errors within a date range (ISO8601 format)
loglens errors --since 2026-07-12T12:00:00+00:00 --until 2026-07-12T18:00:00+00:00 /var/log/apache2/access.log
```

**Text output:**
```
STATUS  PATH                                 COUNT
--------------------------------------------------
404     /old-page                              12
404     /missing-resource                       8
500     /api/data                               5
403     /admin                                  3
```

**JSON output:**
```bash
loglens --format json errors /var/log/apache2/access.log
```

```json
{
  "errors": [
    {"status": 404, "path": "/old-page", "count": 12},
    {"status": 404, "path": "/missing-resource", "count": 8},
    {"status": 500, "path": "/api/data", "count": 5}
  ]
}
```

### Hourly

Show request distribution across hours of the day (00–23) as a histogram.

```bash
loglens hourly /var/log/apache2/access.log
```

**Text output:**
```
00:00 │ ████ 12
01:00 │ ██ 6
02:00 │ ████████ 24
...
23:00 │ █████ 15
```

**JSON output:**
```bash
loglens --format json hourly /var/log/apache2/access.log
```

```json
{
  "hourly": [
    {"hour": 0, "count": 12},
    {"hour": 1, "count": 6},
    {"hour": 2, "count": 24},
    ...
    {"hour": 23, "count": 15}
  ]
}
```

## Global Options

### Format

Control output format with `--format`:

```bash
loglens --format text summary access.log     # Human-readable (default)
loglens --format json summary access.log     # Valid JSON
```

## Log Format

loglens parses the Combined Log Format (CLF), the standard Apache/Nginx access log format:

```
IP - USER [TIMESTAMP] "METHOD PATH HTTP/VERSION" STATUS BYTES "REFERER" "USER-AGENT"
```

Example:
```
203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"
```

## Error Handling

- **Malformed lines:** Skipped silently. Count of malformed lines reported to stderr.
- **Missing file:** Error message, exit code 2.
- **No valid lines:** Error message, exit code 1.
- **Success:** Exit code 0.

Example:
```bash
$ loglens summary /nonexistent.log
Error: File '/nonexistent.log' not found or unreadable
$ echo $?
2
```

## Development

### Running tests

```bash
# Run all tests
pytest

# Run with verbose output
pytest -v

# Run a specific test file
pytest tests/test_parser.py
```

### Code quality

```bash
# Check code style with ruff
ruff check src/ tests/

# Format code
ruff format src/ tests/

# Run all checks (lint + test)
just check
```

### Project structure

```
loglens/
├── pyproject.toml          # Project metadata and dependencies
├── README.md               # This file
├── Justfile                # Task runner
├── src/
│   └── loglens/
│       ├── __init__.py
│       ├── cli.py          # Click CLI entry point
│       ├── parser.py       # CLF parser and LogEntry model
│       ├── analyzers/      # Analysis modules (summary, top, errors, hourly)
│       └── formatters/     # Output formatters (text, JSON)
└── tests/
    ├── fixtures/
    │   └── sample.log      # Sample log file for testing
    └── test_*.py           # Test suites
```

## License

MIT
