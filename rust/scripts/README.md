# KNHK Validation Scripts

This directory contains automated validation scripts for the KNHK monorepo build matrix.

## Quick Reference

| Script | Time | Use Case |
|--------|------|----------|
| `validate-pre-commit.sh` | ~3 min | Before committing code |
| `validate-pre-push.sh` | ~6 min | Before pushing to remote |
| `validate-feature-matrix.sh` | ~10 min | Feature flag validation |
| `validate-integrations.sh` | ~15 min | Integration scenarios |
| `validate-tests.sh` | ~20 min | All test suites |
| `validate-release.sh` | ~25 min | Pre-release validation |

## Usage

### Pre-Commit (Fast)

```bash
./scripts/validate-pre-commit.sh
```

Runs:
- Format check
- Clippy lints
- Library tests
- Chicago TDD tests

### Pre-Push (Standard)

```bash
./scripts/validate-pre-push.sh
```

Runs:
- All pre-commit checks
- Full workspace tests
- Performance validation
- Feature flag sampling

### Feature Matrix

```bash
./scripts/validate-feature-matrix.sh
```

Tests all feature flag combinations across:
- knhk-otel (3 combinations)
- knhk-connectors (4 combinations)
- knhk-unrdf (4 combinations)
- knhk-etl (6 combinations)
- knhk-warm (4 combinations)
- knhk-validation (7 combinations)
- knhk-patterns (2 combinations)
- knhk-cli (3 combinations)

**Total: 32 feature combinations**

### Integration Scenarios

```bash
./scripts/validate-integrations.sh
```

Tests:
- Core system integration
- Pipeline system integration
- Validation system integration
- Full workspace integration

### Test Suites

```bash
./scripts/validate-tests.sh
```

Runs:
- Library tests (unit tests)
- Integration tests
- Documentation tests
- Chicago TDD tests
- Performance benchmarks
- Integration v2 tests

### Release Validation

```bash
./scripts/validate-release.sh
```

Comprehensive pre-release validation:
- All pre-push checks
- Full workspace build (all features)
- Full test suite (all features)
- Documentation generation
- Security audit
- Dependency check
- Package verification

**⚠️ CRITICAL:** After this passes, run Weaver validation:

```bash
weaver registry check -r registry/
weaver registry live-check --registry registry/
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: CI

on: [push, pull_request]

jobs:
  quick-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: ./scripts/validate-pre-commit.sh

  full-validation:
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: ./scripts/validate-pre-push.sh
```

## Parallel Execution

All scripts support parallel execution where possible. Test threads are controlled via:

```bash
# Library tests use 4 threads
cargo test --workspace --lib -- --test-threads=4

# Integration tests use 2 threads (heavier tests)
cargo test --workspace --test '*' -- --test-threads=2
```

## Customization

### Skip Specific Tests

```bash
# Skip Chicago TDD tests
SKIP_CHICAGO=1 ./scripts/validate-pre-commit.sh

# Skip performance tests
SKIP_PERF=1 ./scripts/validate-pre-push.sh
```

### Verbose Output

```bash
# Show full cargo output
VERBOSE=1 ./scripts/validate-feature-matrix.sh
```

### Custom Test Filters

```bash
# Run only specific package tests
cargo test -p knhk-etl -p knhk-warm

# Run only specific test names
cargo test --workspace -- --test pattern_*
```

## Troubleshooting

### Script Not Executable

```bash
chmod +x scripts/*.sh
```

### Makefile Not Found

Some scripts depend on Makefile targets:
- `make test-chicago-v04`
- `make test-performance-v04`
- `make test-integration-v2`

If Makefile is missing, these tests will be skipped with warnings.

### cargo-audit Not Installed

```bash
cargo install cargo-audit
```

### cargo-tarpaulin Not Installed (for coverage)

```bash
cargo install cargo-tarpaulin
```

## Performance Tips

### Use Incremental Builds

```bash
# Keep incremental builds enabled (default)
export CARGO_INCREMENTAL=1
```

### Use Cached Dependencies

```bash
# Use sccache for faster builds
cargo install sccache
export RUSTC_WRAPPER=sccache
```

### Parallel Package Builds

```bash
# Build packages in parallel (requires GNU parallel)
parallel 'cargo build -p {} --release' ::: knhk-hot knhk-otel knhk-config
```

## Output

All scripts provide:
- Progress indicators
- Success/failure status (✅/❌)
- Timing information
- Summary reports

Example output:

```
=== Pre-Commit Validation ===
Running fast validation checks before commit

## 1. Format Check
✅ Code formatting is correct

## 2. Clippy Lints
✅ No clippy warnings

## 3. Library Tests
✅ Library tests passed

## 4. Chicago TDD Validation
✅ Chicago TDD tests passed

=== Pre-Commit Validation Summary ===
Time elapsed: 187s
✅ All pre-commit checks passed

You can now safely commit your changes!
```

## Related Documentation

- [BUILD_VALIDATION_MATRIX.md](../docs/BUILD_VALIDATION_MATRIX.md) - Complete validation matrix
- [CLAUDE.md](../../CLAUDE.md) - Project configuration and guidelines
- [Definition of Done](../docs/DEFINITION_OF_DONE.md) - Release criteria

## Support

For issues or questions:
1. Check [BUILD_VALIDATION_MATRIX.md](../docs/BUILD_VALIDATION_MATRIX.md) Section 9 (Common Issues)
2. Review script output for specific errors
3. Run individual cargo commands to isolate issues
4. Check GitHub Actions logs for CI failures
