# loglens development tasks

# Run linting and the test suite (the single "run all checks" command)
check: lint test

# Lint with ruff
lint:
    uv run ruff check .

# Run the test suite
test:
    uv run pytest -q

# Auto-fix lint issues and format
fix:
    uv run ruff check --fix .
