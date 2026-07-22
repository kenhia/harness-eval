#!/usr/bin/env just --justfile

set positional-arguments

@default:
    just --list

@check:
    #!/bin/bash
    set -e
    echo "Running tests..."
    python -m pytest tests/ -q
    echo "Running ruff check..."
    python -m ruff check .
    echo "Checking CLI help..."
    loglens --help > /dev/null
    echo "All checks passed!"

@test:
    python -m pytest tests/ -v

@lint:
    python -m ruff check . --fix

@dev:
    uv sync
