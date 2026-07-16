# feedhub task runner. Run `just` to see recipes, `just check` for the full gate.

# Show available recipes.
default:
    @just --list

# Build all binaries in release mode.
build:
    cargo build --release

# Run the full test suite.
test:
    cargo test

# Check formatting without modifying files.
fmt-check:
    cargo fmt --check

# Apply formatting.
fmt:
    cargo fmt

# Lint with clippy, denying all warnings.
clippy:
    cargo clippy --all-targets -- -D warnings

# The full quality gate: format check, clippy, tests, release build.
check: fmt-check clippy test build
    @echo "all checks passed"

# Generate the fixture corpus into ./fixtures.
fixtures:
    cargo run --release -q -p feedgen -- make-fixtures ./fixtures

# Remove build artifacts.
clean:
    cargo clean
