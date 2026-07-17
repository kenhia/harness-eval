# feedhub — single command to run all checks
check:
	cargo fmt --check
	cargo build --release
	cargo clippy --all-targets -- -D warnings
	cargo test

fmt:
	cargo fmt

build:
	cargo build --release
