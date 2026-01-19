SHELL := /usr/bin/env bash
NIGHTLY_TOOLCHAIN := nightly

.PHONY: nightly-version format format-fix clippy clippy-fix check-features build-program-test test all-checks

nightly-version:
	@echo $(NIGHTLY_TOOLCHAIN)

format:
	@cargo +$(NIGHTLY_TOOLCHAIN) fmt --all -- --check

format-fix:
	@cargo +$(NIGHTLY_TOOLCHAIN) fmt --all

clippy:
	@cargo +$(NIGHTLY_TOOLCHAIN) clippy --all --all-features --all-targets -- -D warnings

clippy-fix:
	@cargo +$(NIGHTLY_TOOLCHAIN) clippy --all --all-features --all-targets --fix --allow-dirty --allow-staged -- -D warnings

build-program-test:
	@cd program-test && cargo +$(NIGHTLY_TOOLCHAIN) build-bpf

test:
	@$(MAKE) build-program-test
	@cargo test

all-checks:
	@echo "Running all checks..."
	@$(MAKE) format
	@$(MAKE) clippy
	@$(MAKE) test
	@echo "All checks passed!"