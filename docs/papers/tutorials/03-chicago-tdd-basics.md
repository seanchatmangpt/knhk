# Tutorial: Chicago TDD Basics

**Level**: Intermediate
**Time**: 25-30 minutes
**Learning Objectives**: Master Test-Driven Development with KNHK's Chicago TDD pattern

## What You'll Learn

By the end of this tutorial, you'll understand:
- The Chicago School of TDD philosophy
- Writing tests before code (red-green-refactor)
- How KNHK's Chicago TDD validates behavior
- Complete testing workflow with real examples
- Performance-first thinking

## Prerequisites

- Completed: [Getting Started with KNHK](01-getting-started-with-knhk.md)
- Completed: [Understanding Telemetry in KNHK](02-understanding-telemetry.md)
- Basic Rust knowledge
- ~30 minutes

## The Chicago School Philosophy

Chicago TDD (vs. London School):
- **Focus**: Real behavior, not mocks
- **Testing**: Integration-focused, behavioral
- **Design**: Emerges from tests
- **Confidence**: Tests prove actual functionality

## Step 1: Understand the Red-Green-Refactor Cycle

### The Cycle

1. **Red** 🔴 - Write a test that fails
2. **Green** 🟢 - Write minimal code to pass
3. **Refactor** 🔵 - Improve without changing behavior
4. **Repeat** ↩️ - Next test

### Example Cycle

```rust
// RED: Write failing test
#[test]
fn test_calculate_total() {
    let items = vec![10, 20, 30];
    let total = calculate_total(&items);
    assert_eq!(total, 60);  // This fails - function doesn't exist
}

// GREEN: Write minimal implementation
fn calculate_total(items: &[i32]) -> i32 {
    items.iter().sum()
}

// REFACTOR: Improve quality
fn calculate_total(items: &[i32]) -> i32 {
    items.iter().sum::<i32>()  // More explicit
}
```

## Step 2: Set Up Your Test File

Create `tests/chicago_tdd_example.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        let result = add(2, 3);
        assert_eq!(result, 5);
    }

    #[test]
    fn test_string_concat() {
        let result = concat_strings("hello", "world");
        assert_eq!(result, "helloworld");
    }

    #[test]
    fn test_edge_case() {
        let result = add(0, 0);
        assert_eq!(result, 0);
    }
}

// Implementation follows tests
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn concat_strings(a: &str, b: &str) -> String {
    format!("{}{}", a, b)
}
```

## Step 3: Run Tests (Red Phase)

Write tests before implementation:

```bash
cargo test --lib
```

Expected: Tests fail ❌

```
test tests::test_addition ... FAILED
```

## Step 4: Implement Minimal Code (Green Phase)

Add just enough code to pass:

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b  // Minimal implementation
}
```

Run tests:
```bash
cargo test --lib
```

Expected: Tests pass ✅

## Step 5: Refactor (Refactor Phase)

Improve without changing behavior:

```rust
#[tracing::instrument]
fn add(a: i32, b: i32) -> i32 {
    let result = a + b;
    debug!("add({}, {}) = {}", a, b, result);
    result
}
```

Run tests again:
```bash
cargo test --lib
```

Expected: Tests still pass ✅

## Step 6: Test Behavior, Not Implementation

Write tests for WHAT, not HOW:

```rust
// ✅ GOOD - Tests behavior
#[test]
fn test_process_returns_correct_result() {
    let result = process_data("input");
    assert_eq!(result.len(), 10);
}

// ❌ BAD - Tests implementation details
#[test]
fn test_process_uses_hashmap() {
    // Don't test internal HashMap usage
    // This breaks when refactoring
}
```

## Step 7: Chicago TDD Pattern in KNHK

KNHK enhances Chicago TDD with:

```rust
#[tracing::instrument]  // Add telemetry
#[test]
fn test_feature_emits_telemetry() {
    // Test actual behavior with real dependencies
    let result = my_function("data");

    assert_eq!(result.status, "success");
    // Telemetry is validated by Weaver
}
```

## Step 8: Complete Example

Here's a complete Chicago TDD cycle:

```rust
#[cfg(test)]
mod calculator_tests {
    use super::*;

    // RED: All tests fail initially
    #[test]
    fn test_add_positive_numbers() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_add_negative_numbers() {
        assert_eq!(add(-2, -3), -5);
    }

    #[test]
    fn test_add_zero() {
        assert_eq!(add(0, 0), 0);
    }

    #[test]
    fn test_subtract() {
        assert_eq!(subtract(5, 3), 2);
    }

    #[test]
    fn test_multiply() {
        assert_eq!(multiply(3, 4), 12);
    }

    #[test]
    fn test_divide() {
        assert_eq!(divide(10, 2), 5);
    }

    #[test]
    fn test_divide_by_zero_returns_error() {
        let result = safe_divide(10, 0);
        assert!(result.is_err());
    }
}

// GREEN: Minimal implementations
#[tracing::instrument]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[tracing::instrument]
fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

#[tracing::instrument]
fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

#[tracing::instrument]
fn divide(a: i32, b: i32) -> i32 {
    a / b
}

#[tracing::instrument]
fn safe_divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

// REFACTOR: Add error handling, telemetry
#[tracing::instrument]
fn safe_divide(a: i32, b: i32) -> Result<i32, DivisionError> {
    if b == 0 {
        warn!("Division by zero attempted");
        Err(DivisionError::DivideByZero)
    } else {
        let result = a / b;
        info!("Division successful: {} / {} = {}", a, b, result);
        Ok(result)
    }
}

#[derive(Debug)]
enum DivisionError {
    DivideByZero,
}
```

## Step 9: Run the Make Target

Run KNHK's Chicago TDD test suite:

```bash
make test-chicago-v04
```

This validates your code follows Chicago TDD patterns.

## Step 10: Performance Testing

Add performance assertions:

```rust
#[test]
fn test_performance_meets_chatman_constant() {
    let start = std::time::Instant::now();

    let result = add(100, 200);

    let duration = start.elapsed();
    assert!(duration.as_nanos() <= 8); // ≤ 8 nanoseconds
    assert_eq!(result, 300);
}
```

Run performance tests:
```bash
make test-performance-v04
```

## Best Practices

### ✅ DO:
- Write tests first
- One assertion per test (when possible)
- Descriptive test names
- Test behavior, not implementation
- Use real dependencies (not mocks)
- Run tests frequently
- Add telemetry to code

### ❌ DON'T:
- Test implementation details
- Mock everything
- Write huge test functions
- Skip edge cases
- Ignore test failures
- Commit untested code

## Test Naming Convention

```rust
#[test]
fn test_<function>_<condition>_<expected_result>() {
    // test_add_two_positive_numbers_returns_sum
}
```

Examples:
- ✅ `test_add_positive_numbers_returns_sum`
- ✅ `test_divide_by_zero_returns_error`
- ✅ `test_process_empty_input_returns_empty`
- ❌ `test1`
- ❌ `test_stuff`

## Debugging Test Failures

```bash
# See detailed output
cargo test --lib failing_test -- --nocapture

# Run sequentially to isolate issues
cargo test --lib failing_test -- --test-threads=1

# With logging
RUST_LOG=debug cargo test --lib failing_test -- --nocapture
```

## Integration with Telemetry

Chicago TDD + Telemetry = Behavior Proof:

```rust
#[test]
fn test_function_emits_required_telemetry() {
    // Run function
    let result = my_function("data");

    // Verify behavior
    assert_eq!(result.status, "success");

    // Telemetry automatically validated by Weaver
    // No manual validation needed
}
```

## Common Test Patterns

### Arrange-Act-Assert (AAA)
```rust
#[test]
fn test_aaa_pattern() {
    // Arrange: Set up
    let input = vec![1, 2, 3];

    // Act: Execute
    let result = sum_vector(&input);

    // Assert: Verify
    assert_eq!(result, 6);
}
```

### Testing Error Cases
```rust
#[test]
fn test_error_case() {
    let result = risky_operation("invalid");

    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), ErrorKind::Invalid);
    }
}
```

## What You've Learned

Congratulations! You now understand:

1. **Chicago School TDD** - Real behavior testing
2. **Red-Green-Refactor** - The TDD cycle
3. **Test-First Development** - Why it works
4. **Behavior Testing** - What to test
5. **Integration with Telemetry** - Proving behavior

## Next Steps

Ready to implement features with TDD?
- See: [How to Add New Features](../how-to-guides/04-add-new-features.md)
- See: [How to Run Tests Efficiently](../how-to-guides/02-run-tests-efficiently.md)

Want to optimize performance?
- See: [Optimizing Performance](../tutorials/04-optimizing-performance.md) (coming soon)
- See: [How to Optimize Performance](../how-to-guides/07-optimize-performance.md) (coming soon)

## Key Commands

```bash
# Run tests
cargo test --lib

# Run Chicago TDD suite
make test-chicago-v04

# Run with output
cargo test --lib -- --nocapture

# Run single test
cargo test --lib test_name

# Debug with logging
RUST_LOG=debug cargo test --lib test_name -- --nocapture
```

## Resources

- **Rust Testing**: https://doc.rust-lang.org/book/ch11-00-testing.html
- **Chicago School TDD**: https://artisanworks.jp/files/ChicagoSchoolTDD.pdf
- **KNHK Tests**: `tests/` directory in repository

---

**You are here**: Tutorial (Learning-oriented)
**Framework**: Diátaxis
**Tutorial Duration**: ~30 minutes
**Difficulty**: Intermediate
**Prerequisites**: Getting Started, Understanding Telemetry
