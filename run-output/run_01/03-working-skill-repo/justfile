# loglens task runner
# Usage: `just <recipe>` — run `just` with no args to list recipes.

# List available recipes
default:
    @just --list

# Install project + dev dependencies into a uv-managed venv
install:
    uv sync

# Lint with ruff
lint:
    uv run ruff check .

# Run the test suite
test:
    uv run pytest

# Run all checks (lint + tests)
check: lint test
