.PHONY: all build run test lint clean harden help

# Default target
all: build test lint

## Build the project
build:
	@echo "--- BUILDING ---"
	@cargo build

## Run the project
run:
	@cargo run

## Run tests
test:
	@echo "--- TESTING ---"
	@cargo test

## Run lints (clippy and fmt)
lint:
	@echo "--- LINTING ---"
	@cargo fmt --all -- --check
	@cargo clippy -- -D warnings

## Clean build artifacts
clean:
	@echo "--- CLEANING ---"
	@cargo clean

## Run all hardening and quality checks
harden:
	@echo "--- HARDENING METRICS ---"
	@echo "1. Cognitive Complexity (Clippy)"
	@cargo clippy -- -W clippy::cognitive_complexity 2>&1 | grep "cognitive complexity" || echo "Complexity within limits."
	@echo "\n2. Verification (Tests)"
	@cargo test --quiet
	@echo "\n3. Security (Unsafe Code)"
	@grep -r "unsafe" src || echo "No unsafe code blocks found."
	@echo "\n4. Robustness (unwrap/expect count)"
	@printf "Unwrap calls: "
	@grep -r "unwrap()" src --exclude-dir=target | wc -l
	@printf "Expect calls: "
	@grep -r "expect(" src --exclude-dir=target | wc -l
	@echo "\n5. Coverage (Tarpaulin)"
	@if command -v cargo-tarpaulin >/dev/null; then \
		cargo tarpaulin --ignore-tests; \
	else \
		echo "Tarpaulin not installed. Install with 'cargo install cargo-tarpaulin'."; \
	fi
	@echo "--------------------------"

## Show help
help:
	@echo "Available targets:"
	@echo "  build   - Build the project"
	@echo "  run     - Run the project"
	@echo "  test    - Run tests"
	@echo "  lint    - Run lints (fmt and clippy)"
	@echo "  clean   - Clean build artifacts"
	@echo "  harden  - Run comprehensive quality checks"
	@echo "  all     - build, test, and lint"
