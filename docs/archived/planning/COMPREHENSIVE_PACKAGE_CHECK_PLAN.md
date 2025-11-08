# Comprehensive Package Check Plan - Chicago TDD Methodology

## Overview
Systematically check all Rust packages in the workspace for compilation issues, version mismatches, and other problems using Chicago TDD principles: test behavior (compilation) rather than just configuration.

## What Has Been Fixed
1. ✅ `thiserror` versions - All updated to 2.0
2. ✅ `tokio` versions - All updated to 1.35  
3. ✅ `opentelemetry` versions - Updated to 0.31 in knhk-etl and knhk-unrdf
4. ✅ Unused field warnings - Already have `#[allow(dead_code)]` attributes

## What Needs to Be Checked

### Phase 1: Dependency Version Verification (CRITICAL)

#### 1.1 Blake3 Version Mismatch
- **Issue**: knhk-unrdf uses `blake3 = "1.0"` while workspace uses `"1.5"`
- **File**: `rust/knhk-unrdf/Cargo.toml` line 21
- **Fix**: Update to `"1.5"` to match workspace

#### 1.2 All Dependency Versions
Check these dependencies match workspace versions across all packages:
- `serde` / `serde_json` - Should be `"1.0"`
- `bincode` - Should be `"1.3"`
- `sha2` - Should be `"0.10"`
- `hex` - Should be `"0.4"`
- `reqwest` - Should be `"0.11"`
- `rdkafka` - Should be `"0.36"`
- `tonic` / `tonic-build` - Should be `"0.10"`
- `prost` / `prost-types` - Should be `"0.12"`
- `oxigraph` - Should be `"0.5"`
- `lru` - Should be `"0.16"`
- `ahash` - Should be `"0.8"`
- `tera` - Should be `"1.19"`
- `anyhow` - Should be `"1.0"`
- `miette` - Should be `"7.6"`
- `regorus` - Should be `"0.4"`

### Phase 2: Actual Compilation Verification (CRITICAL)

#### 2.1 Run Cargo Check on All Packages
```bash
cd /Users/sac/knhk/rust
cargo check --workspace
```

**Expected**: Zero compilation errors
**Current Status**: Unknown - needs verification

#### 2.2 Check Individual Package Compilation
For each package in workspace:
- `knhk-hot`
- `knhk-otel`
- `knhk-connectors`
- `knhk-lockchain`
- `knhk-unrdf`
- `knhk-etl`
- `knhk-warm`
- `knhk-aot`
- `knhk-validation`
- `knhk-config`
- `knhk-sidecar` (excluded from workspace but should still compile)
- `knhk-cli`
- `knhk-integration-tests`

### Phase 3: Known Compilation Errors (HIGH PRIORITY)

#### 3.1 knhk-sidecar Compilation Errors
- **Status**: 76-90+ compilation errors reported
- **Issues**:
  - Unresolved imports (knhk_connectors, knhk_otel, CircuitBreaker)
  - Type mismatches
  - Missing struct fields
- **Action**: Fix all compilation errors

#### 3.2 knhk-warm Compilation Errors
- **Status**: 14+ compilation errors reported
- **Action**: Fix all compilation errors

#### 3.3 knhk-etl Type Mismatches
- **Issues**:
  - HashMap vs Map in serde_json
  - Mutability issues in emit stage
  - Test compilation errors
- **Action**: Fix type mismatches and mutability issues

### Phase 4: Cargo.toml Structure Verification (MEDIUM)

#### 4.1 Package Metadata
- Verify all packages have:
  - `name` field
  - `version` field
  - `edition = "2021"` field
  - `license` field (if applicable)

#### 4.2 Crate Type Configuration
- Verify `[lib]` sections have correct `crate-type` values
- Verify `[[bin]]` sections have correct paths
- Verify `path` fields point to correct files

#### 4.3 Feature Flags
- Verify feature flags are consistent
- Verify optional dependencies use `optional = true`
- Verify default features match workspace expectations

### Phase 5: Test Compilation (MEDIUM)

#### 5.1 Test Dependencies
- Verify `[dev-dependencies]` are correct
- Verify test dependencies don't conflict with main dependencies

#### 5.2 Test Compilation
```bash
cargo test --workspace --no-run
```
**Expected**: All tests compile successfully

### Phase 6: C Compilation Issues (HIGH PRIORITY)

#### 6.1 C Makefile Path Issues
- **Issue**: Test files in wrong locations
- **File**: `c/Makefile`
- **Action**: Fix path references to test files

#### 6.2 C Library Compilation
- **Issue**: `libknhk.a` compilation issues
- **Action**: Verify C library compiles correctly

### Phase 7: Code Quality (LOW PRIORITY - Not Blocking)

#### 7.1 Compiler Warnings
- Run `cargo clippy --workspace -- -D warnings`
- Fix all warnings (24+ reported)

#### 7.2 unwrap() Calls
- 291+ unwrap() calls in production code
- Replace with proper error handling

## Execution Order

1. **Fix blake3 version mismatch** (Quick win)
2. **Run cargo check --workspace** (Verify current state)
3. **Fix known compilation errors** (knhk-sidecar, knhk-warm, knhk-etl)
4. **Verify all dependency versions** (Systematic check)
5. **Verify Cargo.toml structure** (Configuration check)
6. **Verify test compilation** (Test infrastructure)
7. **Fix C compilation issues** (C codebase)
8. **Address code quality issues** (Polish)

## Success Criteria

- ✅ `cargo check --workspace` passes with 0 errors
- ✅ `cargo test --workspace --no-run` passes with 0 errors
- ✅ All dependency versions match workspace versions
- ✅ All Cargo.toml files have correct structure
- ✅ C Makefile paths are correct
- ✅ C library compiles successfully

## Files to Check/Modify

### Rust Packages (13 packages)
- `rust/knhk-hot/Cargo.toml`
- `rust/knhk-otel/Cargo.toml`
- `rust/knhk-connectors/Cargo.toml`
- `rust/knhk-lockchain/Cargo.toml`
- `rust/knhk-unrdf/Cargo.toml` ⚠️ (blake3 version)
- `rust/knhk-etl/Cargo.toml`
- `rust/knhk-warm/Cargo.toml` ⚠️ (compilation errors)
- `rust/knhk-aot/Cargo.toml`
- `rust/knhk-validation/Cargo.toml`
- `rust/knhk-config/Cargo.toml`
- `rust/knhk-sidecar/Cargo.toml` ⚠️ (compilation errors)
- `rust/knhk-cli/Cargo.toml`
- `rust/knhk-integration-tests/Cargo.toml`

### C Codebase
- `c/Makefile` ⚠️ (path issues)

### Source Files with Known Issues
- `rust/knhk-sidecar/src/*.rs` ⚠️ (compilation errors)
- `rust/knhk-warm/src/*.rs` ⚠️ (compilation errors)
- `rust/knhk-etl/src/*.rs` ⚠️ (type mismatches)

## Chicago TDD Approach

1. **Arrange**: Set up verification environment
2. **Act**: Run compilation checks
3. **Assert**: Verify all packages compile successfully
4. **Refactor**: Fix issues found
5. **Repeat**: Re-verify after fixes

This approach tests the actual behavior (compilation) rather than just checking configuration files.

