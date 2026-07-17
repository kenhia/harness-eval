# feedhub task runner.
#
# `just check` is the single command that runs every gate. If you would rather
# not install `just`, ./check is a plain shell script that runs the same four
# commands in the same order.

default: check

# Every gate, in the order that fails fastest.
check: fmt-check clippy test build

# Formatting is clean.
fmt-check:
    cargo fmt --all --check

# No clippy lints, across every target including tests.
clippy:
    cargo clippy --all-targets -- -D warnings

# The whole test suite.
test:
    cargo test --workspace

# Release binaries build.
build:
    cargo build --release

# Apply formatting.
fmt:
    cargo fmt --all

# Write the fixture corpus and serve it on 127.0.0.1:8601.
serve-fixtures dir="/tmp/feedhub-fixtures":
    cargo run --bin feedgen -- make-fixtures {{dir}}
    cargo run --bin feedgen -- serve --dir {{dir}}

# Run a server against a scratch database.
run db="/tmp/feedhub.db":
    cargo run --bin feedd -- --db {{db}}

clean:
    cargo clean
