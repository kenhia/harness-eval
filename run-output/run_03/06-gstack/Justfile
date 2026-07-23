# loglens Justfile

# Run all checks
check: lint test
    @echo "✓ All checks passed!"

# Run linting with ruff
lint:
    ruff check src/ tests/
    ruff format --check src/ tests/

# Format code with ruff
fmt:
    ruff format src/ tests/

# Run tests with pytest
test:
    pytest -v

# Install package in development mode
install:
    uv pip install -e .

# Clean build artifacts
clean:
    rm -rf build dist *.egg-info
    find . -type d -name __pycache__ -exec rm -rf {} +
    find . -type f -name "*.pyc" -delete

# Run loglens help
help:
    python -m loglens.cli --help
