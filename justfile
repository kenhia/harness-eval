# List available recipes
default:
    @just --list

# Run CI gates (lint + tests)
check: lint test

# Lint with ruff
lint:
    uv run ruff check .

# Run the test suite
test:
    uv run pytest

# Auto-format / auto-fix with ruff
fmt:
    uv run ruff check --fix .
