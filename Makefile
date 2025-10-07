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
bench: ## Run all benchmarks in single-threaded mode
	RUSTFLAGS="-C target-cpu=native" cargo bench --profile optimized --bench full_rlnc_encoder
	RUSTFLAGS="-C target-cpu=native" cargo bench --profile optimized --bench full_rlnc_recoder
	RUSTFLAGS="-C target-cpu=native" cargo bench --profile optimized --bench full_rlnc_decoder

.PHONY: bench_parallel
bench_parallel: ## Run all benchmarks in multi-threaded mode
	RUSTFLAGS="-C target-cpu=native" cargo bench --profile optimized --features parallel --bench full_rlnc_encoder
	RUSTFLAGS="-C target-cpu=native" cargo bench --profile optimized --features parallel --bench full_rlnc_recoder
	RUSTFLAGS="-C target-cpu=native" cargo bench --profile optimized --features parallel --bench full_rlnc_decoder

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

# -----------------------------------------------------------------------------
# Following Make recipes are for easy plotting of benchmark run data, in JSONL format, on Linux or Mac machines.
# Plotting is done using Matplotlib library. Plotted images are placed inside the ./plots directory.

# Define Virtual environment state directory along with path to executables inside venv
VENV := .venv
VENV_PYTHON := $(VENV)/bin/python
VENV_PIP := $(VENV)/bin/pip

OS := $(shell uname -s)
CPU_NAME := $(shell if [ "$(OS)" = "Darwin" ]; then sysctl -n machdep.cpu.brand_string | tr ' ' '_' | tr -d '()'; elif [ "$(OS)" = "Linux" ]; then grep "model name" /proc/cpuinfo | uniq | cut -d: -f2 | sed 's/^[ \t]*//' | tr ' ' '_' | tr -d '()'; else echo "Unsupported OS: $(OS)" >&2; exit 1; fi)

.PHONY: setup
setup: install ## Sets up virtual environment and installs dependencies
	@echo "\n✅ Virtual environment is ready."
	@echo "To activate it, run: source $(VENV)/bin/activate"
	@echo "After activated, to deactivate it, run: deactivate"

# The install target now depends on the venv being created
.PHONY: install
install: $(VENV)/bin/activate ## Install dependencies using `pip`
	@echo "Installing dependencies..."
	$(VENV_PIP) install -r plots/scripts/requirements.txt

# This target creates the virtual environment
$(VENV)/bin/activate: plots/scripts/requirements.txt
	@echo "Setting up environment with venv..."
	python -m venv $(VENV)

.PHONY: bench_then_plot
bench_then_plot: ## Run benchmark, collect JSONL output and produce plot of benchmark throughput
	RUSTFLAGS="-C target-cpu=native" cargo criterion --bench full_rlnc_encoder --message-format json | tee full_rlnc_encoder.jsonl &&\
	$(VENV_PYTHON) plots/scripts/plot_benchmark.py -u GB/s --title-format "{group_name} on $(CPU_NAME)" full_rlnc_encoder.jsonl &&\
	rm full_rlnc_encoder.jsonl &&\
	mv benchmark_encode.png plots/benchmark_encode_on_$(CPU_NAME)_with_$(shell rustc --version | tr ' ' '_' | tr -d '()').png &&\
	mv benchmark_encode_zero_alloc.png plots/benchmark_encode_zero_alloc_on_$(CPU_NAME)_with_$(shell rustc --version | tr ' ' '_' | tr -d '()').png

	RUSTFLAGS="-C target-cpu=native" cargo criterion --bench full_rlnc_recoder --message-format json | tee full_rlnc_recoder.jsonl &&\
	$(VENV_PYTHON) plots/scripts/plot_benchmark.py -u GB/s --title-format "{group_name} on $(CPU_NAME)" full_rlnc_recoder.jsonl &&\
	rm full_rlnc_recoder.jsonl &&\
	mv benchmark_recode.png plots/benchmark_recode_on_$(CPU_NAME)_with_$(shell rustc --version | tr ' ' '_' | tr -d '()').png &&\
	mv benchmark_recode_zero_alloc.png plots/benchmark_recode_zero_alloc_on_$(CPU_NAME)_with_$(shell rustc --version | tr ' ' '_' | tr -d '()').png

	RUSTFLAGS="-C target-cpu=native" cargo criterion --bench full_rlnc_decoder --message-format json | tee full_rlnc_decoder.jsonl &&\
	$(VENV_PYTHON) plots/scripts/plot_benchmark.py --title-format "{group_name} on $(CPU_NAME)" full_rlnc_decoder.jsonl &&\
	rm full_rlnc_decoder.jsonl &&\
	mv benchmark_decode.png plots/benchmark_decode_on_$(CPU_NAME)_with_$(shell rustc --version | tr ' ' '_' | tr -d '()').png
