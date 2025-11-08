# KNHK Automation Infrastructure

## Overview

The KNHK project has a comprehensive automation infrastructure that ensures code quality, validates production readiness, and enforces the project's Definition of Done criteria at every stage of development.

## Table of Contents

1. [Automation Scripts](#automation-scripts)
2. [Git Hooks](#git-hooks)
3. [CI/CD Pipelines](#cicd-pipelines)
4. [Validation Hierarchy](#validation-hierarchy)
5. [Usage Guide](#usage-guide)
6. [Troubleshooting](#troubleshooting)

---

## Automation Scripts

All automation scripts are located in the `scripts/` directory and are executable.

### Core Scripts

#### 1. `run-all-tests.sh`
**Purpose**: Comprehensive test suite that validates all Definition of Done criteria.

**What it does**:
- ✅ Runs Weaver registry validation (MANDATORY - source of truth)
- ✅ Builds all Rust crates with zero warnings
- ✅ Runs Clippy linting with zero issues
- ✅ Executes all unit tests
- ✅ Runs integration tests

**Usage**:
```bash
./scripts/run-all-tests.sh
```

**Exit Codes**:
- `0` - All tests passed
- `1` - Some tests failed

---

#### 2. `validate-production-ready.sh`
**Purpose**: Production readiness validation following the KNHK validation hierarchy.

**What it does**:
- **LEVEL 1**: Weaver schema validation (MANDATORY)
- **LEVEL 2**: Compilation & code quality checks
- **LEVEL 3**: Traditional tests (supporting evidence)

**Usage**:
```bash
./scripts/validate-production-ready.sh
```

**Key Checks**:
1. Weaver registry schema is valid
2. Zero compilation warnings
3. Zero Clippy issues
4. No unsafe code patterns (`.unwrap()`, `.expect()`)
5. All cargo tests pass
6. Integration tests pass

---

#### 3. `test-integration.sh`
**Purpose**: Run integration tests across all KNHK crates.

**What it does**:
- Tests knhk-etl pipeline integration
- Tests knhk-hot FFI layer
- Tests knhk-warm warm path
- Tests knhk-sidecar gRPC functionality

**Usage**:
```bash
./scripts/test-integration.sh
```

---

#### 4. `setup-git-hooks.sh`
**Purpose**: Install Git pre-commit hooks for automatic validation.

**What it does**:
- Creates symlink to pre-commit hook script
- Makes hook executable
- Configures automatic validation on every commit

**Usage**:
```bash
./scripts/setup-git-hooks.sh
```

**Output**:
```
✅ Git hooks installed successfully!

Pre-commit hook will now run on every commit.
To skip the hook (not recommended): git commit --no-verify
```

---

#### 5. `verify-weaver.sh`
**Purpose**: Verify Weaver installation and registry validation.

**What it does**:
- Checks if Weaver is installed
- Validates OTel registry schema
- Provides installation instructions if needed

**Usage**:
```bash
./scripts/verify-weaver.sh
```

---

#### 6. `install-weaver.sh`
**Purpose**: Install OpenTelemetry Weaver tool.

**What it does**:
- Downloads latest Weaver release
- Installs to `/usr/local/bin/`
- Verifies installation

**Usage**:
```bash
./scripts/install-weaver.sh
```

---

## Git Hooks

### Pre-commit Hook

**Location**: `.git/hooks/pre-commit` → `scripts/pre-commit-hook.sh`

**Purpose**: Fast validation before each commit to catch issues early.

**What it validates**:
1. ✅ Cargo Clippy (zero warnings)
2. ✅ Cargo build check
3. ✅ No unsafe patterns (`.unwrap()`, `.expect()`, `println!`)
4. ✅ Quick unit tests (lib tests only)

**Runtime**: ~30 seconds (much faster than full test suite)

**Bypass** (not recommended):
```bash
git commit --no-verify
```

**Trigger**: Automatically runs on `git commit`

---

## CI/CD Pipelines

### GitHub Actions Workflows

#### 1. `ci.yml` - Continuous Integration
**Trigger**: Push to `main`/`develop`, Pull Requests

**Jobs**:

##### Job 1: Weaver Validation (MANDATORY)
- Validates OTel registry schema
- **Blocks all other jobs if fails**
- Comments on PR if validation fails

##### Job 2: Build and Lint
- Runs on Ubuntu and macOS
- Builds all crates
- Runs Clippy with `-D warnings`
- Checks code formatting
- Detects unsafe patterns

##### Job 3: Test Suite
- Runs all unit tests
- Runs integration tests
- Generates test coverage (Ubuntu)
- Uploads coverage to Codecov

##### Job 4: Production Ready
- Runs full production validation
- Comments on PR with results
- Final gate before merge

##### Job 5: Security Audit
- Runs `cargo audit`
- Runs `cargo deny`
- Checks for vulnerabilities

**Matrix Strategy**:
```yaml
os: [ubuntu-latest, macos-latest]
rust: [stable]
```

---

#### 2. `v0.4.0-release.yml` - Release Validation
**Trigger**: Version tags (`v0.4.0*`), Manual dispatch

**Jobs**:

##### Validate
- Multi-OS validation (Ubuntu, macOS)
- Builds C library
- Runs all tests
- Uploads validation reports

##### Release
- Creates GitHub release
- Generates release notes
- Attaches artifacts

---

## Validation Hierarchy

KNHK uses a **three-level validation hierarchy** to ensure production readiness:

### Level 1: Weaver Schema Validation (MANDATORY)
**Source of Truth** - Must always pass

```bash
weaver registry check -r registry/
weaver registry live-check --registry registry/
```

**Why it's critical**:
- Schema-first validation proves runtime behavior
- External tool validates our framework (no circular dependency)
- Detects "fake-green" tests that pass but don't validate actual behavior

---

### Level 2: Compilation & Code Quality (Baseline)
**Must compile and meet quality standards**

```bash
cargo build --workspace   # Zero warnings
cargo clippy --workspace -- -D warnings  # Zero issues
```

**Checks**:
- ✅ Code compiles successfully
- ✅ Zero compiler warnings
- ✅ Zero Clippy linting issues
- ✅ No unsafe patterns (`.unwrap()`, `.expect()`)

---

### Level 3: Traditional Tests (Supporting Evidence)
**Can have false positives** - Use as supporting evidence

```bash
cargo test --workspace
cd rust/knhk-integration-tests && cargo test
```

**Important**:
- Tests can pass even when features don't work (false positives)
- Only Weaver validation proves runtime behavior
- Tests provide supporting evidence, not proof

---

## Usage Guide

### For Developers

#### Before Committing
**Quick validation** (30 seconds):
```bash
# Pre-commit hook runs automatically on git commit
git add .
git commit -m "feat: implement new feature"
```

**Manual validation**:
```bash
./scripts/pre-commit-hook.sh
```

---

#### Before Opening PR
**Full validation** (5-10 minutes):
```bash
./scripts/run-all-tests.sh
```

**Production readiness check**:
```bash
./scripts/validate-production-ready.sh
```

---

#### During Development
**Watch mode** (continuous testing):
```bash
cargo watch -x test -x clippy
```

**Single crate testing**:
```bash
cd rust/knhk-etl
cargo test
cargo clippy
```

---

### For CI/CD

#### Required Checks (Branch Protection)
Enable these checks in GitHub branch protection rules:

- ✅ `weaver-validation` - MANDATORY
- ✅ `build-and-lint (ubuntu-latest)` - MANDATORY
- ✅ `build-and-lint (macos-latest)` - MANDATORY
- ✅ `test (ubuntu-latest)` - MANDATORY
- ✅ `production-ready` - MANDATORY

---

#### Optional Checks
- Security Audit (non-blocking)
- Test Coverage (informational)

---

### For Release Management

#### Pre-release Validation
```bash
# Run full validation suite
./scripts/validate-production-ready.sh

# Check release checklist
./scripts/release_checklist.sh
```

#### Release Process
1. Create version tag: `git tag v0.4.0`
2. Push tag: `git push origin v0.4.0`
3. CI runs release validation workflow
4. GitHub release created automatically

---

## Troubleshooting

### Common Issues

#### 1. Pre-commit Hook Not Running
**Problem**: Hook doesn't execute on commit

**Solution**:
```bash
# Reinstall hooks
./scripts/setup-git-hooks.sh

# Verify installation
ls -la .git/hooks/pre-commit
```

---

#### 2. Weaver Validation Fails
**Problem**: `weaver registry check` fails

**Solution**:
```bash
# Check Weaver installation
weaver --version

# Reinstall if needed
./scripts/install-weaver.sh

# Validate registry manually
weaver registry check -r registry/
```

---

#### 3. CI Jobs Failing Locally But Pass in CI
**Problem**: Tests pass locally but fail in CI

**Solution**:
```bash
# Clean build cache
cargo clean

# Run with same flags as CI
cargo test --workspace --no-fail-fast

# Check for OS-specific issues
# CI runs on both Ubuntu and macOS
```

---

#### 4. Clippy Warnings on CI
**Problem**: Clippy passes locally but fails in CI

**Solution**:
```bash
# Run Clippy with exact CI flags
cargo clippy --workspace -- -D warnings

# Fix all warnings (not just errors)
cargo clippy --workspace --fix
```

---

#### 5. Git Hook Too Slow
**Problem**: Pre-commit hook takes too long

**Solution**:
```bash
# Temporary bypass (not recommended)
git commit --no-verify

# Or run only quick validation
cargo check --workspace
cargo clippy --workspace -- -D warnings
```

**Note**: Full validation runs in CI regardless

---

### Performance Tips

#### Speed Up Local Testing
```bash
# Use cargo nextest (faster test runner)
cargo install cargo-nextest
cargo nextest run

# Run tests in parallel (default behavior)
cargo test -- --test-threads=8

# Skip doc tests
cargo test --lib --bins
```

---

#### Speed Up CI
**Caching**: CI automatically caches Cargo dependencies
```yaml
uses: actions/cache@v4
key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

**Parallel jobs**: CI runs jobs in parallel when possible

---

## Best Practices

### For Developers

1. **Run pre-commit hook** before every commit
2. **Run full validation** before opening PR
3. **Never skip validation** with `--no-verify` unless absolutely necessary
4. **Fix warnings immediately** - Don't let them accumulate
5. **Trust Weaver validation** as the source of truth

---

### For Maintainers

1. **Enforce branch protection** with required checks
2. **Review CI failures** before merging
3. **Keep automation scripts updated** with project structure
4. **Monitor CI performance** and optimize slow jobs
5. **Document any CI bypasses** in PR description

---

## Script Execution Matrix

| Script | Runtime | When to Use | Exit Code |
|--------|---------|-------------|-----------|
| `pre-commit-hook.sh` | ~30s | Before every commit | 0=pass, 1=fail |
| `run-all-tests.sh` | 5-10min | Before PR | 0=pass, 1=fail |
| `validate-production-ready.sh` | 5-10min | Before release | 0=pass, 1=fail |
| `test-integration.sh` | 2-5min | Integration testing | 0=pass, 1=fail |
| `setup-git-hooks.sh` | <1s | Once per clone | 0=success |
| `verify-weaver.sh` | <1s | Check Weaver install | 0=installed, 1=missing |

---

## Additional Resources

- **Weaver Documentation**: https://github.com/open-telemetry/weaver
- **GitHub Actions**: https://docs.github.com/actions
- **Cargo Documentation**: https://doc.rust-lang.org/cargo/
- **KNHK Definition of Done**: See `/Users/sac/knhk/CLAUDE.md`

---

## Support

For issues with automation infrastructure:
1. Check this documentation first
2. Review CI logs for specific errors
3. Run scripts locally to reproduce issues
4. Create GitHub issue with logs and error details

---

**Last Updated**: 2025-11-06
**Version**: 1.0.0
**Maintainer**: KNHK Automation Team
