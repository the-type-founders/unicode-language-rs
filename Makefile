all: check test

check:
	cargo clippy --all-features -- -D warnings
	cargo fmt --all -- --check

test:
	cargo build
	cargo test --all-features

.PHONY: all check test
