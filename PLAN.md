<!-- /autoplan restore point: /home/ken/src/ai-agents/harness-eval/_eval/profiles/claude-gstack/.gstack/projects/06-gstack/main-autoplan-restore-20260723-044827.md -->
# loglens — Web Access Log Analysis CLI

## Project Vision

Build a Python command-line tool (`loglens`) that analyzes web server access logs in Combined Log Format (CLF). The tool provides quick insights into traffic patterns, error rates, and request distribution through four subcommands.

**Target audience:** DevOps engineers, system operators, developers debugging server logs.

**Success metric:** A single installable CLI with four working subcommands, comprehensive test coverage, clean code (ruff pass), and complete documentation covering install and usage.

## Core Requirements

### 1. Language & Package Management
- Python 3.12+ (enforce via pyproject.toml python field)
- `uv` for package management (pyproject.toml + uv.lock)
- Installable console entry point: `loglens`
- Code must be `ruff check` clean (linting, formatting)
- Tests via `pytest`

### 2. Log Format (Combined Log Format / CLF)
Parse lines like:
```
203.0.113.7 - - [12/Jul/2026:06:25:24 +0000] "GET /index.html HTTP/1.1" 200 5413 "https://example.com/" "Mozilla/5.0"
```

Fields: IP, ident (ignored), user (ignored), timestamp, method, path, HTTP version, status, bytes, referer, user-agent.

### 3. Subcommands

#### `summary LOGFILE`
- Total request count
- Unique client IPs
- First timestamp (earliest)
- Last timestamp (latest)
- Error rate: percentage of 4xx + 5xx responses

#### `top LOGFILE --by {ip|path|status} [-n N]`
- Top N values (default 10) by request count
- Descending by count
- Ties broken by value ascending (stable, deterministic)

#### `errors LOGFILE [--since ISO8601] [--until ISO8601]`
- 4xx and 5xx requests only
- Grouped by (status, path) with counts
- Sorted by frequency (descending)
- Optional date range filtering (ISO8601 timestamps)

#### `hourly LOGFILE`
- Request count per hour of day (00–23)
- Rendered as text histogram (ASCII bar chart)

### 4. Global Options
- `--format {text|json}` (default `text`)
- JSON output: single valid JSON document on stdout
- Text output: human-readable format

### 5. Error Handling & Exit Codes

**Malformed lines:**
- Skip silently during processing
- Count and report to stderr only
- Do NOT include in stdout output

**Exit codes:**
- `0`: Success
- `1`: File contains no valid log lines
- `2`: LOGFILE missing or unreadable

## Implementation Strategy

### Phase 1: Foundation (Core CLI & Parsing)
1. Create `pyproject.toml` with Python 3.12+ requirement
2. Implement log parser (regex-based, robust to malformed lines)
3. Define data model (LogEntry namedtuple or dataclass)
4. Set up Click CLI framework with base command and subcommands
5. Implement `--format` global option

### Phase 2: Subcommands
1. `summary` — aggregate stats
2. `top` — frequency counting with sorting
3. `errors` — filtering + grouping by (status, path)
4. `hourly` — histogram generation

### Phase 3: Output Formatting
1. Text renderers for each subcommand
2. JSON serializers (ensure valid single-document output)
3. Consistent formatting across subcommands

### Phase 4: Testing & Quality
1. Create sample log fixture with:
   - 30+ lines minimum
   - Multiple IPs, paths, statuses (2xx/3xx/4xx/5xx)
   - Coverage of all 24 hours
   - 2+ malformed lines for error handling
2. Unit tests for parser (edge cases: missing fields, wrong formats)
3. Integration tests for each subcommand
4. Fixture and output format validation
5. ruff check pass (lint + format)

### Phase 5: Documentation & Install
1. README.md with:
   - Install instructions (via `uv`)
   - Usage examples for all four subcommands
   - Output format examples (text + JSON)
2. Ensure console entry point works
3. Test install via `pip install -e .` or `uv pip install -e .`

### Phase 6: Definition of Done
1. All subcommands working per spec
2. `pytest` passes (all tests green)
3. `ruff check` is clean
4. Sample log fixture in `tests/fixtures/sample.log`
5. README.md with full documentation
6. Meaningful git commits (one per feature/phase)
7. Single check command (e.g., `just check` or `make check`)

## File Structure

```
loglens/
├── pyproject.toml
├── uv.lock
├── README.md
├── Justfile (or Makefile)
├── src/
│   └── loglens/
│       ├── __init__.py
│       ├── cli.py            (main Click CLI entry point)
│       ├── parser.py         (log parsing logic)
│       ├── analyzers/
│       │   ├── __init__.py
│       │   ├── summary.py
│       │   ├── top.py
│       │   ├── errors.py
│       │   └── hourly.py
│       └── formatters/
│           ├── __init__.py
│           ├── text.py
│           └── json.py
└── tests/
    ├── fixtures/
    │   └── sample.log
    └── test_*.py
```

## Key Design Decisions

### 1. Parser Robustness
- Regex-based parsing with fallback for malformed lines
- Track malformed line count for stderr reporting
- No exceptions during normal parsing — graceful degradation

### 2. CLI Framework
- Use Click (standard Python CLI library)
- Single main group with four subcommands
- Global `--format` option applied uniformly

### 3. Output Consistency
- All text output follows Unix conventions (one item per line, no headers for streaming)
- JSON output: always a single valid JSON object/array at top level
- Subcommand-specific JSON structures (summary → object, top/errors/hourly → arrays or objects as appropriate)

### 4. Testing Strategy
- Parser unit tests (edge cases, malformed input)
- Integration tests per subcommand (run on real fixture)
- Snapshot tests for output formats (ensures consistency)
- Fixture covers all status codes and hours

### 5. No External Dependencies (Minimal)
- Click (required for CLI framework)
- Standard library for everything else (datetime, json, re, etc.)
- pytest for testing only

## Success Criteria

✅ All four subcommands working per spec
✅ `pytest` passes with >80% coverage
✅ `ruff check` is clean
✅ Sample fixture in `tests/fixtures/sample.log`
✅ README.md with install + usage examples
✅ Console entry point `loglens` works
✅ Meaningful git commits (5+ commits)
✅ Single `check` command runs all validations

## Scope Out (Not Included)

- Web UI or dashboard
- Real-time log tailing
- Database storage of analytics
- GeoIP or user agent parsing (beyond raw display)
- Remote log fetching (file path only)
- Configuration files or settings
