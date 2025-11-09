# Validate Test Quality

Scans test files for meaningless tests that violate Chicago TDD principles.

## Workflow

1. **Scan Test Files**:
   - Find all test files (`**/*test*.rs`, `**/*_test.rs`)
   - Parse test functions and assertions

2. **Detect Meaningless Tests**:
   - Tests with only `assert_ok!()` or `assert_err!()` without other assertions
   - Tests that don't verify observable outputs
   - Tests with JTBD comments that don't match actual assertions

3. **Report Violations**:
   - List file paths and line numbers
   - Show test names and JTBD comments
   - Suggest what to verify based on test name/JTBD

4. **Exit with Error**:
   - Exit code 1 if violations found (for CI/CD)
   - Exit code 0 if all tests pass validation

## Usage

Run via Cursor command palette: "Validate Test Quality"

Or run script directly:
```bash
./scripts/validate-test-quality.sh
```

## What It Checks

### Pattern 1: Only assert_ok!/assert_err! without behavior verification
```rust
// ❌ BAD: Only checks Ok, doesn't verify behavior
chicago_test!(test_something, {
    let result = do_something();
    assert_ok!(&result); // No other assertions
});
```

### Pattern 2: JTBD comment doesn't match assertions
```rust
// ❌ BAD: Comment says "verify sequential execution" but test doesn't
chicago_test!(test_pattern_1_sequence_jtbd, {
    // JTBD: Execute tasks sequentially
    let result = registry.execute(&PatternId(1), &ctx);
    assert_ok!(&result); // Doesn't verify sequential execution
});
```

### Pattern 3: No observable output verification
```rust
// ❌ BAD: Doesn't verify state changes
#[test]
fn test_register() {
    let result = register(name);
    assert_ok!(&result); // Doesn't verify registration appears in list
}
```

## Output Format

```
Found 5 meaningless tests:

1. rust/knhk-workflow-engine/tests/chicago_tdd_all_43_patterns.rs:61
   Test: test_pattern_1_sequence_jtbd
   JTBD: Execute tasks sequentially, passing data through each step
   Issue: Only checks assert_ok!(), doesn't verify sequential execution
   Suggestion: Verify execution order, task timestamps, or data flow

2. rust/knhk-cli/tests/chicago_tdd_connect.rs:10
   Test: test_connect_register_returns_result
   JTBD: Test behavior (registration) not implementation (storage)
   Issue: Only checks Result type, doesn't verify registration appears in list
   Suggestion: After register(), call list() and verify connector appears
```

## Integration

Add to CI/CD pipeline:
```yaml
- name: Validate Test Quality
  run: ./scripts/validate-test-quality.sh
```

