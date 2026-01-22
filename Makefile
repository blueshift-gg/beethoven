SHELL := /usr/bin/env bash
NIGHTLY_TOOLCHAIN := nightly

.PHONY: nightly-version format format-fix clippy clippy-fix check-features build-program build-program-upstream test test-upstream all-checks

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

build-program:
	@cd program-test && cargo build-sbf

build-program-upstream:
	@cd program-test && cargo +$(NIGHTLY_TOOLCHAIN) build-bpf --features upstream-bpf

test-upstream:
	@$(MAKE) build-program-upstream
	@cargo test --features upstream-bpf

test:
	@$(MAKE) build-program
	@cargo test

all-checks:
	@echo "Running all checks..."
	@$(MAKE) format
	@$(MAKE) clippy
	@$(MAKE) test
	@echo "All checks passed!"