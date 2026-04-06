.PHONY: harden
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
