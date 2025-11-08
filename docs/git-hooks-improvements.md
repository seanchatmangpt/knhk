# Git Hooks Improvements - Core Team Best Practices Alignment

## Summary

The git hooks have been updated to align with core team 80/20 best practices: fast feedback loops, pragmatic exceptions, and focus on critical path enforcement. The improvements focus on:

1. **Faster pre-commit** - Only checks staged files/packages (2-5s target vs 30-60s)
2. **Pragmatic exceptions** - CLI code, build scripts, and test files with proper attributes
3. **Smarter validation** - Distinguishes between production library code vs CLI/test/build code
4. **Core practice alignment** - Enforces critical path (production library code), allows documented exceptions

## Key Improvements

### 1. Pre-Commit Hook Enhancements

**Before:**
- Ran clippy on entire workspace (slow)
- Blocked on all expect() calls, even in test files
- No distinction between production and test code

**After:**
- Only checks staged files/packages (faster)
- Allows `expect()` in test files with `#![allow(clippy::expect_used)]`
- Allows CLI code (`knhk-cli`) to use `expect()` for user-facing errors
- Exempts build scripts (`build.rs`) from checks
- Skips test/example/bench directories automatically
- Respects `#[allow]` attributes in production code

**Benefits:**
- Faster commits (2-5 seconds vs 30-60 seconds) - aligns with "incremental builds ≤2s"
- Test code can use `expect()` for clearer test failures
- CLI code can use `expect()` for user errors (pragmatic exception)
- Build scripts exempt (build-time only, not runtime)
- Production library code still strictly enforced

### 2. Pre-Push Hook Enhancements

**Before:**
- Blocked on all clippy warnings, including test files
- No distinction between production and test code
- Required fixing test file warnings before push

**After:**
- Allows test files with proper `#[allow]` attributes
- Allows CLI code to use `expect()` (documented exception)
- Exempts build scripts from unwrap/expect checks
- Still strict on production library code
- Better error messages explaining policy and exceptions

**Benefits:**
- Can push with test files that use `expect()` properly
- CLI code can use `expect()` for user-facing errors
- Build scripts exempt (not part of runtime code path)
- Production library code still strictly enforced
- Clearer error messages explaining exceptions

### 3. Core Team Best Practices Alignment

The improved hooks now align with core team 80/20 philosophy:

✅ **No unwrap()/expect() in production library code** - Strictly enforced (critical path)
✅ **CLI code exception** - Can use `expect()` for user-facing errors (documented)
✅ **Build scripts exempt** - `build.rs` files excluded (build-time only)
✅ **Test code can use expect() with allow attributes** - Aligned with practice
✅ **No placeholders** - Still blocks `unimplemented!()`
✅ **Proper error handling** - Enforces `Result<T, E>` patterns in library code
✅ **Fast feedback** - Pre-commit only checks staged files (2-5s target)
✅ **Performance focus** - Pre-commit fast, pre-push comprehensive (30-60s acceptable)

## Usage

### Installing Improved Hooks

```bash
# Install improved hooks (aligned with core team best practices)
./scripts/install-git-hooks.sh
```

### Test File Pattern

Test files can now use `expect()` with proper allow attributes:

```rust
#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]  // ✅ Allowed in test modules
    
    #[test]
    fn test_something() {
        let result = some_function().expect("Should succeed in test");
        // ...
    }
}
```

### Production Library Code Pattern

Production library code must use proper error handling:

```rust
// ❌ Blocked by hook
let value = some_function().unwrap();

// ✅ Allowed
let value = some_function()?;

// ✅ Allowed with explicit error handling
let value = match some_function() {
    Ok(v) => v,
    Err(e) => return Err(e.into()),
};
```

### CLI Code Pattern (Exception)

CLI code can use `expect()` for user-facing errors:

```rust
// ✅ Allowed in knhk-cli (user-facing, different needs)
let config = load_config().expect("Failed to load configuration file");
// User-friendly error message is appropriate for CLI
```

### Build Script Pattern (Exception)

Build scripts (`build.rs`) are exempt from checks:

```rust
// ✅ Allowed in build.rs (build-time only, not runtime)
let version = env::var("CARGO_PKG_VERSION").unwrap();
// Build scripts run at compile time, not in production
```

## Migration Guide

### For Existing Code

1. **Test files with expect()**: Add `#![allow(clippy::expect_used)]` to test modules
2. **Production code with expect()**: Replace with proper error handling or add allow attribute with justification
3. **No changes needed** for code already following best practices

### For New Code

1. **Production code**: Always use `Result<T, E>` and `?` operator
2. **Test code**: Can use `expect()` with `#![allow(clippy::expect_used)]` in test modules
3. **Error handling**: Use match or `?` operator, never unwrap/expect

## Comparison: Old vs New

| Feature | Old Hooks | New Hooks |
|---------|-----------|-----------|
| Pre-commit speed | 30-60s (full workspace) | 2-5s (staged files only) |
| Pre-push speed | 30-60s | 30-60s (acceptable, comprehensive) |
| Test file expect() | Blocked | Allowed with `#[allow]` |
| CLI code expect() | Blocked | Allowed (documented exception) |
| Build scripts | Checked | Exempt (build-time only) |
| Production library expect() | Blocked | Blocked (strict) |
| Clippy scope (pre-commit) | Full workspace | Staged packages only |
| Clippy scope (pre-push) | Full workspace | Full workspace (lib+bins) |
| Test file handling | Same as production | Separate rules |

## Benefits

1. **Faster development** - Pre-commit is 10x faster (2-5s vs 30-60s)
2. **Pragmatic exceptions** - CLI code and build scripts handled appropriately
3. **Better test code** - Can use `expect()` for clearer failures
4. **Strict production library code** - Still enforces no unwrap/expect in critical path
5. **Core practice alignment** - Matches 80/20 philosophy (focus on critical path)
6. **Better UX** - Clearer error messages explaining exceptions
7. **Fast feedback** - Aligns with "incremental builds ≤2s" SLO

## Exception Patterns

### CLI Code Exception

**Rationale**: CLI code is user-facing and benefits from `expect()` for user-friendly error messages. Different needs than library code.

**Pattern**:
```rust
// knhk-cli/src/commands/*.rs
let config = load_config().expect("Failed to load configuration file");
// ✅ Allowed - user-facing error message
```

**Policy**: CLI code (`knhk-cli/src/**/*.rs`) can use `expect()` but not `unwrap()`. Still must use proper error handling for library calls.

### Build Script Exception

**Rationale**: Build scripts run at compile time, not runtime. Not part of production code path.

**Pattern**:
```rust
// **/build.rs
let version = env::var("CARGO_PKG_VERSION").unwrap();
// ✅ Allowed - build-time only
```

**Policy**: All `build.rs` files are exempt from unwrap/expect checks.

### Test File Exception

**Rationale**: Test code should fail fast with clear messages. `expect()` provides better test failures than `unwrap()`.

**Pattern**:
```rust
#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]  // ✅ Required for expect()
    
    #[test]
    fn test_something() {
        let result = function().expect("Should succeed in test");
    }
}
```

**Policy**: Test files can use `expect()` with `#![allow(clippy::expect_used)]` at module level.

### Default Trait Fallback Exception

**Rationale**: Default trait implementations cannot return `Result`. Documented exception pattern.

**Pattern**:
```rust
// knhk-warm/src/graph.rs
#![allow(clippy::expect_used)]  // Documented: Default trait fallback

impl Default for WarmPathGraph {
    fn default() -> Self {
        let query_cache_size = NonZeroUsize::new(1000).expect("1000 is non-zero");
        // ✅ Allowed - Default trait cannot return Result
    }
}
```

**Policy**: Default trait implementations can use `expect()` with module-level allow attribute and documentation.

## Performance Targets

Aligned with core team SLOs:

- **Pre-commit**: 2-5 seconds (target: <5s)
  - Only checks staged files/packages
  - Aligns with "incremental builds ≤2s" SLO
  
- **Pre-push**: 30-60 seconds (acceptable)
  - Comprehensive workspace validation
  - Full test suite, clippy, formatting
  - Appropriate for push gate

## Next Steps

1. Review and test the improved hooks
2. Update team documentation
3. Migrate existing test files to use allow attributes
4. Document CLI code exceptions in codebase
5. Performance/OTEL validation remains in tests/CI (not hooks)

## References

- [Core Team Best Practices](../.cursor/rules/build-system-practices.mdc)
- [Chicago TDD Standards](../.cursor/rules/chicago-tdd-standards.mdc)
- [Original Git Hooks Setup](./git-hooks-setup.md)

