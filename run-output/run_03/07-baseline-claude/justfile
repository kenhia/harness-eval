#!/usr/bin/env just --justfile

# Run all checks (lint and tests)
check: lint test

# Lint with ruff
lint:
    uv run ruff check loglens tests

# Run tests with pytest
test:
    uv run pytest

# Format code with ruff
fmt:
    uv run ruff format loglens tests

# Run pytest with verbose output
test-v:
    uv run pytest -v

# Run pytest with coverage
test-cov:
    uv run pytest --cov=loglens

# Install dependencies with uv
install:
    uv sync
