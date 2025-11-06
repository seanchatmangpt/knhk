# KNHK Test & Validation Scripts

This directory contains automated testing and validation scripts for the KNHK project.

## Scripts Overview

### 1. `run-all-tests.sh`
**Purpose**: Run complete test suite across all validation levels.

**Usage**:
```bash
./scripts/run-all-tests.sh
```

**What it tests**:
- ✅ Level 1: Weaver registry schema validation (source of truth)
- ✅ Level 2: Cargo build, Clippy, C library compilation
- ✅ Level 3: All traditional tests (Cargo, Chicago TDD, Performance, Integration)

**Exit codes**:
- `0` = All tests passed
- `1` = One or more tests failed

---

### 2. `validate-production-ready.sh`
**Purpose**: Comprehensive production readiness validation.

**Usage**:
```bash
./scripts/validate-production-ready.sh
```

**What it validates**:
- ✅ All Definition of Done criteria
- ✅ Weaver schema validation
- ✅ Zero compilation warnings
- ✅ Zero Clippy issues
- ✅ No unsafe code patterns (.unwrap/.expect)
- ✅ All test suites passing
- ✅ Performance constraints (≤8 ticks)

**Exit codes**:
- `0` = Code is production-ready
- `1` = Code is NOT production-ready

---

### 3. `pre-commit-hook.sh`
**Purpose**: Quick validation before git commits.

**Usage**: Automatically runs on `git commit` (after setup).

**What it checks**:
- ✅ Cargo Clippy (zero warnings)
- ✅ Cargo build check
- ✅ Unsafe patterns in staged files
- ✅ Quick unit tests

**To skip** (not recommended):
```bash
git commit --no-verify
```

---

### 4. `setup-git-hooks.sh`
**Purpose**: Install git pre-commit hook.

**Usage**:
```bash
./scripts/setup-git-hooks.sh
```

**What it does**:
- Creates symlink: `.git/hooks/pre-commit` → `scripts/pre-commit-hook.sh`
- Makes hook executable

---

## Recommended Workflow

### During Development
```bash
# Quick validation before commit
git add .
git commit -m "Your message"  # Pre-commit hook runs automatically
```

### Before Pull Request
```bash
# Run full test suite
./scripts/run-all-tests.sh
```

### Before Release
```bash
# Comprehensive production validation
./scripts/validate-production-ready.sh

# If validation passes, also run live-check during runtime:
weaver registry live-check --registry registry/
```

---

## Understanding Validation Levels

### ⚠️ CRITICAL: The Validation Hierarchy

**LEVEL 1: Weaver Schema Validation (MANDATORY - Source of Truth)**
- `weaver registry check -r registry/` - Validates schema definition
- `weaver registry live-check --registry registry/` - Validates runtime telemetry
- **Why it matters**: Only way to prove features actually work (no false positives)

**LEVEL 2: Compilation & Code Quality (Baseline)**
- `cargo build --workspace` - Must compile with zero warnings
- `cargo clippy --workspace -- -D warnings` - Zero Clippy issues
- `make build` - C library must compile

**LEVEL 3: Traditional Tests (Supporting Evidence Only)**
- `cargo test --workspace` - Rust unit tests
- `make test-chicago-v04` - Chicago TDD tests
- `make test-performance-v04` - Performance tests (≤8 ticks)
- `make test-integration-v2` - Integration tests
- **⚠️ Can have false positives**: Tests can pass even when features don't work

---

## Troubleshooting

### Pre-commit hook not running
```bash
# Re-run setup
./scripts/setup-git-hooks.sh
```

### Tests failing on specific subsystem
```bash
# Run subsystem tests individually
cd rust/knhk-etl && cargo test
cd rust/knhk-sidecar && cargo test
```

### Weaver validation failing
```bash
# Check schema validity
weaver registry check -r registry/

# Check for schema errors
cat registry/*.yaml
```

---

## CI/CD Integration

These scripts can be integrated into GitHub Actions:

```yaml
# .github/workflows/ci.yml
- name: Run All Tests
  run: ./scripts/run-all-tests.sh

- name: Validate Production Readiness
  run: ./scripts/validate-production-ready.sh
```

---

## Script Maintenance

**Adding new test suites**:
1. Add test command to `run-all-tests.sh`
2. Add validation logic to `validate-production-ready.sh`
3. Update this README

**Modifying validation criteria**:
1. Update scripts to match Definition of Done in `/Users/sac/knhk/CLAUDE.md`
2. Test changes locally before committing
