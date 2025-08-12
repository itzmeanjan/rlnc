# Collects inspiration from https://github.com/0xMiden/miden-base/blob/983357b2ad42f6e8d3c338d460a69479b99a1136/Makefile

.DEFAULT_GOAL := help

.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

BACKTRACE=RUST_BACKTRACE=1

.PHONY: clippy
clippy: ## Runs clippy showing warnings
	cargo clippy --all-targets -- -D warnings

.PHONY: format
format: ## Formats source tree
	cargo fmt --all

.PHONY: test
test: ## Run all tests
	$(BACKTRACE) RUSTFLAGS="-C target-cpu=native" cargo test --profile test-release
	$(BACKTRACE) RUSTFLAGS="-C target-cpu=native" cargo test --profile test-release --features parallel

.PHONY: test-wasm
test-wasm: ## Run all tests in WASM environment
	$(BACKTRACE) cargo test --target wasm32-wasip1 --profile test-release --no-default-features
	$(BACKTRACE) cargo test --target wasm32-wasip2 --profile test-release --no-default-features

.PHONY: coverage
coverage: ## Generates HTML code coverage report, using `cargo-tarpaulin`
	cargo tarpaulin -t 600 --profile test-release --out Html

.PHONY: bench
bench: ## Run all benchmarks
	RUSTFLAGS="-C target-cpu=native" cargo bench --profile optimized
	RUSTFLAGS="-C target-cpu=native" cargo bench --profile optimized --features parallel

.PHONY: clean
clean: ## Removes cargo target directory
	cargo clean

.PHONY: example
example: ## Runs the Full RLNC example program
	RUSTFLAGS="-C target-cpu=native" cargo run --example full_rlnc
	RUSTFLAGS="-C target-cpu=native" cargo run --example full_rlnc --features parallel

.PHONY: example-wasm
example-wasm: ## Runs the Full RLNC example program in WASM environment
	cargo run --example full_rlnc --target wasm32-wasip1 --no-default-features
	cargo run --example full_rlnc --target wasm32-wasip2 --no-default-features
