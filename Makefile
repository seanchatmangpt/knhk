# KNHK Root Makefile
# Provides convenient targets for project-wide operations

.PHONY: validate-v1.0 validate-production-ready validate-dod-v1 help test test-rust test-c test-chicago-v04 test-performance-v04 test-integration-v2 test-all build build-rust build-rust-debug build-c lint-rust lint-c clean check fmt clippy quick-test all

help:
	@echo "KNHK Makefile Targets:"
	@echo ""
	@echo "Validation:"
	@echo "  validate-v1.0              - Run v1.0 Definition of Done validation"
	@echo "  validate-production-ready  - Run production readiness validation"
	@echo "  validate-dod-v1            - Validate Definition of Done v1.0"
	@echo ""
	@echo "Testing:"
	@echo "  test                       - Run all tests (Rust + C)"
	@echo "  test-rust                  - Run all Rust crate tests (concurrent)"
	@echo "  test-c                     - Run C library tests"
	@echo "  test-chicago-v04           - Run Chicago TDD tests (Rust + C, concurrent)"
	@echo "  test-performance-v04       - Run performance tests (τ ≤ 8 validation, concurrent)"
	@echo "  test-integration-v2        - Run integration tests (C + Rust, concurrent)"
	@echo "  test-all                   - Run test + test-chicago-v04 + test-performance-v04"
	@echo "  quick-test                 - Run fast subset of tests"
	@echo ""
	@echo "Building:"
	@echo "  build                      - Build all (Rust + C, parallel)"
	@echo "  build-rust                 - Build all Rust crates (release, parallel)"
	@echo "  build-rust-debug           - Build all Rust crates (debug, parallel)"
	@echo "  build-c                    - Build C library"
	@echo ""
	@echo "Development:"
	@echo "  check                      - Check compilation without building (fast)"
	@echo "  fmt                        - Format all Rust code"
	@echo "  clippy                     - Alias for lint-rust"
	@echo "  lint-rust                  - Run cargo clippy on all crates (concurrent)"
	@echo "  lint-c                     - Lint C code (future)"
	@echo "  clean                      - Remove build artifacts"
	@echo ""
	@echo "Workflow:"
	@echo "  all                        - Build + test + lint (full validation)"
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
# Ensure C library builds first (required for knhk-hot)
build: build-c build-rust
	@echo "✅ Build complete"

build-rust: build-c
	@echo "🔨 Building Rust crates (parallel workspace build)..."
	@cd rust && cargo build --workspace --release

build-rust-debug: build-c
	@echo "🔨 Building Rust crates (debug, parallel workspace build)..."
	@cd rust && cargo build --workspace

build-c:
	@echo "🔨 Building C library..."
	@cd c && $(MAKE) lib

# Development targets
check:
	@echo "🔍 Checking Rust code compilation (fast, no build)..."
	@cd rust && cargo check --workspace

fmt:
	@echo "📝 Formatting Rust code..."
	@cd rust && cargo fmt --all

clippy: lint-rust

# Lint targets (concurrent execution)
lint-rust:
	@echo "🔍 Linting Rust crates (concurrent)..."
	@bash scripts/run-lint-rust.sh

lint-c:
	@echo "🔍 Linting C code..."
	@echo "TODO: Implement C linting"

# Clean targets
clean:
	@echo "🧹 Cleaning build artifacts..."
	@cd rust && cargo clean
	@cd c && $(MAKE) clean || true
	@echo "✅ Clean complete"

# Quick test target (fast feedback)
quick-test: check
	@echo "⚡ Running quick tests (fast subset)..."
	@cd rust && cargo test --workspace --lib --quiet || true

# Full workflow target
all: clean build test lint-rust
	@echo "✅ Full validation complete: build + test + lint"




validate-dod-v1:
	@echo "Validating Definition of Done v1.0..."
	@./scripts/validate-dod-v1.sh || true
	@bash scripts/generate-dod-report-from-json.sh
	@echo ""
	@echo "Reports generated:"
	@echo "  - docs/V1-DOD-VALIDATION-REPORT.md"
	@echo "  - docs/V1-DOD-PROGRESS.md"
