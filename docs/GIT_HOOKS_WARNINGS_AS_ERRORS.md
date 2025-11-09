# Git Hooks: Warnings as Errors (80/20 Approach)

## Overview

The KNHK git hooks are configured to treat warnings as errors using `-D warnings` flag in clippy checks. This follows the 80/20 principle: focus on high-value warnings while allowing low-value ones.

## Current Configuration

### Pre-commit Hook
- **Location**: `.git/hooks/pre-commit`
- **Clippy Command**: `cargo clippy --package "$pkg" --lib --bins -- -D warnings`
- **Scope**: Only staged packages (fast feedback, 2-5 seconds)
- **Behavior**: Treats warnings as errors, filters test-related warnings

### Pre-push Hook
- **Location**: `.git/hooks/pre-push`
- **Clippy Command**: `cargo clippy --workspace --lib --bins -- -D warnings`
- **Scope**: Full workspace (comprehensive validation, 30-60 seconds)
- **Behavior**: Treats warnings as errors, filters test-related warnings

## 80/20 Strategy

### High-Value Warnings (Treated as Errors) ✅

These warnings are **blocking** and must be fixed:

1. **Unused Variables**
   - Indicates incomplete code or bugs
   - Fixed by prefixing with `_` or removing

2. **Dead Code**
   - Unused functions, structs, fields
   - Fixed by removing or marking with `#[allow(dead_code)]`

3. **Compilation Errors**
   - Type mismatches, missing imports
   - Must be fixed before commit/push

4. **Unwrap/Expect in Production**
   - Checked separately in hooks
   - Must use proper error handling

### Low-Value Warnings (Filtered/Allowed) ⚠️

These warnings are **non-blocking** or filtered:

1. **Deprecated APIs**
   - Already marked with `#[allow(deprecated)]`
   - Filtered out in hook checks

2. **Test Files**
   - Excluded from clippy checks (`--lib --bins`)
   - Test files can use `expect()` with proper allow attributes

3. **Naming Conventions**
   - Static variables should be UPPER_CASE
   - Cosmetic only, doesn't affect functionality

4. **Feature Flags**
   - Configuration warnings (unexpected_cfgs)
   - Not code issues, just configuration

## How It Works

### Pre-commit Hook (Fast Feedback)

```bash
# Only checks staged packages
cargo clippy --package "$pkg" --lib --bins -- -D warnings

# Filters test-related warnings
grep -v "test\|tests\|example\|examples\|bench\|benches"
```

**Behavior**:
- Fast (2-5 seconds)
- Only checks staged files
- Filters test-related warnings
- Blocks on high-value warnings

### Pre-push Hook (Comprehensive)

```bash
# Checks entire workspace
cargo clippy --workspace --lib --bins -- -D warnings

# Filters test-related warnings
grep -v "test\|tests\|example\|examples\|bench\|benches"
```

**Behavior**:
- Comprehensive (30-60 seconds)
- Checks entire workspace
- Filters test-related warnings
- Blocks on high-value warnings

## Examples

### ✅ Allowed (Low-Value)

```rust
#[allow(deprecated)]
fn use_deprecated_api() {
    // Deprecated API usage - allowed with attribute
}

#[cfg(test)]
mod tests {
    #[allow(clippy::expect_used)]
    fn test_example() {
        let result = some_function().expect("test should work");
    }
}
```

### ❌ Blocked (High-Value)

```rust
// ❌ Unused variable - will fail
fn example() {
    let unused = 42;  // Error: unused variable
}

// ✅ Fixed - prefix with underscore
fn example() {
    let _unused = 42;  // OK
}

// ❌ Dead code - will fail
fn unused_function() {  // Error: dead code
    // ...
}

// ✅ Fixed - mark with allow attribute
#[allow(dead_code)]
fn unused_function() {  // OK
    // ...
}
```

## Verification

To verify hooks are working:

```bash
# Test pre-commit hook
git add rust/knhk-cli/src/insights.rs
git commit -m "test"  # Will run pre-commit hook

# Test pre-push hook
git push  # Will run pre-push hook
```

## Summary

- ✅ **Warnings as Errors**: Both hooks use `-D warnings`
- ✅ **80/20 Approach**: High-value warnings blocked, low-value filtered
- ✅ **Fast Feedback**: Pre-commit is fast (2-5s), pre-push comprehensive (30-60s)
- ✅ **Test Files**: Excluded from clippy checks
- ✅ **Deprecated APIs**: Allowed with proper attributes

The hooks are already configured correctly and enforce warnings as errors for high-value issues while allowing low-value ones.



