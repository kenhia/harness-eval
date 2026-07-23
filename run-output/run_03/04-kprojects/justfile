# List available recipes
default:
    @just --list

# Install dependencies
install:
    uv pip install -e .[dev]

# Run CI gates (lint, typecheck, tests)
check: lint test
    @echo "All checks passed!"

# Lint code with ruff
lint:
    uv run ruff check .

# Format code with ruff
fmt:
    uv run ruff format .

# Run tests with pytest
test:
    uv run pytest

# Run tests with coverage
test-cov:
    uv run pytest --cov=loglens
