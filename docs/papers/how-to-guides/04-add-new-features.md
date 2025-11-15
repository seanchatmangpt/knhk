# How-to Guide: Add New Features

**Goal**: Implement new features following KNHK patterns
**Time**: 30-60 minutes per feature
**Difficulty**: Intermediate

## Overview

Adding features in KNHK means:
1. Plan the feature (specifications)
2. Design the solution (architecture)
3. Write tests first (TDD)
4. Implement code
5. Add telemetry
6. Validate with Weaver
7. Commit and push

## Before You Start

✅ Setup complete: [Setup Development Environment](01-setup-development-environment.md)
✅ Tests passing: `make test-chicago-v04`
✅ New feature branch created

## Step 1: Create Feature Branch

```bash
git checkout -b feature/my-feature-name
```

Use descriptive names:
- ✅ `feature/add-knowledge-hooks`
- ✅ `feature/improve-telemetry-validation`
- ❌ `feature/new-feature`
- ❌ `feature/fix-stuff`

## Step 2: Plan the Feature

**Write a specification** (in code comments or a doc):

```rust
// Feature: Process data with validation
// Input: String data
// Output: ProcessedData struct
// Behavior:
//   1. Validate input length (100-1000 chars)
//   2. Normalize whitespace
//   3. Return structured result
// Performance: Must complete in ≤8 ticks
// Telemetry: Emit span 'process_data' with attributes:
//   - input_length: input length
//   - validation_passed: boolean
```

## Step 3: Design the Architecture

Sketch the structure:

```rust
// In src/lib.rs or appropriate module

/// Processes and validates input data
#[derive(Debug, Clone)]
pub struct ProcessedData {
    pub original: String,
    pub normalized: String,
    pub valid: bool,
}

/// Error type for processing
#[derive(Debug)]
pub enum ProcessingError {
    TooShort,
    TooLong,
    InvalidCharacters,
}

// Main processing function
#[tracing::instrument]
pub fn process_data(input: &str) -> Result<ProcessedData, ProcessingError> {
    // TODO: Implementation
    unimplemented!("Feature not yet implemented")
}
```

## Step 4: Write Tests First (TDD)

In `tests/test_process_data.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_data() {
        let input = "hello world";
        let result = process_data(input);

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.original, "hello world");
        assert!(data.valid);
    }

    #[test]
    fn test_too_short() {
        let input = "hi";
        let result = process_data(input);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProcessingError::TooShort);
    }

    #[test]
    fn test_too_long() {
        let input = &"a".repeat(1001);
        let result = process_data(input);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProcessingError::TooLong);
    }

    #[test]
    fn test_normalization() {
        let input = "hello    world";  // Extra spaces
        let result = process_data(input);

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.normalized, "hello world");
    }
}
```

Run tests (they should all fail):
```bash
cargo test --lib process_data
```

Expected: All tests fail because function is not implemented.

## Step 5: Implement the Feature

In `src/lib.rs`:

```rust
#[tracing::instrument(skip(input))]
pub fn process_data(input: &str) -> Result<ProcessedData, ProcessingError> {
    // Validate length
    if input.len() < 100 {
        return Err(ProcessingError::TooShort);
    }
    if input.len() > 1000 {
        return Err(ProcessingError::TooLong);
    }

    // Normalize whitespace
    let normalized = input
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    // Validate characters
    if !normalized.chars().all(|c| c.is_alphanumeric() || c == ' ') {
        return Err(ProcessingError::InvalidCharacters);
    }

    // Return result
    Ok(ProcessedData {
        original: input.to_string(),
        normalized,
        valid: true,
    })
}
```

Run tests:
```bash
cargo test --lib process_data
```

Expected: All tests pass!

## Step 6: Add Telemetry

Document what your function does:

```rust
#[tracing::instrument(skip(input))]
pub fn process_data(input: &str) -> Result<ProcessedData, ProcessingError> {
    let input_length = input.len();

    info!(
        input_length = input_length,
        "Starting data processing"
    );

    // Validate length
    if input.len() < 100 {
        warn!("Input too short");
        return Err(ProcessingError::TooShort);
    }
    if input.len() > 1000 {
        warn!("Input too long");
        return Err(ProcessingError::TooLong);
    }

    // ... rest of implementation

    info!(
        validation_passed = true,
        "Processing completed successfully"
    );

    Ok(ProcessedData {
        original: input.to_string(),
        normalized,
        valid: true,
    })
}
```

## Step 7: Create Telemetry Schema

Create `registry/process_data.yaml`:

```yaml
instrumentation:
  name: process_data
  version: 1.0.0
  description: "Data processing with validation"

spans:
  - name: process_data
    description: "Process and validate input data"
    attributes:
      - input_length:
          description: "Length of input string"
          type: int
      - validation_passed:
          description: "Whether validation succeeded"
          type: bool
    events:
      - name: "Processing completed successfully"
        description: "Processing completed without errors"
      - name: "Input too short"
        description: "Input length below minimum"
      - name: "Input too long"
        description: "Input length exceeds maximum"
```

## Step 8: Validate Schema

```bash
weaver registry check -r registry/
```

Expected:
```
✓ Schema valid
✓ All components documented
```

If it fails, fix the schema or code. See [Fix Weaver Validation Errors](03-fix-weaver-validation-errors.md).

## Step 9: Run All Tests

```bash
cargo test --lib --release
```

All tests should pass.

## Step 10: Run Performance Tests

```bash
make test-performance-v04
```

Verify your feature meets ≤8 tick constraint.

If it fails, see [How to Optimize Performance](07-optimize-performance.md).

## Step 11: Integration Tests

If adding to existing system, test integration:

```bash
make test-integration-v2
```

## Step 12: Code Review Checklist

Before committing, verify:

- [ ] Code follows Rust conventions
  ```bash
  cargo fmt --all
  cargo clippy --workspace -- -D warnings
  ```

- [ ] Tests pass
  ```bash
  cargo test --lib --release
  ```

- [ ] Performance is acceptable
  ```bash
  make test-performance-v04
  ```

- [ ] Telemetry is documented
  ```bash
  weaver registry check -r registry/
  ```

- [ ] No `unwrap()` in production code (use `Result<T, E>`)

- [ ] No `println!` (use `tracing` macros)

- [ ] Documentation is complete

## Step 13: Commit Your Work

```bash
git add .
git commit -m "feat: add process_data feature with validation"
```

Commit message format:
- `feat:` for new features
- `fix:` for bug fixes
- `docs:` for documentation
- `test:` for tests

## Step 14: Push and Create PR

```bash
git push origin feature/my-feature-name
```

Create pull request on GitHub with:
- Description of what the feature does
- Link to any related issues
- Test results
- Performance metrics

## Complete Feature Checklist

```
Planning
  ✓ Feature specified
  ✓ Architecture designed

Development
  ✓ Tests written first (TDD)
  ✓ Code implemented
  ✓ Telemetry added
  ✓ Schema documented

Validation
  ✓ Unit tests pass (cargo test --lib)
  ✓ Chicago TDD tests pass (make test-chicago-v04)
  ✓ Performance tests pass (make test-performance-v04)
  ✓ Integration tests pass (make test-integration-v2)
  ✓ Weaver validation passes (weaver registry check)
  ✓ Code quality checks pass (clippy, fmt)

Submission
  ✓ Code committed with descriptive message
  ✓ Changes pushed to feature branch
  ✓ Pull request created with description
  ✓ CI/CD pipeline passes
```

## Example: Complete Feature

See this example feature in CLAUDE.md for a complete walkthrough of the SPARC methodology.

## Performance Tips

For features meeting ≤8 tick constraint:

- Avoid allocations in hot path
- Use `#[inline]` for small functions
- Prefer `&str` over `String`
- Use iterators instead of loops when possible
- Profile with: `cargo bench` (if benchmarks available)

## Common Mistakes

❌ **Not writing tests first**
- Tests help clarify requirements
- Refactoring becomes safer
- Fewer bugs reach production

❌ **Forgetting telemetry**
- Code becomes black box
- Hard to debug issues
- Violates KNHK principles

❌ **Skipping Weaver validation**
- Undocumented behavior slips in
- Creates false positives
- Defeats KNHK's purpose

❌ **Performance assumptions**
- Don't assume your code is fast
- Always run performance tests
- Profile bottlenecks

## Next Steps

- **Optimize performance**: [How to Optimize Performance](07-optimize-performance.md)
- **Debug issues**: [How to Debug Tests](05-debug-tests.md)
- **Create schemas**: [How to Create OTel Schemas](06-create-otel-schemas.md)

## Key Commands

```bash
# Create feature branch
git checkout -b feature/my-feature

# Run tests during development
cargo test --lib --release

# Validate before commit
cargo clippy --workspace -- -D warnings
cargo fmt --all
weaver registry check -r registry/

# Run all validations
make test-chicago-v04
make test-performance-v04

# Commit and push
git add .
git commit -m "feat: description"
git push origin feature/my-feature
```

---

**Category**: How-to Guides (Task-oriented)
**Framework**: Diátaxis
**Difficulty**: Intermediate
**Related**: Setup, Testing, Performance, Telemetry
