# How-to Guide 2: Run Tests Efficiently

## Goal

Learn various testing strategies for KNHK to validate your changes quickly, ensure quality, and meet performance constraints.

**Time Estimate**: 10-20 minutes
**Prerequisites**: Development environment setup (see [How-to: Setup Development](01-setup-development-environment.md))
**Outcomes**: Know which tests to run, when to run them, and how to interpret results

---

## The Testing Hierarchy

KNHK uses a **three-tier validation hierarchy**:

### Tier 1: Schema Validation ⭐ (Source of Truth)
```bash
weaver registry live-check --registry registry/
```
Validates actual runtime telemetry. This is what REALLY matters.

### Tier 2: Compilation & Code Quality
```bash
cargo build --release
cargo clippy --workspace -- -D warnings
```
Ensures code is valid and follows best practices.

### Tier 3: Traditional Tests (Supporting Evidence)
```bash
cargo test --workspace
make test-chicago-v04
make test-performance-v04
```
Tests can pass while features are broken. Use them to catch obvious issues, not as proof of correctness.

---

## Quick Test Commands

### Run All Tests (Complete Suite)

```bash
# Fast check (recommended for development)
cargo nextest run --workspace

# Traditional test runner
cargo test --workspace

# With output
cargo test --workspace -- --nocapture
```

**Time**: 2-5 minutes
**Use when**: You've made changes and want to validate nothing broke

### Run Specific Tests

```bash
# Run tests in a specific module
cargo test --lib core

# Run a specific test
cargo test --lib core::tests::test_name

# Run tests matching a pattern
cargo test telemetry
```

**Time**: 30 seconds - 2 minutes
**Use when**: Debugging a specific feature

### Run Only Unit Tests

```bash
cargo test --lib
```

**Time**: 1-2 minutes
**Use when**: Quick validation during development

### Run Integration Tests

```bash
cargo test --test '*'
# Or specific test file
cargo test --test integration_test
```

**Time**: 2-5 minutes
**Use when**: Testing complete workflows

---

## Project-Specific Test Commands

### Chicago TDD Tests

```bash
# Run Chicago Test-Driven Development suite
make test-chicago-v04

# Expected: All Chicago assertions pass
```

**What it tests**:
- Core functionality correctness
- Integration points
- Chicago-style behavior-driven assertions

**Use when**: Validating feature completeness

### Performance Tests

```bash
# Verify ≤8 ticks (Chatman Constant)
make test-performance-v04

# Expected: All operations ≤8 ticks
```

**What it validates**:
- Operations meet performance constraints
- No performance regressions
- Chatman Constant compliance

**Critical**: If any operation exceeds 8 ticks, it FAILS.

### Integration Tests

```bash
make test-integration-v2

# Or run manually
cargo test --test integration
```

**What it tests**:
- Complete workflows
- Component interactions
- End-to-end functionality

---

## Code Quality Checks

### Run Clippy (Linting)

```bash
# Check with clippy
cargo clippy --all-targets --all-features -- -D warnings

# Expected: No warnings
```

**Fix warnings**:
```bash
# Some warnings can be automatically fixed
cargo clippy --fix --all-targets --all-features
cargo fmt --all
```

### Format Code

```bash
# Format all code
cargo fmt --all

# Check formatting without changing
cargo fmt --all -- --check
```

### Type Checking

```bash
# Quick type and syntax check
cargo check --workspace

# Fast: 10-30 seconds
```

**Use when**: You want quick feedback without compiling

---

## Testing Strategy by Development Stage

### During Feature Development

**Fastest feedback loop**:
```bash
# 1. Quick syntax check (fastest)
cargo check --workspace     # 10-30s

# 2. Run unit tests for your module
cargo test --lib my_module  # 30s-2min

# 3. Format and lint
cargo fmt --all
cargo clippy --all-targets
```

**Total time**: 1-3 minutes

### Before Committing Changes

```bash
# 1. Check syntax
cargo check --workspace

# 2. Run all tests
cargo test --workspace

# 3. Run clippy
cargo clippy --all-targets -- -D warnings

# 4. Format code
cargo fmt --all
```

**Total time**: 3-5 minutes

### Before Creating Pull Request

```bash
# Full validation suite
cargo build --release
cargo test --workspace
cargo clippy --all-targets -- -D warnings
make test-chicago-v04
make test-performance-v04
weaver registry live-check --registry registry/
```

**Total time**: 5-15 minutes
**Critical**: ALL must pass before PR

---

## Using cargo nextest (Recommended)

### Why nextest?

- 2-5x faster than traditional `cargo test`
- Parallel execution by default
- Better error reporting
- Progress indication

### Basic Usage

```bash
# Install (if not done)
cargo install cargo-nextest

# Run all tests
cargo nextest run --workspace

# Run specific test
cargo nextest run my_test

# List tests without running
cargo nextest list
```

### Advanced Options

```bash
# Run with specific number of threads
cargo nextest run --workspace -j 4

# Stop on first failure
cargo nextest run --fail-fast

# Exclude certain tests
cargo nextest run --workspace --exclude test_slow
```

---

## Interpreting Test Results

### Test Passes ✅

```
test result: ok. XX passed; 0 failed; 0 ignored; 0 measured
```

**Meaning**: Test code executed successfully
**Does NOT mean**: Feature actually works (use Weaver validation for proof)

### Test Fails ❌

```
test my_feature ... FAILED

failures:
    test my_feature panicked at 'assertion failed: ...'
```

**Action**:
1. Read error message carefully
2. Check line number in stack trace
3. Review code at that location
4. Fix the issue

### Test Ignored

```
test my_test ... ignored
```

**Meaning**: Test is marked with `#[ignore]`
**Action**:
```bash
# Run ignored tests
cargo test -- --ignored

# Run all tests including ignored
cargo test -- --include-ignored
```

---

## Common Test Issues & Solutions

### Issue: "test timed out"

**Cause**: Test runs too long (default 60 seconds)
**Solution**:
```bash
# Increase timeout
cargo test -- --test-threads=1 --nocapture

# Or increase in test itself
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
```

### Issue: "failed to resolve: use of undeclared type"

**Cause**: Missing imports or type not found
**Solution**:
```bash
# Check compilation
cargo check

# Add missing imports
// Add: use my_module::MyType;
```

### Issue: "thread 'main' panicked"

**Cause**: Assertion failed or panic in code
**Solution**:
```bash
# Run with backtrace
RUST_BACKTRACE=1 cargo test

# Get full backtrace
RUST_BACKTRACE=full cargo test
```

### Issue: "Weaver validation failed"

**Cause**: Runtime telemetry doesn't match schema
**Solution**:
```bash
# Check schema
weaver registry check -r registry/

# Verify telemetry emissions
# Review code for missing instrumentation
# Update schema if behavior changed
```

---

## Test Coverage

### Generate Coverage Report

```bash
# Install coverage tool
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --workspace --out Html

# Opens coverage/tarpaulin-report.html
```

### Interpret Coverage

- **Green**: Covered by tests
- **Red**: Not covered
- **Yellow**: Partially covered

**Goal**: 80%+ coverage, 100% for critical paths

---

## Performance Testing Details

### What ≤8 Ticks Means

The Chatman Constant specifies operations must complete in ≤8 ticks:

```bash
# Running performance test
make test-performance-v04

# Output shows:
# operation_name: 5 ticks ✓
# expensive_operation: 12 ticks ✗ FAIL
```

### Optimizing for Performance

```bash
# Profile with flamegraph
cargo install flamegraph
cargo flamegraph --bin my_binary

# Generate profile
perf report

# Find bottlenecks and optimize
```

---

## Continuous Development Workflow

### Recommended Pattern

```bash
# 1. Make code change
# (edit src/main.rs)

# 2. Quick check (10s)
cargo check

# 3. Run relevant tests (30s-2min)
cargo test --lib my_module

# 4. Format and lint (20s)
cargo fmt --all
cargo clippy --all-targets

# 5. Full validation before commit (3-5min)
cargo test --workspace
make test-chicago-v04

# 6. Commit
git add .
git commit -m "feat: description"

# 7. Before PR: full suite (10-15min)
cargo test --workspace
make test-chicago-v04
make test-performance-v04
weaver registry live-check --registry registry/
```

---

## Test Organization Best Practices

### File Structure

```
src/
├── lib.rs
├── module.rs
└── submodule/
    ├── mod.rs
    └── tests.rs      # Tests for submodule

tests/
├── integration/
│   ├── workflow_test.rs
│   └── integration_test.rs
└── common.rs         # Shared test utilities
```

### Test Module Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Arrange, Act, Assert pattern
    #[test]
    fn test_feature() {
        // Arrange: setup
        let input = ...;

        // Act: execute
        let result = my_function(input);

        // Assert: verify
        assert_eq!(result, expected);
    }
}
```

---

## Testing Checklist

Before committing:
- [ ] `cargo check --workspace` passes
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo fmt --all` applied
- [ ] Test coverage ≥80%
- [ ] No `.unwrap()` in production code

Before creating PR:
- [ ] All above + :
- [ ] `make test-chicago-v04` passes
- [ ] `make test-performance-v04` passes (≤8 ticks)
- [ ] `weaver registry live-check` passes
- [ ] Changes documented
- [ ] Examples added if applicable

---

## Quick Reference

| Task | Command | Time |
|------|---------|------|
| Syntax check | `cargo check --workspace` | 10-30s |
| Unit tests | `cargo test --lib` | 1-2min |
| All tests | `cargo test --workspace` | 2-5min |
| Fast tests | `cargo nextest run --workspace` | 1-3min |
| Performance | `make test-performance-v04` | 1-2min |
| Chicago TDD | `make test-chicago-v04` | 2-3min |
| Validation | `weaver registry live-check --registry registry/` | 30s-1min |
| Code quality | `cargo clippy --all-targets -- -D warnings` | 30s-2min |
| Coverage | `cargo tarpaulin --workspace` | 3-5min |

---

## Next Steps

1. **Debug Issues** - [How-to: Debug Failing Tests](03-debug-failing-tests.md) (coming soon)
2. **Optimize Performance** - [How-to: Optimize Performance](11-optimize-performance.md) (coming soon)
3. **Understand Telemetry** - [Tutorial: Understanding Telemetry](../tutorials/02-understanding-telemetry.md) (coming soon)

---

**Created**: 2025-11-15
**Updated**: 2025-11-15
**Status**: Complete
**Difficulty**: Intermediate
