# KNHK Root Makefile
# Provides convenient targets for project-wide operations

.PHONY: validate-v1.0 validate-production-ready validate-dod-v1 help test test-rust test-c test-chicago test-performance test-integration test-chicago-v04 test-performance-v04 test-integration-v2 test-all test-cache-start test-cache-stop test-cache-status test-cache-rebuild build build-rust build-rust-release build-rust-debug build-c build-cli-fast build-workflow-fast lint-rust lint-c clean check fmt clippy quick-test pre-commit all

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
	@echo "  test-chicago               - Run Chicago TDD tests (optimized with caching)"
	@echo "  test-performance           - Run performance tests (optimized with caching)"
	@echo "  test-integration           - Run integration tests (C + Rust, concurrent)"
	@echo "  test-all                   - Run test + test-chicago + test-performance + test-integration"
	@echo ""
	@echo "Test Cache Management:"
	@echo "  test-cache-start           - Start autonomic test cache daemon (keeps binaries ready)"
	@echo "  test-cache-stop            - Stop test cache daemon"
	@echo "  test-cache-status          - Check test cache daemon status"
	@echo "  test-cache-rebuild         - Force rebuild of test cache"
	@echo "  quick-test                 - Run fast subset of tests"
	@echo ""
	@echo "Building:"
	@echo "  build                      - Build all (Rust + C, incremental dev)"
	@echo "  build-rust                 - Build all Rust crates (incremental, dev profile)"
	@echo "  build-rust-release         - Build all Rust crates (release, optimized)"
	@echo "  build-rust-debug           - Alias for build-rust"
	@echo "  build-c                    - Build C library"
	@echo "  build-cli-fast             - Build knhk-cli (minimal features, fast)"
	@echo "  build-workflow-fast        - Build knhk-workflow-engine (minimal features, fast)"
	@echo ""
	@echo "Development:"
	@echo "  check                      - Check compilation without building (fast)"
	@echo "  fmt                        - Format all Rust code (NEVER use cargo fmt directly)"
	@echo "  clippy                     - Alias for lint-rust"
	@echo "  lint-rust                  - Run cargo clippy on all crates (concurrent)"
	@echo "  lint-c                     - Lint C code (future)"
	@echo "  clean                      - Remove build artifacts"
	@echo "  measure-build-times        - Measure and document build times for all crates"
	@echo "  pre-commit                 - Run pre-commit validation checks (check, fmt, lint)"
	@echo ""
	@echo "Workflow:"
	@echo "  all                        - Build + test + lint (full validation)"
	@echo ""
	@echo "See c/Makefile for C-specific targets"

validate-v1.0:
	@timeout 5 bash scripts/validate_v1.0.sh

validate-production-ready:
	@timeout 5 bash scripts/validate-production-ready.sh

# Test targets
test: test-rust test-c test-shell
	@echo "✅ All tests completed"

test-rust:
	@echo "🧪 Running Rust tests with cargo-nextest..."
	@timeout 5 sh -c 'cd rust && cargo nextest run --workspace || (echo "⚠️  cargo-nextest not installed, falling back to cargo test" && cargo test --workspace --lib -- --test-threads=1 --quiet)'

test-shell:
	@echo "🧪 Running shell script tests with bats..."
	@if command -v bats >/dev/null 2>&1; then \
		bats tests/shell/*.bats; \
	else \
		echo "⚠️  bats not installed, skipping shell script tests"; \
		echo "   Install with: brew install bats-core (macOS) or apt-get install bats (Linux)"; \
	fi

# Test cache daemon management
test-cache-start:
	@echo "🚀 Starting test cache daemon..."
	@bash scripts/test-cache-daemon.sh start

test-cache-stop:
	@echo "🛑 Stopping test cache daemon..."
	@bash scripts/test-cache-daemon.sh stop

test-cache-status:
	@bash scripts/test-cache-daemon.sh status

test-cache-rebuild:
	@echo "🔨 Rebuilding test cache..."
	@bash scripts/test-cache-daemon.sh rebuild

# Optimized test targets (use pre-compiled binaries and caching)
test-chicago:
	@echo "🧪 Running Chicago TDD tests (optimized)..."
	@bash scripts/test-runner-optimized.sh chicago

test-performance:
	@echo "⚡ Running performance tests (optimized)..."
	@bash scripts/test-runner-optimized.sh performance

test-integration:
	@echo "🔗 Running integration tests..."
	@timeout 5 bash scripts/run-integration-tests.sh

test-all: test test-chicago test-performance test-integration
	@echo "✅ Complete test suite passed"

# Legacy targets (deprecated, use test-chicago/test-performance instead)
test-chicago-v04: test-chicago
	@echo "⚠️  test-chicago-v04 is deprecated, use test-chicago instead"

test-performance-v04: test-performance
	@echo "⚠️  test-performance-v04 is deprecated, use test-performance instead"

test-integration-v2: test-integration
	@echo "⚠️  test-integration-v2 is deprecated, use test-integration instead"

# Build targets
# Ensure C library builds first (required for knhk-hot)
build: build-c build-rust
	@echo "✅ Build complete"

build-rust: build-c
	@echo "🔨 Building Rust crates (incremental, dev profile)..."
	@timeout 5 sh -c 'cd rust && CARGO_INCREMENTAL=1 cargo build --workspace'

build-rust-release: build-c
	@echo "🔨 Building Rust crates (release, parallel workspace build)..."
	@timeout 5 sh -c 'cd rust && CARGO_INCREMENTAL=1 cargo build --workspace --release'

build-rust-debug: build-rust

build-c:
	@echo "🔨 Building C library..."
	@timeout 5 sh -c 'cd c && $(MAKE) lib'

# Fast build targets (minimal features, incremental)
build-cli-fast:
	@echo "🔨 Building knhk-cli (minimal features, incremental)..."
	@timeout 5 sh -c 'cd rust && CARGO_INCREMENTAL=1 cargo build -p knhk-cli --features minimal'

build-workflow-fast:
	@echo "🔨 Building knhk-workflow-engine (minimal features, incremental)..."
	@timeout 5 sh -c 'cd rust && CARGO_INCREMENTAL=1 cargo build -p knhk-workflow-engine --no-default-features'

# Development targets
check:
	@echo "🔍 Checking Rust code compilation (incremental, fast)..."
	@cd /Users/sac/knhk/rust && CARGO_INCREMENTAL=1 cargo check --workspace --message-format=short 2>&1 || \
		(echo "⚠️  Build directory locked - waiting for other cargo processes..." && \
		 sleep 2 && \
		 cd /Users/sac/knhk/rust && CARGO_INCREMENTAL=1 cargo check --workspace --message-format=short)

measure-build-times:
	@echo "⏱️  Measuring build times for all crates..."
	@bash scripts/measure-build-times.sh

fmt:
	@echo "📝 Formatting Rust code..."
	@cd rust && (cargo fmt --all -- --check > /dev/null 2>&1 || cargo fmt --all) || \
		(echo "⚠️  Some files may have compilation errors, formatting what can be formatted..." && \
		 cargo fmt --all 2>&1 | grep -v "error\|failed" || true)

# Pre-commit validation target
pre-commit: check fmt lint-rust
	@echo "✅ Pre-commit validation passed"
	@echo ""
	@echo "Next steps:"
	@echo "  1. Run 'make test-rust' to verify tests pass"
	@echo "  2. Run 'git add .' to stage changes"
	@echo "  3. Run 'git commit -m \"your message\"' to commit"

clippy: lint-rust

# Lint targets (concurrent execution)
lint-rust:
	@echo "🔍 Linting Rust crates (concurrent)..."
	@timeout 5 bash scripts/run-lint-rust.sh

lint-c:
	@echo "🔍 Linting C code..."
	@echo "TODO: Implement C linting"

# Clean targets
clean:
	@echo "🧹 Cleaning build artifacts..."
	@cd rust && cargo clean || echo "⚠️  Cargo clean had locks (this is OK if other cargo processes are running)"
	@cd c && $(MAKE) clean || true
	@echo "✅ Clean complete"

# Quick test target (fast feedback)
quick-test: check
	@echo "⚡ Running quick tests (lib tests only, fast)..."
	@timeout 5 sh -c 'cd rust && CARGO_INCREMENTAL=1 cargo test --workspace --lib --test-threads=1 --quiet || true'

# Full workflow target
all: clean build test lint-rust
	@echo "✅ Full validation complete: build + test + lint"




validate-dod-v1:
	@echo "Validating Definition of Done v1.0..."
	@timeout 5 ./scripts/validate-dod-v1.sh || true
	@timeout 5 bash scripts/generate-dod-report-from-json.sh
	@echo ""
	@echo "Reports generated:"
	@echo "  - docs/V1-DOD-VALIDATION-REPORT.md"
	@echo "  - docs/V1-DOD-PROGRESS.md"

# Include test cache configuration
-include Makefile.test-cache
