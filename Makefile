# KNHK Root Makefile
# Provides convenient targets for project-wide operations

.PHONY: validate-v1.0 validate-production-ready help test test-rust test-c test-chicago-v04 test-performance-v04 test-integration-v2 test-all build-rust build-c lint-rust lint-c

help:
	@echo "KNHK Makefile Targets:"
	@echo ""
	@echo "Validation:"
	@echo "  validate-v1.0              - Run v1.0 Definition of Done validation"
	@echo "  validate-production-ready  - Run production readiness validation"
	@echo ""
	@echo "Testing:"
	@echo "  test                       - Run all tests (Rust + C)"
	@echo "  test-rust                  - Run all Rust crate tests"
	@echo "  test-c                     - Run C library tests"
	@echo "  test-chicago-v04           - Run Chicago TDD tests (Rust + C)"
	@echo "  test-performance-v04       - Run performance tests (τ ≤ 8 validation)"
	@echo "  test-integration-v2        - Run integration tests (C + Rust)"
	@echo "  test-all                   - Run test + test-chicago-v04 + test-performance-v04"
	@echo ""
	@echo "Building:"
	@echo "  build                      - Build all (Rust + C)"
	@echo "  build-rust                 - Build all Rust crates"
	@echo "  build-c                    - Build C library"
	@echo ""
	@echo "Linting:"
	@echo "  lint-rust                  - Run cargo clippy on all crates"
	@echo "  lint-c                     - Lint C code (future)"
	@echo ""
	@echo "See c/Makefile for C-specific targets"

validate-v1.0:
	@bash scripts/validate_v1.0.sh

validate-production-ready:
	@bash scripts/validate-production-ready.sh

# Test targets
test: test-rust test-c
	@echo "✅ All tests completed"

test-rust:
	@echo "🧪 Running Rust tests..."
	@bash scripts/run-all-rust-tests.sh

test-c:
	@echo "🧪 Running C tests..."
	@cd c && $(MAKE) test

test-chicago-v04:
	@echo "🧪 Running Chicago TDD tests..."
	@bash scripts/run-chicago-tdd-tests.sh
	@cd c && $(MAKE) test-chicago-v04

test-performance-v04:
	@echo "⚡ Running performance tests..."
	@bash scripts/run-performance-tests.sh

test-integration-v2:
	@echo "🔗 Running integration tests..."
	@bash scripts/run-integration-tests.sh

test-all: test test-chicago-v04 test-performance-v04 test-integration-v2
	@echo "✅ Complete test suite passed"

# Build targets
build: build-rust build-c
	@echo "✅ Build complete"

build-rust:
	@echo "🔨 Building Rust crates..."
	@for crate in rust/*/Cargo.toml; do \
		if [ -f "$$crate" ]; then \
			dir=$$(dirname "$$crate"); \
			echo "Building $$dir..."; \
			(cd "$$dir" && cargo build --release) || exit 1; \
		fi \
	done

build-c:
	@echo "🔨 Building C library..."
	@cd c && $(MAKE) lib

# Lint targets
lint-rust:
	@echo "🔍 Linting Rust crates..."
	@for crate in rust/*/Cargo.toml; do \
		if [ -f "$$crate" ]; then \
			dir=$$(dirname "$$crate"); \
			echo "Linting $$dir..."; \
			(cd "$$dir" && cargo clippy --all-targets --all-features -- -D warnings) || exit 1; \
		fi \
	done

lint-c:
	@echo "🔍 Linting C code..."
	@echo "TODO: Implement C linting"




validate-dod-v1:
	@echo "Validating Definition of Done v1.0..."
	@./scripts/validate-dod-v1.sh || true
	@bash scripts/generate-dod-report-from-json.sh
	@echo ""
	@echo "Reports generated:"
	@echo "  - docs/V1-DOD-VALIDATION-REPORT.md"
	@echo "  - docs/V1-DOD-PROGRESS.md"
