# Run all checks: lint + tests
check: lint test

# Lint with ruff
lint:
    uv run ruff check .

# Run the test suite
test:
    uv run pytest -q

# Auto-format and auto-fix lint issues
fmt:
    uv run ruff check --fix .
    uv run ruff format .
