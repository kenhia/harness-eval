# feedhub development tasks. Run `just` to see them.

# Show the available recipes.
default:
    @just --list

# Everything CI would check: formatting, lints, and the tests.
check: fmt-check clippy test

# Build all three binaries in release mode.
build:
    cargo build --release

# Run the whole test suite.
test:
    cargo test

# Fail if anything is not formatted.
fmt-check:
    cargo fmt --all --check

# Reformat everything in place.
fmt:
    cargo fmt --all

# Lint everything, including tests, with warnings as errors.
clippy:
    cargo clippy --all-targets -- -D warnings

# Write the fixture corpus into ./fixtures and serve it on port 8601.
serve-fixtures dir="fixtures" listen="127.0.0.1:8601":
    cargo run --quiet -p feedgen -- make-fixtures {{dir}}
    cargo run --quiet -p feedgen -- serve --dir {{dir}} --listen {{listen}}

# Run feedd against ./feedhub.db with polling off, for poking at by hand.
run-server db="feedhub.db" listen="127.0.0.1:8600":
    cargo run --quiet -p feedd -- --db {{db}} --listen {{listen}} --poll-interval 0

# Remove build output and the scratch files the recipes above create.
clean:
    cargo clean
    rm -rf fixtures feedhub.db feedhub.db-wal feedhub.db-shm
