# List available recipes
default:
    @just --list

# Run all CI gates: format check, clippy (warnings as errors), tests.
check:
    cargo fmt --all --check
    cargo clippy --all-targets -- -D warnings
    cargo test

# Build all three binaries in release mode.
build:
    cargo build --release

# Format the workspace.
fmt:
    cargo fmt --all

# Generate the fixture corpus into .scratch/corpus.
fixtures:
    cargo run -q -p feedgen -- make-fixtures .scratch/corpus

# Serve the fixture corpus locally (default 127.0.0.1:8700).
serve-fixtures: fixtures
    cargo run -q -p feedgen -- serve --dir .scratch/corpus

# Run the server against a scratch database.
run-server db=".scratch/feedd.sqlite":
    cargo run -q -p feedd -- --db {{db}}
