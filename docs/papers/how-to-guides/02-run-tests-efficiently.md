# How-to Guide: Run Tests Efficiently

**Goal**: Understand and execute tests optimally
**Time**: 10 minutes
**Difficulty**: Beginner

## Overview

KNHK has multiple test suites validating different aspects. This guide shows you how to run them efficiently.

## Available Test Suites

| Command | Purpose | Time | When to Use |
|---------|---------|------|------------|
| `cargo test --lib` | Fast unit tests | 30s | During development |
| `make test-chicago-v04` | Comprehensive TDD tests | 2m | Before commit |
| `make test-performance-v04` | Performance validation | 1m | Before push (≤8 ticks) |
| `make test-integration-v2` | Integration tests | 3m | Before merge |
| `cargo test --workspace` | All tests | 5m | CI/CD only |

## Quick Run (What You Need Most)

```bash
# Run only fast unit tests (30 seconds)
cargo test --lib --release
```

Use this during development. It's fast and catches most issues.

## Before Committing

```bash
# Run comprehensive tests
make test-chicago-v04
```

This validates:
- All unit tests pass
- Chicago TDD patterns are followed
- Basic functionality works

**Expected**: All tests pass, no failures

## Before Pushing

```bash
# Validate performance meets Chatman Constant
make test-performance-v04
```

This validates:
- All operations meet ≤8 tick constraint
- No performance regressions
- Memory usage is efficient

**Expected**: All performance metrics pass

## Full Validation (Before Merge)

```bash
# Complete integration test suite
make test-integration-v2
```

This validates:
- All components work together
- Cross-module integration is correct
- End-to-end workflows succeed

**Expected**: Full success

## Testing Single Test

```bash
# Run one specific test
cargo test --lib your_test_name -- --nocapture
```

Options:
- `--nocapture` - Show println! output
- `--test-threads=1` - Run sequentially (slower but deterministic)
- `--release` - Run optimized (matches performance tests)

## Testing Specific Module

```bash
# Test one module
cargo test --lib module_name::
```

Example:
```bash
cargo test --lib chatman_equation::
```

## Debug Test Output

```bash
# Show detailed logging
RUST_LOG=debug cargo test --lib your_test_name -- --nocapture --test-threads=1
```

Show what happened inside the test.

## Performance Testing Detail

```bash
# Run performance tests with output
RUST_LOG=info make test-performance-v04
```

See actual tick counts for each operation.

## Run All Tests

```bash
# Complete validation suite (careful: takes 5+ minutes)
cargo test --workspace --release && \
make test-chicago-v04 && \
make test-performance-v04 && \
make test-integration-v2
```

## Optimize Test Runtime

### Skip Some Tests Temporarily
```bash
# Run only tests matching a pattern
cargo test --lib search_pattern
```

### Run Tests in Parallel
```bash
# Default: use all CPU cores
cargo test --lib --release

# Limit to 4 threads
cargo test --lib --release -- --test-threads=4
```

### Profile Test Performance
```bash
# See which tests are slowest
cargo test --lib --release -- --nocapture | grep -E "test.*ok|test.*FAILED"
```

## Troubleshooting

### Tests Timeout
```bash
# Increase timeout
cargo test --lib --release -- --test-threads=1
```

### Tests Fail Intermittently
```bash
# Run sequentially to find the culprit
cargo test --lib --release -- --test-threads=1
```

### Out of Memory
```bash
# Use fewer threads
cargo test --lib --release -- --test-threads=2
```

### Some Tests Fail
```bash
# See detailed error
cargo test --lib failing_test_name -- --nocapture
```

## Continuous Testing (For Development)

```bash
# Watch for changes and run tests automatically
cargo watch -x 'test --lib' -c
```

Requires: `cargo install cargo-watch`

## CI/CD Pipeline Tests

These run automatically on push:

```bash
# This is what CI runs
cargo test --workspace --release && \
cargo clippy --workspace -- -D warnings && \
weaver registry check -r registry/
```

## Best Practices

✅ **DO:**
- Run `cargo test --lib` before each commit
- Run `make test-performance-v04` before push
- Use `--release` for accurate performance
- Use `--nocapture` when debugging
- Run full suite before merge

❌ **DON'T:**
- Ignore test failures
- Commit with failing tests
- Change tests to make them pass (fix code instead)
- Skip performance tests
- Run tests on main branch directly

## Test Coverage

Check what code is being tested:

```bash
# Generate coverage report (if available)
cargo tarpaulin --out Html
```

Requires: `cargo install cargo-tarpaulin`

## Related Commands

```bash
# Check code compiles
cargo check --release

# Format code before testing
cargo fmt --all

# Lint code for issues
cargo clippy --workspace -- -D warnings

# Update dependencies
cargo update

# Clean build artifacts
cargo clean
```

## Common Test Failures

### "Function not found"
- Rebuild: `cargo build --release`
- Check imports in test file

### "Assertion failed"
- Run with `--nocapture` to see actual value
- Check test expectations vs implementation

### "Timeout"
- Function may be looping infinitely
- Add debug prints to find issue
- Run with `--test-threads=1`

### "Performance test failed"
- Code exceeds ≤8 tick limit
- Profile to find bottleneck
- See: How to Optimize Performance

## Next Steps

- **Debug a failing test**: Run with `--nocapture` and `--test-threads=1`
- **Optimize performance**: See [How to Optimize Performance](07-optimize-performance.md)
- **Add new tests**: See [How to Add Tests](05-add-tests.md)

## Key Commands

```bash
# Fast development tests
cargo test --lib --release

# Before committing
make test-chicago-v04

# Before pushing
make test-performance-v04

# Full validation
make test-integration-v2

# Debug a failing test
cargo test --lib failing_test -- --nocapture --test-threads=1
```

---

**Category**: How-to Guides (Task-oriented)
**Framework**: Diátaxis
**Difficulty**: Beginner
**Related**: Setup, Performance, Troubleshooting
