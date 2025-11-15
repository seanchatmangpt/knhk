# How-to Guide: Debug Failing Tests

**Goal**: Systematically identify and fix test failures
**Time**: 15-45 minutes (depends on complexity)
**Difficulty**: Intermediate

## Quick Triage

When a test fails:

```bash
# Step 1: Run the specific test
cargo test --lib failing_test_name -- --nocapture

# Step 2: Check error message
# Step 3: Use the decision tree below
```

## Common Failure Patterns

### Pattern 1: Assertion Failed

**Symptom**:
```
assertion failed: result == expected
   left: `5`,
  right: `10`
```

**Debug**:
```bash
# Show actual values
cargo test --lib failing_test -- --nocapture --test-threads=1

# Add debug assertions
println!("Actual: {:?}", result);
println!("Expected: {:?}", expected);
assert_eq!(result, expected);
```

**Fix**:
- Check calculation logic
- Verify input data
- Look for off-by-one errors

### Pattern 2: Panic in Test

**Symptom**:
```
thread 'test_name' panicked at 'attempt to subtract with overflow'
```

**Debug**:
```bash
# Use RUST_BACKTRACE for details
RUST_BACKTRACE=1 cargo test --lib failing_test -- --nocapture

# Add intermediate assertions
let value = compute_something();
assert!(value > 0, "Value should be positive: {}", value);
```

**Fix**:
- Add bounds checking
- Handle edge cases
- Verify assumptions

### Pattern 3: Timeout

**Symptom**:
```
test timed out after 60 seconds
```

**Debug**:
```bash
# Run sequentially (no parallelization)
cargo test --lib failing_test -- --test-threads=1

# Add logging to find where it hangs
RUST_LOG=trace cargo test --lib failing_test -- --nocapture --test-threads=1
```

**Fix**:
- Look for infinite loops
- Check for deadlocks
- Verify resource cleanup

### Pattern 4: Async/Await Issue

**Symptom**:
```
error: block_on called from async context
```

**Debug**:
```rust
#[tokio::test]  // Use tokio::test for async tests
async fn test_async() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

**Fix**:
- Use `#[tokio::test]` for async code
- Await all futures
- Don't use `block_on` inside async

## Step-by-Step Debugging

### Step 1: Run with Full Output

```bash
cargo test --lib failing_test -- --nocapture --test-threads=1
```

See:
- What the test does
- Where it fails
- What values are involved

### Step 2: Add Strategic Logging

```rust
#[test]
fn test_calculation() {
    let input = 5;
    println!("Input: {}", input);  // Debug output

    let result = compute(input);
    println!("Result: {}", result);  // See what happened

    assert_eq!(result, 10);
}
```

Run:
```bash
cargo test --lib test_calculation -- --nocapture
```

### Step 3: Simplify Test

Reduce to minimal reproduction:

```rust
// ❌ BEFORE: Complex test
#[test]
fn test_complex() {
    let data = load_from_file("large.csv");
    let processed = process(data);
    let results = analyze(processed);
    assert_eq!(results.count, 1000);
}

// ✅ AFTER: Simplified
#[test]
fn test_simple() {
    let data = vec![1, 2, 3];
    let result = compute(&data);
    println!("Result: {}", result);
    assert_eq!(result, 6);
}
```

### Step 4: Check Assumptions

```rust
#[test]
fn test_with_assumptions() {
    let data = vec![];

    // Validate assumptions
    assert!(!data.is_empty(), "Data should not be empty");

    let result = process(&data);
    assert_eq!(result.len(), 1);
}
```

If the assertion before your test code fails, you found the problem.

### Step 5: Use Debugger

```bash
# Install rust-gdb
rustup component add rust-gdb

# Run test in debugger
rust-gdb --args target/debug/deps/test_binary-hash

# Inside gdb:
# (gdb) break test_name
# (gdb) run
# (gdb) next
# (gdb) print variable_name
```

## Decision Tree

```
Test fails
  │
  ├─ Shows error message?
  │  ├─ Assertion failure?
  │  │  └─ Check actual vs expected values
  │  ├─ Panic?
  │  │  └─ Add RUST_BACKTRACE=1
  │  └─ Unknown error?
  │     └─ Add --nocapture to see output
  │
  ├─ Hangs/times out?
  │  ├─ Check for infinite loops
  │  ├─ Check for deadlocks
  │  └─ Run with --test-threads=1
  │
  ├─ Flaky (sometimes passes)?
  │  ├─ Check for race conditions
  │  ├─ Check for time-dependent code
  │  └─ Use --test-threads=1
  │
  └─ Async issues?
     ├─ Use #[tokio::test]
     └─ Make sure to .await
```

## Common Mistakes

❌ **Not running with --nocapture**
```bash
# ❌ Won't see output
cargo test --lib my_test

# ✅ Will see output
cargo test --lib my_test -- --nocapture
```

❌ **Running parallel tests with shared state**
```bash
# ❌ Tests interfere with each other
cargo test --lib

# ✅ Run sequentially
cargo test --lib -- --test-threads=1
```

❌ **Asserting on implementation details**
```rust
// ❌ Tests internal structure
#[test]
fn test_impl() {
    let v = vec![1, 2, 3];
    assert_eq!(v.capacity(), 3);  // May vary
}

// ✅ Tests behavior
#[test]
fn test_behavior() {
    let v = vec![1, 2, 3];
    assert_eq!(v.len(), 3);
}
```

❌ **Using unwrap() without checking**
```rust
// ❌ Will panic if None
let value = option.unwrap();

// ✅ Check first
let value = option.expect("Should have value");

// ✅ Or assert
assert!(option.is_some());
let value = option.unwrap();
```

## Debugging Strategies

### Strategy 1: Binary Search

If a test fails inconsistently:

```rust
#[test]
fn test_large_computation() {
    let inputs = vec![/* 1000 items */];

    // Test first half
    let half = inputs.len() / 2;
    let result = process(&inputs[0..half]);
    assert!(result.is_ok());  // Find which half fails

    // Test second half
    let result = process(&inputs[half..]);
    assert!(result.is_ok());
}
```

### Strategy 2: Instrumentation

```rust
#[test]
fn test_with_logging() {
    // Enable logging for test
    let _ = env_logger::builder().is_test(true).try_init();

    info!("Test starting");
    let result = my_function();
    debug!("Result: {:?}", result);

    assert!(result.is_ok());
}

// Run:
// RUST_LOG=debug cargo test --lib test_with_logging -- --nocapture
```

### Strategy 3: Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_property(x in 0i32..100) {
        // This property must hold for all values
        assert!(my_function(x) >= 0);
    }
}
```

## Advanced Debugging

### Test with MIRI (Undefined Behavior)

```bash
cargo +nightly miri test --lib my_test
```

Detects:
- Memory safety issues
- Undefined behavior
- Data race conditions

### Flamegraph (Performance Issues)

```bash
cargo flamegraph --test failing_test
open flamegraph.svg
```

Shows where time is spent.

### Address Sanitizer (Memory Issues)

```bash
RUSTFLAGS="-Z sanitizer=address" cargo test --lib my_test
```

Detects:
- Memory leaks
- Use-after-free
- Buffer overflows

## Test-Specific Debugging

### For Async Tests

```rust
#[tokio::test]
async fn test_async() {
    let result = async_function().await;

    // Timeout after 5 seconds if it hangs
    tokio::time::timeout(
        Duration::from_secs(5),
        async_operation()
    ).await.expect("Operation timed out");
}
```

### For Database Tests

```rust
#[tokio::test]
async fn test_database() {
    let db = setup_test_db().await;

    // Use test-specific data
    let user = db.create_user("test_user").await?;

    // Clean up after
    db.delete_user(&user.id).await?;

    assert!(result);
}
```

### For Integration Tests

```rust
#[test]
fn test_integration() {
    let server = start_test_server();
    let client = create_test_client();

    let response = client.get("/api/endpoint");

    assert_eq!(response.status(), 200);

    server.shutdown();
}
```

## Next Steps

- **Fix the test**: Apply the pattern above
- **Verify fix**: Run all tests
- **Prevent recurrence**: Add assertions

## Key Commands

```bash
# Run test with output
cargo test --lib test_name -- --nocapture

# Run test sequentially
cargo test --lib test_name -- --test-threads=1

# Run with logging
RUST_LOG=debug cargo test --lib test_name -- --nocapture

# Run with backtrace
RUST_BACKTRACE=1 cargo test --lib test_name

# Run in debugger
rust-gdb --args target/debug/deps/test_binary

# Run with memory checks
RUSTFLAGS="-Z sanitizer=address" cargo test --lib test_name
```

---

**Category**: How-to Guides (Task-oriented)
**Framework**: Diátaxis
**Difficulty**: Intermediate
**Related**: Testing, Running Tests Efficiently
