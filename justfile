# Run every check: lint, format check, tests, and a JSON smoke test.
check: lint fmt test smoke

# Lint with ruff.
lint:
    uv run ruff check .

# Verify formatting.
fmt:
    uv run ruff format --check .

# Run the test suite.
test:
    uv run pytest

# Prove --format json emits a single valid JSON document for every subcommand.
smoke:
    #!/usr/bin/env bash
    set -euo pipefail
    log=tests/fixtures/sample.log
    for cmd in "summary" "top --by ip" "errors" "hourly"; do
        uv run loglens --format json $cmd "$log" | python3 -c 'import json,sys; json.load(sys.stdin)'
        echo "  ok: loglens --format json $cmd"
    done

# Apply formatting fixes.
format:
    uv run ruff format .
