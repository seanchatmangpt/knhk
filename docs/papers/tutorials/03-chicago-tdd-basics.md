# Tutorial 3: Chicago TDD Basics

## Learning Objectives

By the end of this tutorial, you'll understand and practice:

- **What TDD is** and why it matters for KNHK
- **Chicago style TDD** (behavior-driven, outside-in)
- **The Red-Green-Refactor cycle** with real code
- **Writing effective tests** that validate behavior
- **Building features step-by-step** using TDD
- **Integrating telemetry** with test-driven development

**Time**: 20-25 minutes | **Level**: Beginner-Intermediate
**Prerequisites**: [Your First KNHK Workflow](01-getting-started.md), [Understanding Telemetry](02-understanding-telemetry.md)

---

## Part 1: What is Chicago TDD?

### TDD Philosophy

**Test-Driven Development (TDD)** is a discipline where you:

1. **Write tests first** (before code)
2. **Write implementation** (to pass tests)
3. **Refactor** (improve code)

```
Traditional Development:
Code → Test → (hope it works)

TDD Development:
Test → Code → Refactor
(certainty it works)
```

### Chicago Style (vs London Style)

There are two TDD styles:

| Aspect | Chicago | London |
|--------|---------|--------|
| **Focus** | Behavior | Interactions |
| **Testing** | Real objects | Mocked dependencies |
| **When to mock** | External (DB, API) | Everything |
| **Philosophy** | Test outcomes | Test contracts |
| **KNHK uses** | ✓ Chicago | (London for advanced) |

**Chicago TDD for KNHK:**
```
Test actual behavior → Mock only externals → Assert outcomes
```

### Why KNHK Uses Chicago TDD

```
KNHK needs to validate:
  ✓ Actual runtime behavior (not mocks)
  ✓ Telemetry emission (spans, metrics, logs)
  ✓ Performance constraints (≤8 ticks)

Chicago TDD provides:
  ✓ Real objects execute real code
  ✓ Actual telemetry is generated
  ✓ Real performance measurements
```

---

## Part 2: The Red-Green-Refactor Cycle

### Red: Write a Failing Test

**Red phase**: Write a test for behavior that doesn't exist yet.

```rust
#[test]
fn test_user_can_deactivate_account() {
    // Arrange
    let mut user = User {
        id: 1,
        email: "test@example.com".to_string(),
        status: "ACTIVE".to_string(),
        deactivated_at: None,
    };

    // Act
    let result = deactivate_account(&mut user);

    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(user.status, "INACTIVE");
    assert!(user.deactivated_at.is_some());
}
```

Run the test:
```bash
cargo test test_user_can_deactivate_account -- --nocapture
# test result: FAILED (as expected)
# error: cannot find function `deactivate_account` in this scope
```

✓ Test is RED (failing) as expected

### Green: Write Minimal Code to Pass

**Green phase**: Write the simplest code to pass the test.

```rust
fn deactivate_account(user: &mut User) -> Result<(), String> {
    user.status = "INACTIVE".to_string();
    user.deactivated_at = Some(Utc::now());
    Ok(())
}
```

Run the test:
```bash
cargo test test_user_can_deactivate_account
# test result: ok
```

✓ Test is GREEN (passing)

### Refactor: Improve Code Quality

**Refactor phase**: Improve code without changing behavior.

```rust
// Before refactor (works, but could be better)
fn deactivate_account(user: &mut User) -> Result<(), String> {
    user.status = "INACTIVE".to_string();
    user.deactivated_at = Some(Utc::now());
    Ok(())
}

// After refactor (cleaner, better error handling)
#[instrument(skip(user))]
fn deactivate_account(user: &mut User) -> Result<()> {
    if user.status == "INACTIVE" {
        return Err(UserError::AlreadyDeactivated);
    }

    user.status = "INACTIVE".to_string();
    user.deactivated_at = Some(Utc::now());

    info!("User deactivated successfully");
    Ok(())
}
```

Run tests again:
```bash
cargo test test_user_can_deactivate_account
# test result: ok
```

✓ Code improved, test still GREEN

### The Cycle Repeats

```
Write Test (RED) → Write Code (GREEN) → Refactor → Repeat
```

---

## Part 3: Hands-On: Implementing a Feature with TDD

### Feature: User Registration with Validation

Let's build a user registration system using Chicago TDD.

#### Step 1: Red - Test 1: Email Validation

```rust
#[test]
fn test_registration_rejects_invalid_email() {
    let result = register_user("invalid-email", "password123");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Invalid email format");
}
```

Run it:
```bash
cargo test test_registration_rejects_invalid_email
# error: cannot find function `register_user` in this scope
# RED ✗
```

#### Step 2: Green - Implement Email Validation

```rust
fn register_user(email: &str, password: &str) -> Result<User, String> {
    // Validate email
    if !email.contains('@') {
        return Err("Invalid email format".to_string());
    }

    // Create user (simplified for now)
    Ok(User {
        id: 1,
        email: email.to_string(),
        status: "ACTIVE".to_string(),
        deactivated_at: None,
    })
}
```

Run test:
```bash
cargo test test_registration_rejects_invalid_email
# test result: ok
# GREEN ✓
```

#### Step 3: Red - Test 2: Password Validation

```rust
#[test]
fn test_registration_requires_strong_password() {
    let result = register_user("user@example.com", "123");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Password too weak");
}
```

Run it:
```bash
cargo test test_registration_requires_strong_password
# FAILED - password not validated yet
# RED ✗
```

#### Step 4: Green - Add Password Validation

```rust
fn register_user(email: &str, password: &str) -> Result<User, String> {
    // Validate email
    if !email.contains('@') {
        return Err("Invalid email format".to_string());
    }

    // Validate password (NEW)
    if password.len() < 8 {
        return Err("Password too weak".to_string());
    }

    Ok(User {
        id: 1,
        email: email.to_string(),
        status: "ACTIVE".to_string(),
        deactivated_at: None,
    })
}
```

Run test:
```bash
cargo test test_registration_requires_strong_password
# test result: ok
# GREEN ✓
```

#### Step 5: Red - Test 3: Duplicate Email Prevention

```rust
#[test]
fn test_registration_prevents_duplicate_email() {
    // User already exists
    let _first = register_user("user@example.com", "password123");

    // Attempt to register same email
    let result = register_user("user@example.com", "password456");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Email already registered");
}
```

Run it:
```bash
cargo test test_registration_prevents_duplicate_email
# FAILED - no duplicate checking yet
# RED ✗
```

#### Step 6: Green - Add Duplicate Prevention

For this, we need a way to track users. Let's use a simple approach with a test-only database:

```rust
// Add to your code
use std::collections::HashSet;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref REGISTERED_EMAILS: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

fn register_user(email: &str, password: &str) -> Result<User, String> {
    // Validate email
    if !email.contains('@') {
        return Err("Invalid email format".to_string());
    }

    // Validate password
    if password.len() < 8 {
        return Err("Password too weak".to_string());
    }

    // Check for duplicates (NEW)
    let mut emails = REGISTERED_EMAILS.lock().unwrap();
    if emails.contains(email) {
        return Err("Email already registered".to_string());
    }

    emails.insert(email.to_string());

    Ok(User {
        id: 1,
        email: email.to_string(),
        status: "ACTIVE".to_string(),
        deactivated_at: None,
    })
}
```

Run test:
```bash
cargo test test_registration_prevents_duplicate_email
# test result: ok
# GREEN ✓
```

#### Step 7: Refactor - Add Telemetry

Now that all tests pass, let's add proper instrumentation:

```rust
#[instrument(skip(password))]  // Skip password in logs for security
fn register_user(email: &str, password: &str) -> Result<User, String> {
    counter!("user_registration_attempts", 1);

    // Validate email
    if !email.contains('@') {
        counter!("user_registration_invalid_email", 1);
        return Err("Invalid email format".to_string());
    }

    // Validate password
    if password.len() < 8 {
        counter!("user_registration_weak_password", 1);
        return Err("Password too weak".to_string());
    }

    // Check for duplicates
    let mut emails = REGISTERED_EMAILS.lock().unwrap();
    if emails.contains(email) {
        counter!("user_registration_duplicate_email", 1);
        return Err("Email already registered".to_string());
    }

    emails.insert(email.to_string());

    counter!("user_registration_success", 1);
    info!("User registered successfully");

    Ok(User {
        id: 1,
        email: email.to_string(),
        status: "ACTIVE".to_string(),
        deactivated_at: None,
    })
}
```

Run all tests:
```bash
cargo test test_registration
# test test_registration_rejects_invalid_email ... ok
# test test_registration_requires_strong_password ... ok
# test test_registration_prevents_duplicate_email ... ok
# test result: ok. 3 passed
```

✓ All tests GREEN, code is telemetry-instrumented

---

## Part 4: AAA Pattern (Arrange-Act-Assert)

Every good test follows the AAA pattern:

```rust
#[test]
fn test_user_deactivation() {
    // ARRANGE: Set up test data
    let mut user = User {
        id: 123,
        email: "john@example.com".to_string(),
        status: "ACTIVE".to_string(),
        deactivated_at: None,
    };

    // ACT: Perform the action being tested
    let result = deactivate_account(&mut user);

    // ASSERT: Verify the outcome
    assert_eq!(result, Ok(()));
    assert_eq!(user.status, "INACTIVE");
    assert!(user.deactivated_at.is_some());
}
```

### Why AAA?

```
Good Tests are Clear:
  Arrange (Setup)  - Reader knows initial state
  Act (Action)     - Reader knows what's being tested
  Assert (Result)  - Reader knows what should happen

Without AAA:
  user.status = "ACTIVE";
  user = transform(user);
  assert_eq!(user.status, "INACTIVE");
  // What are we testing? Unclear.
```

---

## Part 5: Chicago TDD Best Practices

### 1. Test Behavior, Not Implementation

**❌ WRONG: Testing implementation**
```rust
#[test]
fn test_user_has_deactivated_at_field() {
    let user = User::default();
    assert!(user.deactivated_at.is_none());  // Testing field, not behavior
}
```

**✅ CORRECT: Testing behavior**
```rust
#[test]
fn test_deactivated_user_cannot_login() {
    let mut user = active_user();
    deactivate_account(&mut user);

    let result = user.can_login();
    assert!(!result);  // Testing behavior, not field
}
```

### 2. Use Real Objects (Chicago Style)

**❌ WRONG: Over-mocking**
```rust
let mock_user = MockUser::new();
mock_user.expect_status().returning(|| "ACTIVE");
// Test doesn't verify real behavior
```

**✅ CORRECT: Real objects**
```rust
let mut user = User::new("john@example.com");
deactivate_account(&mut user);
assert_eq!(user.status, "INACTIVE");  // Real behavior verified
```

### 3. Only Mock External Dependencies

**External dependencies that should be mocked:**
- Databases
- APIs
- Network calls
- File system
- Time (sometimes)

**Internal logic should NOT be mocked:**
- Business logic
- Validation
- Data structures
- Your own functions

### 4. Test One Thing at a Time

**❌ WRONG: Testing multiple things**
```rust
#[test]
fn test_registration() {
    // Testing email validation AND password validation AND duplicate prevention
    let result = register_user("invalid@", "weak");
    assert!(result.is_err());
}
```

**✅ CORRECT: One thing per test**
```rust
#[test]
fn test_registration_requires_valid_email() {
    let result = register_user("invalid@", "strongpass123");
    assert!(result.is_err());
}

#[test]
fn test_registration_requires_strong_password() {
    let result = register_user("valid@example.com", "weak");
    assert!(result.is_err());
}
```

### 5. Use Descriptive Test Names

**❌ WRONG: Unclear names**
```rust
#[test]
fn test_user() { }

#[test]
fn test_func() { }
```

**✅ CORRECT: Descriptive names**
```rust
#[test]
fn test_inactive_user_cannot_access_dashboard() { }

#[test]
fn test_registration_rejects_email_without_at_symbol() { }
```

### 6. Arrange-Act-Assert Consistently

Every test should follow AAA:
1. **Arrange**: Set up test data
2. **Act**: Call the function being tested
3. **Assert**: Check the result

```rust
#[test]
fn test_something() {
    // Arrange: Create test data
    let input = prepare_test_input();

    // Act: Perform the action
    let result = function_under_test(input);

    // Assert: Check the outcome
    assert_eq!(result, expected_value);
}
```

---

## Part 6: Integration with Telemetry

### TDD + Telemetry = Powerful Combo

**Why add telemetry to TDD?**

```
TDD validates: behavior is correct
Telemetry validates: behavior produces proper instrumentation
Together: Complete validation
```

### Adding Telemetry in TDD

```rust
#[instrument]  // Add tracing
fn register_user(email: &str, password: &str) -> Result<User> {
    counter!("registration_attempt", 1);

    // ... validation ...

    counter!("registration_success", 1);
    Ok(user)
}

#[test]
fn test_registration_emits_telemetry() {
    // Arrange: Set up telemetry capture
    let telemetry = setup_telemetry_capture();

    // Act: Register a user
    let result = register_user("user@example.com", "password123");

    // Assert: Check both behavior AND telemetry
    assert!(result.is_ok());

    // Verify telemetry was emitted
    assert_eq!(telemetry.counter("registration_success"), 1);
    assert_eq!(telemetry.counter("registration_attempt"), 1);
    assert!(telemetry.has_span("register_user"));
}
```

---

## Part 7: Step-by-Step TDD Workflow

### For a New Feature

```
1. UNDERSTAND the requirement
   What should this feature do?
   What are the edge cases?

2. WRITE test(s) for main behavior
   Red: Test fails (feature doesn't exist)

3. IMPLEMENT main behavior
   Green: Test passes

4. WRITE test(s) for edge cases
   Red: Test fails (edge cases not handled)

5. HANDLE edge cases
   Green: Tests pass

6. ADD telemetry
   Verify behavior emits proper spans/metrics

7. REFACTOR for clarity
   Keep tests passing

8. VERIFY performance
   Ensure ≤8 ticks for hot paths

9. RUN full test suite
   Make sure nothing broke

10. COMMIT and push
    Feature is complete
```

---

## Part 8: Common TDD Mistakes

### Mistake 1: Writing Tests That Are Too Vague

**❌ WRONG:**
```rust
#[test]
fn test_something() {
    let x = do_something();
    assert!(x);
}
```

**✅ CORRECT:**
```rust
#[test]
fn test_inactive_user_cannot_post_comment() {
    let mut user = active_user();
    deactivate_account(&mut user);

    let result = user.can_post_comment();

    assert!(!result, "Inactive user should not post comments");
}
```

### Mistake 2: Skipping Refactoring

**After green, always refactor:**

```rust
// Before refactor
#[test]
fn test_deactivation() {
    let mut u = User { id: 1, email: "x@y.z".to_string(), status: "ACTIVE".to_string(), deactivated_at: None };
    deactivate_account(&mut u);
    assert_eq!(u.status, "INACTIVE");
}

// After refactor (same test, cleaner)
#[test]
fn test_deactivation() {
    let mut user = active_user_with_email("john@example.com");
    deactivate_account(&mut user);

    assert_eq!(user.status, "INACTIVE");
    assert!(user.deactivated_at.is_some());
}
```

### Mistake 3: Testing Implementation Details

**❌ WRONG:**
```rust
#[test]
fn test_hash_uses_sha256() {
    let hash = compute_hash("test");
    assert_eq!(hash.len(), 64);  // Testing implementation
}
```

**✅ CORRECT:**
```rust
#[test]
fn test_different_inputs_produce_different_hashes() {
    let hash1 = compute_hash("input1");
    let hash2 = compute_hash("input2");

    assert_ne!(hash1, hash2);  // Testing behavior
}
```

---

## Part 9: Running Tests with Chicago TDD

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_registration_rejects_invalid_email

# Run tests with output
cargo test -- --nocapture

# Run tests with detailed output
cargo test -- --nocapture --test-threads=1

# Run with KNHK Chicago TDD suite
make test-chicago-v04
```

---

## Summary: Chicago TDD Workflow

### The Process

1. **Red**: Write failing test for behavior
2. **Green**: Implement minimum code to pass
3. **Refactor**: Improve code quality
4. **Telemetry**: Add proper instrumentation
5. **Repeat**: Build feature incrementally

### Key Principles

- **Test behavior, not implementation**
- **Use real objects, mock only externals**
- **One assertion per test (ideally)**
- **Use AAA pattern consistently**
- **Descriptive test names matter**
- **Refactor after green phase**
- **Integrate telemetry validation**

### Benefits for KNHK

✓ Tests validate actual runtime behavior
✓ Telemetry is built in from the start
✓ Performance constraints are testable
✓ Code quality is high
✓ Refactoring is safe (tests catch errors)

---

**Created**: 2025-11-15
**Status**: Complete
**Difficulty**: Beginner-Intermediate
**Next**: [How-to: Add New Features](../how-to-guides/04-add-new-features.md)
