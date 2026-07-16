# feedhub developer tasks. Run `just check` for the full gate.

# Run the complete quality gate: format check, clippy, build, tests.
check: fmt-check clippy build test

# Build all three binaries in release mode.
build:
    cargo build --release

# Run the whole test suite (unit + end-to-end).
test:
    cargo test

# Verify formatting without modifying files.
fmt-check:
    cargo fmt --all --check

# Apply formatting.
fmt:
    cargo fmt --all

# Lint with clippy, treating warnings as errors.
clippy:
    cargo clippy --all-targets -- -D warnings

# Generate the fixture corpus into ./fixtures.
fixtures dir="fixtures":
    cargo run --release --bin feedgen -- make-fixtures {{dir}}
