.PHONY: check fmt fmt-check lint test deny machete audit all

check:
	cargo check --workspace --all-targets --all-features

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test --workspace --all-targets --all-features

deny:
	cargo deny check

machete:
	cargo machete

audit:
	cargo audit

all: fmt-check lint test deny machete
