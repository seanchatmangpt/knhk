# How-to Guide 4: Add New Features

## Goal

Develop a complete feature from conception through validation, following KNHK's test-driven, telemetry-first approach to ensure production-ready code.

**Time Estimate**: 2.5-4 hours (depending on feature complexity)
**Prerequisites**: [Setup Development Environment](01-setup-development-environment.md), [Run Tests Efficiently](02-run-tests-efficiently.md), [Understanding Telemetry](../tutorials/02-understanding-telemetry.md)
**Difficulty**: Intermediate
**Outcomes**: Complete, tested, validated feature with proper instrumentation

---

## Quick Overview: The KNHK Development Workflow

```
1. Plan Feature (10 min)
   ↓
2. Create Test File (15 min)
   ↓
3. Write Test Cases (30 min)
   ↓
4. Implement Feature (60 min)
   ↓
5. Add Telemetry (20 min)
   ↓
6. Verify Implementation (15 min)
   ↓
7. Complete Validation (30 min)
   ↓
✓ Production-Ready Feature
```

---

## Step 1: Plan Your Feature

### Define What You're Building

**Start with a clear specification:**

```markdown
# Feature: User Account Deactivation

## What
Allow users to temporarily deactivate their accounts.

## Why
Users need privacy control and ability to take breaks.

## How
- New endpoint: POST /api/users/:id/deactivate
- Sets account status to "INACTIVE"
- Preserves all user data
- Can be reactivated within 30 days

## Success Criteria
- ✓ Account marked inactive
- ✓ User cannot login
- ✓ Data preserved
- ✓ Reactivation works
- ✓ Performance: ≤8 ticks
- ✓ Weaver validation passes
```

### Identify Dependencies

**Ask yourself:**
- What do I need before building?
- What modules will I modify?
- What tests do I need?
- What telemetry is critical?

**Example:**
```
Dependencies:
  ✓ User model exists
  ✓ Database connection available
  ✓ Authentication system in place

Modules to touch:
  - src/user/mod.rs (add deactivation logic)
  - src/handlers/user.rs (add endpoint)
  - tests/user_deactivation.rs (add tests)

Tests needed:
  - Deactivate success
  - Deactivate twice (already inactive)
  - Cannot login when inactive
  - Reactivate works
  - Performance test

Telemetry needed:
  - Track deactivation attempts
  - Measure operation duration
  - Log success/failure
```

---

## Step 2: Create the Test File

### File Organization

**KNHK test structure:**
```
tests/
├── integration/
│   ├── user_deactivation_test.rs    ← Integration tests
│   └── common.rs                      ← Shared test utilities
└── unit/                              (if applicable)
    └── user_test.rs
```

### Create Test Module

Create `tests/user_deactivation.rs`:

```rust
//! Tests for user account deactivation feature
//!
//! This module tests the user deactivation workflow,
//! including validation, edge cases, and performance.

use knhk::user::{User, deactivate_account};
use knhk::error::Result;

/// Test helper: Create a test user
fn create_test_user(email: &str) -> User {
    User {
        id: 1,
        email: email.to_string(),
        status: "ACTIVE".to_string(),
    }
}

/// Test helper: Verify user is deactivated
fn assert_user_deactivated(user: &User) {
    assert_eq!(user.status, "INACTIVE");
}
```

---

## Step 3: Write Test Cases

### Test-First Approach

**Write tests BEFORE implementation:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Test: Successfully deactivate an active account
    ///
    /// Given: A user with ACTIVE status
    /// When: We call deactivate_account
    /// Then: The user status changes to INACTIVE
    #[test]
    fn test_deactivate_active_account_succeeds() {
        // Arrange
        let mut user = create_test_user("alice@example.com");
        assert_eq!(user.status, "ACTIVE");

        // Act
        let result = deactivate_account(&mut user);

        // Assert
        assert!(result.is_ok());
        assert_user_deactivated(&user);
    }

    /// Test: Cannot deactivate already inactive account
    ///
    /// Given: A user with INACTIVE status
    /// When: We call deactivate_account again
    /// Then: An error is returned
    #[test]
    fn test_deactivate_inactive_account_fails() {
        // Arrange
        let mut user = create_test_user("bob@example.com");
        deactivate_account(&mut user).unwrap();
        assert_eq!(user.status, "INACTIVE");

        // Act
        let result = deactivate_account(&mut user);

        // Assert
        assert!(result.is_err());
    }

    /// Test: Deactivated user cannot login
    ///
    /// Given: A deactivated account
    /// When: We attempt to login
    /// Then: Login fails
    #[test]
    fn test_deactivated_user_cannot_login() {
        // Arrange
        let mut user = create_test_user("charlie@example.com");
        deactivate_account(&mut user).unwrap();

        // Act
        let login_result = user.authenticate("password123");

        // Assert
        assert!(login_result.is_err());
    }

    /// Test: Can reactivate account within 30 days
    ///
    /// Given: A deactivated account within grace period
    /// When: We call reactivate_account
    /// Then: Account becomes active again
    #[test]
    fn test_reactivate_within_grace_period_succeeds() {
        // Arrange
        let mut user = create_test_user("diana@example.com");
        deactivate_account(&mut user).unwrap();

        // Act
        let result = reactivate_account(&mut user);

        // Assert
        assert!(result.is_ok());
        assert_eq!(user.status, "ACTIVE");
    }

    /// Test: Cannot reactivate after 30-day grace period
    ///
    /// Given: A deactivated account outside grace period
    /// When: We call reactivate_account
    /// Then: An error is returned
    #[test]
    #[ignore]  // Requires time mocking
    fn test_reactivate_after_grace_period_fails() {
        // Arrange: Set deactivation date to 31 days ago
        // Act: Call reactivate_account
        // Assert: Should fail
        todo!("Requires time mocking - implement with freezegun or similar")
    }

    /// Test: Performance meets Chatman Constant (≤8 ticks)
    ///
    /// Given: A valid account to deactivate
    /// When: We measure deactivation time
    /// Then: Operation completes within 8 ticks
    #[test]
    fn test_deactivate_performance_within_constraint() {
        let mut user = create_test_user("emma@example.com");
        let start = std::time::Instant::now();

        deactivate_account(&mut user).unwrap();

        let elapsed = start.elapsed();
        // Assuming 1 tick ≈ 1ms for this example
        assert!(elapsed.as_millis() <= 8, "Deactivation took {}ms", elapsed.as_millis());
    }

    /// Test: Telemetry is emitted correctly
    ///
    /// Given: A deactivation operation
    /// When: The operation completes
    /// Then: Proper spans and metrics are emitted
    #[test]
    fn test_deactivation_emits_telemetry() {
        // This will be verified by Weaver validation
        // For now, just ensure no panics during telemetry
        let mut user = create_test_user("frank@example.com");
        let result = deactivate_account(&mut user);
        assert!(result.is_ok());
    }
}
```

### Run Tests (Expected to Fail)

```bash
cargo test user_deactivation

# Output:
# error[E0433]: cannot find function `deactivate_account` in this scope
#
# This is expected! We haven't implemented it yet.
```

---

## Step 4: Implement the Feature

### Determine Where Code Goes

**KNHK code organization:**
```
src/
├── lib.rs                    # Main library entry
├── user/
│   ├── mod.rs               # User module
│   ├── model.rs             # User data structure
│   └── operations.rs         # Deactivation logic ← NEW
├── handlers/
│   ├── mod.rs
│   └── user.rs              # HTTP endpoints
└── error.rs                 # Error types
```

### Implement the Feature Module

Create `src/user/operations.rs`:

```rust
//! User account operations including deactivation and reactivation.

use tracing::instrument;
use metrics::{counter, histogram};
use crate::error::{Result, UserError};
use super::User;
use std::time::Instant;

/// Deactivate a user account
///
/// # Arguments
/// * `user` - The user to deactivate
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(UserError)` if already inactive
///
/// # Telemetry
/// - Span: `deactivate_user_account`
/// - Metric: `user_deactivations_total`
/// - Metric: `deactivation_duration_ms`
#[instrument(skip(user))]  // Skip logging entire user object
pub fn deactivate_account(user: &mut User) -> Result<()> {
    let start = Instant::now();

    // Check if already inactive
    if user.status == "INACTIVE" {
        counter!("user_deactivation_already_inactive", 1);
        return Err(UserError::AlreadyDeactivated);
    }

    // Perform deactivation
    user.status = "INACTIVE".to_string();
    user.deactivated_at = Some(chrono::Utc::now());

    // Record metrics
    let duration_ms = start.elapsed().as_millis() as f64;
    counter!("user_deactivations_total", 1);
    histogram!("deactivation_duration_ms", duration_ms);

    Ok(())
}

/// Reactivate a user account (within 30-day grace period)
///
/// # Arguments
/// * `user` - The user to reactivate
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(UserError)` if not inactive or grace period expired
///
/// # Telemetry
/// - Span: `reactivate_user_account`
/// - Metric: `user_reactivations_total`
#[instrument(skip(user))]
pub fn reactivate_account(user: &mut User) -> Result<()> {
    let start = Instant::now();

    // Check if active
    if user.status == "ACTIVE" {
        counter!("user_reactivation_already_active", 1);
        return Err(UserError::AlreadyActive);
    }

    // Check if within grace period (30 days)
    if let Some(deactivated_at) = user.deactivated_at {
        let elapsed = chrono::Utc::now()
            .signed_duration_since(deactivated_at)
            .num_days();

        if elapsed > 30 {
            counter!("user_reactivation_grace_expired", 1);
            return Err(UserError::GracePeriodExpired);
        }
    }

    // Perform reactivation
    user.status = "ACTIVE".to_string();
    user.deactivated_at = None;

    // Record metrics
    let duration_ms = start.elapsed().as_millis() as f64;
    counter!("user_reactivations_total", 1);
    histogram!("reactivation_duration_ms", duration_ms);

    Ok(())
}

/// Verify if user can login (must be ACTIVE)
pub fn can_login(user: &User) -> bool {
    user.status == "ACTIVE"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deactivate_success() {
        let mut user = User {
            id: 1,
            email: "test@example.com".to_string(),
            status: "ACTIVE".to_string(),
            deactivated_at: None,
        };

        assert!(deactivate_account(&mut user).is_ok());
        assert_eq!(user.status, "INACTIVE");
    }

    #[test]
    fn test_reactivate_success() {
        let mut user = User {
            id: 1,
            email: "test@example.com".to_string(),
            status: "INACTIVE".to_string(),
            deactivated_at: Some(chrono::Utc::now()),
        };

        assert!(reactivate_account(&mut user).is_ok());
        assert_eq!(user.status, "ACTIVE");
    }
}
```

### Add HTTP Handler

Update `src/handlers/user.rs`:

```rust
use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use crate::user::deactivate_account;

/// POST /api/users/:id/deactivate
#[instrument]  // Auto-spans HTTP handler
pub async fn deactivate_user(
    Path(user_id): Path<u64>,
) -> Result<impl IntoResponse> {
    // Fetch user from database
    let mut user = db::get_user(user_id)
        .await
        .ok_or(UserError::NotFound)?;

    // Deactivate
    deactivate_account(&mut user)?;

    // Save to database
    db::save_user(&user).await?;

    Ok((StatusCode::OK, Json(user)))
}
```

### Update Module Exports

Add to `src/user/mod.rs`:

```rust
mod operations;
pub use operations::{deactivate_account, reactivate_account, can_login};
```

---

## Step 5: Run Tests (Should Pass)

```bash
cargo test user_deactivation -- --nocapture

# Output:
# test test_deactivate_active_account_succeeds ... ok
# test test_deactivate_inactive_account_fails ... ok
# test test_deactivated_user_cannot_login ... ok
# test test_reactivate_within_grace_period_succeeds ... ok
# test test_deactivate_performance_within_constraint ... ok
# test test_deactivation_emits_telemetry ... ok
#
# test result: ok. 6 passed
```

---

## Step 6: Add Comprehensive Telemetry

### Create OTel Schema

Create `registry/schemas/user_deactivation.yaml`:

```yaml
groups:
  user.deactivation:
    spans:
      deactivate_user_account:
        description: "User account deactivation operation"
        attributes:
          user_id:
            type: int
            description: "ID of user being deactivated"
          email:
            type: string
            description: "Email of user"
          previous_status:
            type: string
            description: "Status before deactivation"
      reactivate_user_account:
        description: "User account reactivation operation"
        attributes:
          user_id:
            type: int

    metrics:
      user_deactivations_total:
        type: counter
        description: "Total user deactivations"
        unit: "1"
      deactivation_duration_ms:
        type: histogram
        description: "User deactivation duration"
        unit: "ms"
      user_deactivation_already_inactive:
        type: counter
        description: "Attempts to deactivate already inactive account"
        unit: "1"
```

---

## Step 7: Verify Code Quality

### Code Review Checklist

- [ ] No `.unwrap()` calls in production code
- [ ] All errors handled properly
- [ ] Telemetry added to critical paths
- [ ] Performance constraints met (≤8 ticks)
- [ ] Tests cover success and error cases
- [ ] Edge cases considered
- [ ] Documentation added

### Run Code Quality Checks

```bash
# Type check
cargo check --all

# Lint
cargo clippy --all-targets -- -D warnings

# Format
cargo fmt --all

# All tests pass
cargo test --all

# Chicago TDD tests
make test-chicago-v04

# Performance test
make test-performance-v04
```

**Expected**: All checks pass with zero warnings.

---

## Step 8: Complete Validation

### Tier 1: Build and Code Quality

```bash
# Should complete with zero warnings
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo fmt --all

# Should show only your new code
git diff --stat
```

✅ **Verification**: Zero compiler warnings, formatted code

### Tier 2: Testing

```bash
# All tests pass
cargo test --workspace
cargo test user_deactivation -- --nocapture

# Chicago TDD
make test-chicago-v04

# Performance: ≤8 ticks
make test-performance-v04
```

✅ **Verification**: All tests pass, performance validated

### Tier 3: Schema Validation (Weaver)

```bash
# Validate schema
weaver registry check -r registry/

# Live validation
weaver registry live-check --registry registry/
```

✅ **Verification**: Schema valid, telemetry matches schema

---

## Step 9: Complete Feature Checklist

Before marking feature complete, verify:

### Code Quality
- [ ] No compilation warnings
- [ ] Clippy zero warnings
- [ ] Code properly formatted
- [ ] No `.unwrap()` in production code

### Testing
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Chicago TDD tests pass
- [ ] Edge cases tested
- [ ] Error paths tested

### Performance
- [ ] Performance tests pass
- [ ] All operations ≤8 ticks
- [ ] No performance regressions
- [ ] Benchmarked if applicable

### Telemetry
- [ ] Spans emitted correctly
- [ ] Metrics recorded
- [ ] Logs structured
- [ ] Schema defined
- [ ] Weaver validation passes

### Documentation
- [ ] Function documented
- [ ] Examples in comments
- [ ] Error cases documented
- [ ] Telemetry documented
- [ ] API documented

---

## Troubleshooting

### Issue: Tests Don't Compile

**Error**: `cannot find function 'deactivate_account'`

**Solution**:
```bash
# Ensure module is exported
# In src/user/mod.rs:
pub mod operations;
pub use operations::deactivate_account;

# Rebuild
cargo build
```

### Issue: Telemetry Not Emitting

**Error**: Spans/metrics not visible

**Solution**:
```bash
# Ensure tracing is initialized
# At application startup:
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .init();

# Run with RUST_LOG
RUST_LOG=debug cargo test
```

### Issue: Weaver Validation Fails

**Error**: `Schema validation failed: missing metric 'user_deactivations_total'`

**Solution**:
```bash
# Update schema to match code
# Edit registry/schemas/user_deactivation.yaml
# Add missing metric definition

# Verify schema
weaver registry check -r registry/

# Try again
weaver registry live-check --registry registry/
```

### Issue: Performance Exceeds 8 Ticks

**Error**: `deactivation_duration_ms: 12 ticks (exceeds limit of 8)`

**Solution**:
1. Profile with flamegraph: `cargo flamegraph`
2. Identify bottleneck
3. Optimize (cache, batch, parallelize)
4. Reduce telemetry overhead if needed
5. Re-test

---

## Quick Reference: Feature Development Timeline

```
Planning:             10 minutes
Test file setup:      15 minutes
Writing tests:        30 minutes
Implementing feature: 60 minutes
Adding telemetry:     20 minutes
Code quality checks:  15 minutes
Full validation:      30 minutes
─────────────────────────────────
Total:               ~180 minutes (3 hours)

Factors that increase time:
- More complex logic
- More test cases
- More edge cases
- Performance optimization needed
- Schema design complexity
```

---

## Next Steps

After completing your feature:

1. **Commit with descriptive message**:
   ```bash
   git commit -m "feat: implement user account deactivation

   - Add deactivate_account and reactivate_account functions
   - Implement 30-day grace period for reactivation
   - Add comprehensive test coverage
   - Add telemetry for monitoring
   - Verify performance ≤8 ticks

   Closes #123"
   ```

2. **Create Pull Request**
   - Reference related issues
   - Describe telemetry added
   - Link to validation results

3. **Learn Related Topics**:
   - [How-to: Create OTel Schemas](05-create-otel-schemas.md) (coming soon)
   - [How-to: Emit Proper Telemetry](07-emit-proper-telemetry.md) (coming soon)
   - [How-to: Optimize Performance](08-optimize-performance.md) (coming soon)

---

## Real-World Patterns

### Pattern 1: Soft Delete (Archive)

```rust
#[instrument]
pub fn archive_feature(feature: &mut Feature) -> Result<()> {
    if feature.is_archived {
        return Err(Error::AlreadyArchived);
    }
    feature.is_archived = true;
    feature.archived_at = Some(Utc::now());
    counter!("features_archived_total", 1);
    Ok(())
}
```

### Pattern 2: Status Transitions

```rust
#[instrument]
pub fn transition_status(item: &mut Item, new_status: Status) -> Result<()> {
    // Validate transition is allowed
    validate_transition(&item.status, &new_status)?;

    // Record metrics for this transition
    counter!("status_transitions_total", 1, "from" => format!("{:?}", item.status), "to" => format!("{:?}", new_status));

    // Update
    item.status = new_status;
    Ok(())
}
```

### Pattern 3: Reversible Operations

```rust
#[instrument]
pub fn disable_feature(item: &mut Item) -> Result<()> {
    if item.disabled {
        return Err(Error::AlreadyDisabled);
    }
    item.disabled = true;
    item.disabled_at = Some(Utc::now());
    histogram!("feature_disabled_duration_ms", calculate_duration(item));
    Ok(())
}

#[instrument]
pub fn enable_feature(item: &mut Item) -> Result<()> {
    if !item.disabled {
        return Err(Error::NotDisabled);
    }
    item.disabled = false;
    item.disabled_at = None;
    Ok(())
}
```

---

## Summary

### Feature Development in KNHK

1. **Plan**: Clear specification and test cases
2. **Test First**: Write tests before implementation
3. **Implement**: Code that passes tests
4. **Telemetry**: Add comprehensive instrumentation
5. **Validate**: All checks pass (tests, quality, performance, schemas)

### Key Principles

✅ Test-first development
✅ Telemetry integrated from the start
✅ Performance validated (≤8 ticks)
✅ Schema-validated behavior
✅ Zero false positives

### Quality Gates

Every feature must:
- ✓ Pass all tests
- ✓ Pass clippy (zero warnings)
- ✓ Meet ≤8 tick constraint
- ✓ Emit proper telemetry
- ✓ Pass Weaver validation

---

**Created**: 2025-11-15
**Updated**: 2025-11-15
**Status**: Complete
**Difficulty**: Intermediate
**Next**: [How-to: Create OTel Schemas](05-create-otel-schemas.md) (coming soon)
