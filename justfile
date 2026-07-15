# loglens developer tasks

# Run all quality checks (lint + tests)
check:
    uv run ruff check .
    uv run pytest

# Auto-fix lint issues
fmt:
    uv run ruff check --fix .

# Run the test suite
test:
    uv run pytest
