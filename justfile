# Run all checks: format, lint, tests, and release build.
check: fmt-check clippy test build-release

# Verify formatting without modifying files.
fmt-check:
    cargo fmt --all -- --check

# Apply formatting.
fmt:
    cargo fmt --all

# Lint every target; warnings are errors.
clippy:
    cargo clippy --all-targets -- -D warnings

# Run the whole test suite (unit + end-to-end).
test:
    cargo test

# Build all three binaries in release mode.
build-release:
    cargo build --release

# Generate the fixture corpus into ./fixtures.
fixtures:
    cargo run -q -p feedgen -- make-fixtures fixtures
