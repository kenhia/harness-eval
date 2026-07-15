# loglens developer tasks

# Run all checks: lint + tests
check: lint test

# Lint with ruff
lint:
    uv run ruff check .

# Run the test suite
test:
    uv run pytest

# Auto-format / auto-fix lint issues
fix:
    uv run ruff check --fix .

# Run the CLI (pass args after --), e.g. `just run summary tests/fixtures/sample.log`
run *ARGS:
    uv run loglens {{ARGS}}
