set shell := ["bash", "-c"]

# Run all checks (tests and linting)
check:
    uv run pytest tests/ -v
    uv run ruff check .

# Run tests
test:
    uv run pytest tests/ -v

# Run linting
lint:
    uv run ruff check .

# Install dependencies
install:
    uv pip install -e ".[dev]"

# Format code
fmt:
    uv run ruff format .
    uv run ruff check . --fix

# Run CLI help
help:
    uv run loglens --help
