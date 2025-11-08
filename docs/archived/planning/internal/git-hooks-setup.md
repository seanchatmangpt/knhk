# Git Hooks Setup - Poka-Yoke for KNHK

## Overview

KNHK uses **poka-yoke** (mistake-proofing) git hooks to enforce Definition of Done requirements at commit-time. These hooks prevent common errors from entering the codebase:

- **No `unwrap()` calls** in production code
- **No `unimplemented!()` placeholders**
- **Zero clippy warnings**
- **Properly formatted code**
- **All tests passing**

## Installation

### Quick Install

```bash
# From project root
./scripts/install-git-hooks.sh
```

### Manual Install

If the script doesn't work, manually copy hooks:

```bash
# From project root
cp scripts/hooks/pre-commit .git/hooks/pre-commit
cp scripts/hooks/pre-push .git/hooks/pre-push
chmod +x .git/hooks/pre-commit
chmod +x .git/hooks/pre-push
```

## What the Hooks Do

### Pre-Commit Hook

Runs on every `git commit`:

1. **Blocks `unwrap()` calls** in production Rust code
   - Requires proper `Result<T, E>` error handling
   - Allows `unwrap()` in test code

2. **Blocks `unimplemented!()` placeholders**
   - Forces completion of implementations
   - Suggests using `todo!()` for acknowledged technical debt

3. **Warns about `expect()` calls**
   - Non-blocking warning for potential panics
   - Encourages using `?` operator or match

4. **Runs `cargo clippy --workspace`**
   - Blocks commit if any clippy warnings
   - Ensures code quality standards

5. **Checks formatting**
   - Blocks commit if code isn't formatted
   - Requires `cargo fmt --all` before commit

### Pre-Push Hook (5-Gate Validation)

Runs on every `git push`:

**Gate 1: Cargo Check**
- Verifies all code compiles
- Catches compilation errors early

**Gate 2: Clippy (Strict Mode)**
- Zero warnings allowed
- Enforces code quality

**Gate 3: Formatting Check**
- All code must be formatted
- Consistent style across codebase

**Gate 4: Fast Tests**
- Runs lib and bin tests only
- Skips slow integration tests
- Ensures core functionality works

**Gate 5: Security Audit**
- Runs `cargo audit` if installed
- Non-blocking warnings
- Alerts about known vulnerabilities

**Bonus: DoD Validation**
- Runs full Definition of Done validation
- Checks all DoD criteria
- Non-blocking but reports issues

## Testing the Hooks

Run the automated test suite:

```bash
./scripts/test-git-hooks.sh
```

### Manual Testing

**Test 1: Verify `unwrap()` is blocked**

```bash
# Create test file with unwrap()
cat > rust/test.rs << 'EOF'
pub fn bad() {
    let x = Some(1).unwrap(); // Should be blocked
}
EOF

git add rust/test.rs
git commit -m "test"
# Should see: "❌ ERROR: Cannot commit 1 unwrap() calls"

# Clean up
git reset HEAD rust/test.rs
rm rust/test.rs
```

**Test 2: Verify proper error handling is allowed**

```bash
# Create test file with proper error handling
cat > rust/test.rs << 'EOF'
pub fn good() -> Result<(), Box<dyn std::error::Error>> {
    let x = Some(1).ok_or("error")?;
    Ok(())
}
EOF

git add rust/test.rs
git commit -m "test"
# Should succeed with clippy/fmt checks

# Clean up
git reset --soft HEAD~1
git reset HEAD rust/test.rs
rm rust/test.rs
```

## Bypassing Hooks (Not Recommended)

If you absolutely must bypass hooks (e.g., WIP commits):

```bash
# Skip pre-commit hook (NOT RECOMMENDED)
git commit --no-verify -m "WIP: incomplete"

# Skip pre-push hook (NOT RECOMMENDED)
git push --no-verify
```

⚠️ **Warning:** Bypassing hooks violates the Definition of Done and may break CI/CD.

## Troubleshooting

### Hook Not Running

```bash
# Check hook exists and is executable
ls -la .git/hooks/pre-commit
ls -la .git/hooks/pre-push

# Make executable if needed
chmod +x .git/hooks/pre-commit
chmod +x .git/hooks/pre-push
```

### Hook Fails on Valid Code

If the hook blocks code that should be allowed:

1. Check if `unwrap()` is truly necessary (usually isn't)
2. For test code, ensure it's in a `#[test]` or `#[cfg(test)]` block
3. Run `cargo clippy` and `cargo fmt` manually to see issues
4. Report false positives to the team

### Hook Takes Too Long

The pre-push hook runs full workspace tests. To speed up:

```bash
# Only run specific tests
cargo test --lib --bins --package knhk-etl

# Or bypass for emergency pushes (not recommended)
git push --no-verify
```

## Best Practices

1. **Commit early, commit often**
   - Hooks ensure each commit meets quality standards
   - Easier to debug when issues caught early

2. **Run tests locally before committing**
   - Hooks run tests, but pre-running saves time
   - `cargo test --workspace --lib --bins`

3. **Keep commits focused**
   - Smaller commits pass hooks faster
   - Easier to review and debug

4. **Format code before committing**
   - `cargo fmt --all` before `git add`
   - Avoids hook blocking on formatting

5. **Fix clippy warnings immediately**
   - Don't let warnings accumulate
   - `cargo clippy --workspace -- -D warnings`

## Configuration

### Customizing Hooks

Edit hooks in `.git/hooks/` or modify the installation script:

```bash
# Edit installation template
vim scripts/install-git-hooks.sh

# Reinstall hooks
./scripts/install-git-hooks.sh
```

### Team Consistency

All team members should install hooks:

```bash
# Add to onboarding checklist
./scripts/install-git-hooks.sh
./scripts/test-git-hooks.sh
```

## Integration with CI/CD

Hooks enforce local quality, but CI/CD provides final validation:

```yaml
# .github/workflows/ci.yml
- name: Run DoD Validation
  run: ./scripts/validate-dod-v1.sh

- name: Check No Unwrap
  run: |
    if git grep -n '\.unwrap()' -- '*.rs' ':!tests'; then
      echo "❌ Found unwrap() calls"
      exit 1
    fi
```

## References

- [Definition of Done](/Users/sac/knhk/docs/archived/v1-dod/DEFINITION_OF_DONE.md)
- [Validation Script](/Users/sac/knhk/scripts/validate-dod-v1.sh)
- [Poka-Yoke Concept](https://en.wikipedia.org/wiki/Poka-yoke)
