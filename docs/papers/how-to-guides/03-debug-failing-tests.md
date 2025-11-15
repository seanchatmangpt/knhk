# How-to Guide 3: Debug Failing Tests

## Goal

Systematically identify and fix failing tests by understanding error messages, using debugging tools, and applying common solutions.

**Time Estimate**: 15-30 minutes per issue
**Prerequisites**: [Setup Development Environment](01-setup-development-environment.md), [Run Tests Efficiently](02-run-tests-efficiently.md)
**Outcomes**: Ability to quickly diagnose and fix test failures

---

## The Debugging Workflow

```
Test Fails → Read Error → Locate Code → Analyze → Fix → Verify
```

---

## Step 1: Run the Failing Test

### Run Specific Failing Test

```bash
# Run exact failing test
cargo test test_name -- --exact

# Run tests matching pattern
cargo test pattern_name

# Run with output
cargo test -- --nocapture
cargo test -- --nocapture --test-threads=1
```

### Get Full Error Output

```bash
# Show all output (not just summary)
cargo test -- --nocapture --test-threads=1

# Get backtrace
RUST_BACKTRACE=1 cargo test

# Get full backtrace
RUST_BACKTRACE=full cargo test
```

### Example Output with Backtrace

```
thread 'tests::my_test' panicked at 'assertion failed: expected == actual', src/lib.rs:42:5

stack backtrace:
   0: rust_begin_unwind
   1: core::panicking::panic_fmt
   2: my_module::tests::my_test
   3: core::ops::function::FnOnce::call_once
```

---

## Step 2: Understand Common Error Types

### Assertion Failures

```
thread 'test_name' panicked at 'assertion failed: x == y'
```

**Cause**: `assert_eq!(x, y)` where `x != y`

**Debug**:
```bash
# Print actual values
cargo test -- --nocapture

# Add debug output
println!("Expected: {:?}", expected);
println!("Actual: {:?}", actual);

# Run with RUST_BACKTRACE
RUST_BACKTRACE=1 cargo test
```

**Fix**: Verify the expected values are correct

### Panic in Code

```
thread 'test_name' panicked at 'unwrap failed on value None'
```

**Cause**: `.unwrap()` or `.expect()` called on `None` or `Err`

**Debug**:
```rust
// Instead of:
let value = some_option.unwrap();  // Will panic!

// Use pattern matching:
let value = match some_option {
    Some(v) => v,
    None => {
        eprintln!("Value was None!");
        return Err("missing value");
    }
};

// Or use ? operator:
let value = some_option?;
```

**Fix**: Handle the error case properly

### Compile Error

```
error[E0425]: cannot find value `x` in this scope
```

**Cause**: Variable doesn't exist or isn't imported

**Debug**:
```bash
cargo check  # Faster than full compile

# Look at line numbers in error
```

**Fix**:
```rust
// Add missing import
use my_module::my_type;

// Or define variable
let x = 5;
```

### Lifetime Error

```
error[E0106]: missing lifetime specifier
```

**Cause**: References without specified lifetimes

**Debug**:
```bash
# Read error message carefully
cargo check
```

**Fix**:
```rust
// Add explicit lifetime
fn my_func<'a>(x: &'a str) -> &'a str {
    x
}

// Or let compiler infer (often works)
fn my_func(x: &str) -> &str {
    x
}
```

### Test Timeout

```
test timed out after 60 seconds
```

**Cause**: Test runs longer than timeout (usually infinite loop)

**Debug**:
```bash
# Add debug output to find where it hangs
println!("Reached point A");
println!("Reached point B");
// ... narrow down location

# Or use timeout
timeout 5 cargo test test_name
```

**Fix**:
```rust
// Check for infinite loops
// while condition_always_true {  // Wrong!
// }

// while condition_becomes_false {  // Correct
//     condition = update_condition();
// }

// Increase timeout if intentional long-running test
#[tokio::test(flavor = "multi_thread")]
async fn slow_test() {
    // test code
}
```

---

## Step 3: Use Debugging Techniques

### Add Debug Output

```rust
// Simple println debugging
println!("Variable x = {:?}", x);
println!("Result = {:#?}", result);  // Pretty print

// Use dbg! macro
dbg!(x);
dbg!(&some_variable);

// In tests:
#[test]
fn my_test() {
    let x = 5;
    eprintln!("Debug output: {:?}", x);  // Goes to stderr
    assert_eq!(x, 5);
}
```

### Run with Output

```bash
# See println! and eprintln! output
cargo test -- --nocapture

# Run single-threaded (output in order)
cargo test -- --nocapture --test-threads=1

# Combine both
cargo test my_test -- --nocapture --test-threads=1
```

### Use a Debugger

#### With VS Code and rust-analyzer

1. Install `CodeLLDB` extension
2. Add `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Test",
      "cargo": {
        "args": [
          "test",
          "--no-run",
          "--lib",
          "my_module::tests::my_test"
        ],
        "filter": {
          "name": "my_test",
          "kind": "test"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

3. Set breakpoints and run debug session

#### With gdb (Linux)

```bash
# Compile with debug info
RUST_BACKTRACE=1 cargo test --no-run my_test

# Run in debugger
gdb ./target/debug/deps/my_test-abc123def456

# In gdb:
(gdb) break my_module.rs:42
(gdb) run
(gdb) print x
(gdb) continue
(gdb) quit
```

### Use rust-gdb Script

```bash
# Create debugging script
cat > debug.sh << 'EOF'
#!/bin/bash
RUST_BACKTRACE=1 cargo test --no-run $1
rust-gdb ./target/debug/deps/$1-*
EOF

chmod +x debug.sh
./debug.sh my_test
```

---

## Step 4: Common Test Failures & Solutions

### Type Mismatch

**Error**:
```
expected `u32`, found `i32`
```

**Solution**:
```rust
// Check types match
let x: u32 = 5;     // Correct
let y: u32 = 5i32;  // Convert: as u32
let z = 5u32;       // Explicit type suffix
```

### Missing Trait Implementation

**Error**:
```
error[E0277]: `MyType` doesn't implement `Debug`
```

**Solution**:
```rust
// Add derive macro
#[derive(Debug)]
struct MyType {
    field: i32,
}

// Or implement manually
impl Debug for MyType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "MyType {{ ... }}")
    }
}
```

### Test Data Setup Issues

**Problem**: Test setup fails
**Solution**:
```rust
#[test]
fn test_with_setup() {
    // Setup
    let mut state = State::new();

    // Verify setup succeeded
    assert!(state.is_valid());

    // Test
    let result = state.do_something();
    assert_eq!(result, expected);
}

// Or use fixtures
fn setup() -> State {
    State::new()
}

#[test]
fn test_using_fixture() {
    let state = setup();
    // test code
}
```

### Async Test Failures

**Error**:
```
error[E0752]: `test` functions cannot be declared `async`
```

**Solution**:
```rust
// Use tokio test macro
#[tokio::test]
async fn async_test() {
    let result = async_function().await;
    assert_eq!(result, expected);
}

// Or use block_on
#[test]
fn async_test_sync() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let result = async_function().await;
        assert_eq!(result, expected);
    });
}
```

### State Persistence Between Tests

**Problem**: Tests interfere with each other
**Cause**: Shared mutable state

**Solution**:
```rust
// Use thread-local or per-test setup
#[test]
fn test_1() {
    let mut state = State::new();  // Fresh state
    // test
}

#[test]
fn test_2() {
    let mut state = State::new();  // Fresh state
    // test
}

// Or use test fixture that resets state
fn with_clean_state<F: Fn()>(f: F) {
    let state = State::new();
    f();
    // cleanup
}
```

---

## Step 5: Fix and Verify

### Make the Fix

```rust
// Original (broken)
fn calculate(x: i32) -> i32 {
    x + 5  // Wrong calculation
}

// Fixed
fn calculate(x: i32) -> i32 {
    x * 5  // Correct calculation
}
```

### Run the Test Again

```bash
# Run the specific test
cargo test test_name -- --exact --nocapture

# Expected output:
# test test_name ... ok
```

### Run All Related Tests

```bash
# Run all tests in module
cargo test --lib my_module

# Run all tests
cargo test --workspace
```

---

## Step 6: Prevent Future Failures

### Add More Assertions

```rust
#[test]
fn comprehensive_test() {
    // Test multiple aspects
    assert!(result.is_valid());
    assert_eq!(result.value, expected_value);
    assert!(result.data.len() > 0);
    assert_eq!(result.status, Status::Success);
}
```

### Document Expected Behavior

```rust
#[test]
fn test_with_documentation() {
    // Given: A valid input
    let input = valid_input();

    // When: We process it
    let result = process(input);

    // Then: We expect specific output
    assert_eq!(result, expected);
}
```

### Test Edge Cases

```rust
#[test]
fn test_edge_cases() {
    // Test with empty input
    assert_eq!(process(&[]), empty_result);

    // Test with maximum input
    assert_eq!(process(&[MAX; 1000]), max_result);

    // Test with invalid input
    assert!(process_strict(&[]).is_err());
}
```

---

## Step 7: Special Cases

### Weaver Validation Failures

**Error**:
```
weaver registry live-check failed
```

**Cause**: Runtime telemetry doesn't match schema

**Debug**:
```bash
# Check schema is valid
weaver registry check -r registry/

# View actual telemetry
# Add tracing output
RUST_LOG=debug cargo test

# Check schema matches telemetry
```

**Fix**:
```bash
# Option 1: Fix code to match schema
// Add missing telemetry emissions

# Option 2: Update schema to match code
// Edit registry/schemas/*.yaml
```

### Performance Test Failures

**Error**:
```
operation_name: 12 ticks (exceeds limit of 8)
```

**Cause**: Code takes too long

**Debug**:
```bash
# Profile with flamegraph
cargo install flamegraph
cargo flamegraph --test test_name

# Shows where time is spent
```

**Fix**:
```rust
// Optimize algorithm
// Use better data structures
// Cache results
// Parallelize where possible
```

---

## Debugging Checklist

- [ ] Read error message completely
- [ ] Identify error type (assertion, panic, compile, etc.)
- [ ] Run test with `--nocapture` to see output
- [ ] Run test with `RUST_BACKTRACE=1` to see stack trace
- [ ] Add `println!` or `dbg!` statements
- [ ] Check variable types and values
- [ ] Review code at error location
- [ ] Check test data setup
- [ ] Verify dependencies are correct
- [ ] Try running simpler version first
- [ ] Check if test passes without changes
- [ ] Review git diff to understand recent changes

---

## Quick Reference

| Error Type | Check | Solution |
|------------|-------|----------|
| Assertion fails | Expected value correct? | Verify logic |
| Panic (unwrap) | Is value None/Err? | Use pattern matching or ? |
| Compile error | Missing import? | Add `use` statement |
| Type mismatch | Types match? | Cast or change type |
| Test timeout | Infinite loop? | Check loop conditions |
| Async failure | Using `#[tokio::test]`? | Add attribute |
| Weaver failure | Telemetry matches schema? | Fix code or schema |

---

## Next Steps

1. **Run Tests Efficiently** - [How-to: Run Tests Efficiently](02-run-tests-efficiently.md)
2. **Add New Features** - [How-to: Add New Features](04-add-new-features.md) (coming soon)
3. **Optimize Performance** - [How-to: Optimize Performance](11-optimize-performance.md) (coming soon)

---

**Created**: 2025-11-15
**Updated**: 2025-11-15
**Status**: Complete
**Difficulty**: Intermediate
