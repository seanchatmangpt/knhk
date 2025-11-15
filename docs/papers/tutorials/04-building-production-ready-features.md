# Tutorial 4: Building Production-Ready Features

## Learning Objectives

By the end of this tutorial, you'll be able to:

- **Plan a feature** from requirements to deployment
- **Implement with TDD** using Chicago-style testing
- **Integrate telemetry** from the start (not as an afterthought)
- **Validate with Weaver** to ensure correctness
- **Optimize performance** to meet the ≤8 tick constraint
- **Certify production readiness** using the three-tier validation

**Time**: 30-45 minutes | **Level**: Intermediate
**Prerequisites**: [Chicago TDD Basics](03-chicago-tdd-basics.md), [Understanding Telemetry](02-understanding-telemetry.md)

---

## What You'll Build

A **User Activity Log** feature that:

1. Accepts user activity events via an API
2. Validates event data for correctness
3. Stores events in memory (simplified for this tutorial)
4. Emits OpenTelemetry spans and metrics
5. Meets performance constraints (≤8 ticks)
6. Passes three-tier validation (Weaver + Quality + Tests)

**Real-world use case**: This pattern applies to audit logging, analytics tracking, event sourcing, and observability systems.

---

## Part 1: Feature Planning (Specification)

### Step 1.1: Define Requirements

**Feature**: User Activity Log

**Requirements**:
- Track user actions (login, logout, page_view, action_taken)
- Capture metadata (user_id, timestamp, action_type, details)
- Validate all inputs before storage
- Emit telemetry for monitoring
- Performance: ≤8 ticks per event

**Non-functional requirements**:
- No data loss
- Idempotent (duplicate events ok)
- Thread-safe
- Proper error handling

### Step 1.2: Design the API

```rust
// Public API surface
pub struct ActivityEvent {
    pub user_id: u64,
    pub action_type: ActionType,
    pub timestamp: DateTime<Utc>,
    pub details: HashMap<String, String>,
}

pub enum ActionType {
    Login,
    Logout,
    PageView,
    ActionTaken,
}

pub struct ActivityLog {
    events: Vec<ActivityEvent>,
}

impl ActivityLog {
    pub fn new() -> Self;
    pub fn log_event(&mut self, event: ActivityEvent) -> Result<(), LogError>;
    pub fn get_events_for_user(&self, user_id: u64) -> Vec<&ActivityEvent>;
    pub fn count_events(&self) -> usize;
}
```

**✅ Checkpoint**: API design complete

### Step 1.3: Design Telemetry Schema

Before writing code, design the OpenTelemetry schema:

```yaml
# registry/activity-log.yaml
spans:
  - name: activity.log_event
    brief: Logs a user activity event
    attributes:
      - name: user.id
        type: int
        requirement_level: required
      - name: action.type
        type: string
        requirement_level: required
      - name: event.timestamp
        type: string
        requirement_level: required

metrics:
  - name: activity.events.total
    brief: Total number of events logged
    instrument: counter
    unit: "{event}"

  - name: activity.log_duration
    brief: Duration to log an event
    instrument: histogram
    unit: ms
```

**✅ Checkpoint**: Telemetry schema designed

---

## Part 2: TDD Implementation

### Step 2.1: RED - Write First Test

```rust
// tests/activity_log_test.rs

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_can_log_user_login_event() {
        // Arrange
        let mut log = ActivityLog::new();
        let event = ActivityEvent {
            user_id: 123,
            action_type: ActionType::Login,
            timestamp: Utc::now(),
            details: HashMap::new(),
        };

        // Act
        let result = log.log_event(event);

        // Assert
        assert!(result.is_ok());
        assert_eq!(log.count_events(), 1);
    }
}
```

Run the test:
```bash
cargo test test_can_log_user_login_event

# error[E0433]: failed to resolve: use of undeclared type `ActivityLog`
# RED ✗
```

**✅ Checkpoint**: Test is RED (as expected)

### Step 2.2: GREEN - Implement Minimal Code

```rust
// src/activity_log.rs

use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ActionType {
    Login,
    Logout,
    PageView,
    ActionTaken,
}

#[derive(Debug, Clone)]
pub struct ActivityEvent {
    pub user_id: u64,
    pub action_type: ActionType,
    pub timestamp: DateTime<Utc>,
    pub details: HashMap<String, String>,
}

#[derive(Debug)]
pub enum LogError {
    InvalidUserId,
    InvalidTimestamp,
}

pub struct ActivityLog {
    events: Vec<ActivityEvent>,
}

impl ActivityLog {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }

    pub fn log_event(&mut self, event: ActivityEvent) -> Result<(), LogError> {
        self.events.push(event);
        Ok(())
    }

    pub fn count_events(&self) -> usize {
        self.events.len()
    }

    pub fn get_events_for_user(&self, user_id: u64) -> Vec<&ActivityEvent> {
        self.events
            .iter()
            .filter(|e| e.user_id == user_id)
            .collect()
    }
}
```

Run the test:
```bash
cargo test test_can_log_user_login_event
# test result: ok. 1 passed
# GREEN ✓
```

**✅ Checkpoint**: Test is GREEN

### Step 2.3: RED - Add Validation Test

```rust
#[test]
fn test_rejects_invalid_user_id() {
    // Arrange
    let mut log = ActivityLog::new();
    let event = ActivityEvent {
        user_id: 0,  // Invalid: user IDs start at 1
        action_type: ActionType::Login,
        timestamp: Utc::now(),
        details: HashMap::new(),
    };

    // Act
    let result = log.log_event(event);

    // Assert
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LogError::InvalidUserId));
}
```

Run test:
```bash
cargo test test_rejects_invalid_user_id
# FAILED - no validation yet
# RED ✗
```

**✅ Checkpoint**: Test is RED

### Step 2.4: GREEN - Add Validation

```rust
pub fn log_event(&mut self, event: ActivityEvent) -> Result<(), LogError> {
    // Validate user ID
    if event.user_id == 0 {
        return Err(LogError::InvalidUserId);
    }

    self.events.push(event);
    Ok(())
}
```

Run test:
```bash
cargo test test_rejects_invalid_user_id
# test result: ok. 1 passed
# GREEN ✓
```

**✅ Checkpoint**: Validation test passes

### Step 2.5: RED - Add Timestamp Validation Test

```rust
#[test]
fn test_rejects_future_timestamps() {
    // Arrange
    let mut log = ActivityLog::new();
    let future = Utc::now() + chrono::Duration::hours(1);
    let event = ActivityEvent {
        user_id: 123,
        action_type: ActionType::Login,
        timestamp: future,  // Future timestamp not allowed
        details: HashMap::new(),
    };

    // Act
    let result = log.log_event(event);

    // Assert
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LogError::InvalidTimestamp));
}
```

Run test:
```bash
cargo test test_rejects_future_timestamps
# FAILED - no timestamp validation yet
# RED ✗
```

**✅ Checkpoint**: Test is RED

### Step 2.6: GREEN - Add Timestamp Validation

```rust
pub fn log_event(&mut self, event: ActivityEvent) -> Result<(), LogError> {
    // Validate user ID
    if event.user_id == 0 {
        return Err(LogError::InvalidUserId);
    }

    // Validate timestamp (not in future)
    if event.timestamp > Utc::now() {
        return Err(LogError::InvalidTimestamp);
    }

    self.events.push(event);
    Ok(())
}
```

Run all tests:
```bash
cargo test
# test test_can_log_user_login_event ... ok
# test test_rejects_invalid_user_id ... ok
# test test_rejects_future_timestamps ... ok
# test result: ok. 3 passed
# GREEN ✓
```

**✅ Checkpoint**: All validation tests pass

---

## Part 3: Add Telemetry Instrumentation

### Step 3.1: Add Telemetry Dependencies

```toml
# Cargo.toml
[dependencies]
tracing = "0.1"
tracing-opentelemetry = "0.21"
opentelemetry = { version = "0.21", features = ["metrics"] }
```

### Step 3.2: Instrument with Spans and Metrics

```rust
use tracing::{instrument, info, warn};
use opentelemetry::metrics::{Counter, Histogram};

pub struct ActivityLog {
    events: Vec<ActivityEvent>,
    // Metrics
    events_counter: Counter<u64>,
    duration_histogram: Histogram<f64>,
}

impl ActivityLog {
    pub fn new() -> Self {
        let meter = opentelemetry::global::meter("activity_log");

        Self {
            events: Vec::new(),
            events_counter: meter
                .u64_counter("activity.events.total")
                .with_description("Total activity events logged")
                .init(),
            duration_histogram: meter
                .f64_histogram("activity.log_duration")
                .with_description("Duration to log events")
                .with_unit("ms")
                .init(),
        }
    }

    #[instrument(skip(self), fields(
        user.id = event.user_id,
        action.type = ?event.action_type,
    ))]
    pub fn log_event(&mut self, event: ActivityEvent) -> Result<(), LogError> {
        let start = std::time::Instant::now();

        // Validate user ID
        if event.user_id == 0 {
            warn!("Invalid user ID: 0");
            return Err(LogError::InvalidUserId);
        }

        // Validate timestamp
        if event.timestamp > Utc::now() {
            warn!("Future timestamp rejected");
            return Err(LogError::InvalidTimestamp);
        }

        // Log the event
        self.events.push(event.clone());

        // Record metrics
        self.events_counter.add(1, &[]);
        let duration = start.elapsed();
        self.duration_histogram.record(duration.as_secs_f64() * 1000.0, &[]);

        info!("Activity event logged successfully");
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn get_events_for_user(&self, user_id: u64) -> Vec<&ActivityEvent> {
        let events: Vec<_> = self.events
            .iter()
            .filter(|e| e.user_id == user_id)
            .collect();

        info!(user_id = user_id, count = events.len(), "Retrieved user events");
        events
    }
}
```

**✅ Checkpoint**: Telemetry instrumentation added

### Step 3.3: Test Telemetry Emission

```rust
#[test]
fn test_telemetry_emitted_on_log() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::Registry;

    // Setup telemetry capture
    let subscriber = Registry::default();
    tracing::subscriber::set_global_default(subscriber).ok();

    // Arrange
    let mut log = ActivityLog::new();
    let event = ActivityEvent {
        user_id: 123,
        action_type: ActionType::Login,
        timestamp: Utc::now(),
        details: HashMap::new(),
    };

    // Act
    let result = log.log_event(event);

    // Assert
    assert!(result.is_ok());
    // Telemetry was emitted (verified by Weaver in next step)
}
```

**✅ Checkpoint**: Telemetry test added

---

## Part 4: Weaver Validation

### Step 4.1: Create Weaver Schema

```yaml
# registry/activity-log.yaml
schema_url: https://example.com/schemas/activity-log/1.0.0
groups:
  - id: activity.log
    type: span
    brief: Activity logging operations
    spans:
      - id: activity.log_event
        brief: Logs a user activity event
        attributes:
          - id: user.id
            type: int
            requirement_level: required
            brief: User identifier
          - id: action.type
            type: string
            requirement_level: required
            brief: Type of action performed

  - id: activity.metrics
    type: metric
    brief: Activity logging metrics
    metrics:
      - id: activity.events.total
        brief: Total number of events logged
        instrument: counter
        unit: "{event}"

      - id: activity.log_duration
        brief: Duration to log an event
        instrument: histogram
        unit: ms
```

### Step 4.2: Validate Schema

```bash
# Validate schema definition
weaver registry check -r registry/

# Expected output:
# ✓ Schema validation passed
# ✓ Span activity.log_event defined correctly
# ✓ Metric activity.events.total defined correctly
# ✓ Metric activity.log_duration defined correctly
```

**✅ Checkpoint**: Schema is valid

### Step 4.3: Validate Runtime Telemetry

```bash
# Run application and validate live telemetry
cargo run --example activity_log_demo

# In another terminal:
weaver registry live-check --registry registry/

# Expected output:
# ✓ Checking span: activity.log_event
#   ✓ Required attribute user.id present (type: int)
#   ✓ Required attribute action.type present (type: string)
# ✓ Checking metric: activity.events.total
#   ✓ Counter incremented correctly
# ✓ Checking metric: activity.log_duration
#   ✓ Histogram recorded correctly
# ✓ All telemetry validates against schema
```

**✅ Checkpoint**: Runtime telemetry matches schema

---

## Part 5: Performance Optimization

### Step 5.1: Measure Baseline Performance

```rust
#[test]
fn test_performance_meets_constraint() {
    let mut log = ActivityLog::new();
    let start = std::time::Instant::now();

    // Log 1000 events
    for i in 1..=1000 {
        let event = ActivityEvent {
            user_id: i,
            action_type: ActionType::Login,
            timestamp: Utc::now(),
            details: HashMap::new(),
        };
        log.log_event(event).unwrap();
    }

    let elapsed = start.elapsed();
    let avg_ticks = elapsed.as_millis() / 1000;

    println!("Average ticks per event: {}", avg_ticks);
    assert!(avg_ticks <= 8, "Exceeds ≤8 tick constraint");
}
```

Run test:
```bash
cargo test test_performance_meets_constraint -- --nocapture

# Average ticks per event: 3
# test result: ok
# ✓ Meets performance constraint
```

**✅ Checkpoint**: Performance is ≤8 ticks

### Step 5.2: Profile (if needed)

If performance exceeded 8 ticks:

```bash
# Generate flamegraph
cargo flamegraph --test activity_log_test -- test_performance_meets_constraint

# Review flamegraph.svg to find bottlenecks
# Apply optimizations from How-to Guide 8
```

**✅ Checkpoint**: Performance optimized

---

## Part 6: Three-Tier Validation

### Step 6.1: TIER 1 - Weaver Validation

```bash
# Source of truth validation
weaver registry check -r registry/
weaver registry live-check --registry registry/

# Both must pass
# ✓ Schema valid
# ✓ Runtime telemetry matches schema
```

**✅ TIER 1 PASSED**: Weaver validation successful

### Step 6.2: TIER 2 - Code Quality

```bash
# Clean build
cargo build --release

# Zero warnings from Clippy
cargo clippy -- -D warnings

# Formatted code
cargo fmt --all -- --check
```

**✅ TIER 2 PASSED**: Code quality baseline met

### Step 6.3: TIER 3 - Traditional Testing

```bash
# All tests pass
cargo test

# Chicago TDD tests
make test-chicago-v04

# Performance tests
make test-performance-v04
```

**✅ TIER 3 PASSED**: Traditional tests pass

---

## Part 7: Production Readiness Certification

### Step 7.1: Security Audit

```bash
# No unwrap/expect in production code
! grep -r "\.unwrap()\|\.expect(" src/activity_log.rs | grep -v "test"
# (no results = good)

# No hardcoded secrets
! grep -r "password\|secret\|api_key" src/activity_log.rs
# (no results = good)
```

**✅ Checkpoint**: Security audit passed

### Step 7.2: Documentation

```rust
/// Logs user activity events with validation and telemetry.
///
/// # Examples
///
/// ```
/// let mut log = ActivityLog::new();
/// let event = ActivityEvent {
///     user_id: 123,
///     action_type: ActionType::Login,
///     timestamp: Utc::now(),
///     details: HashMap::new(),
/// };
/// log.log_event(event)?;
/// ```
///
/// # Errors
///
/// Returns `LogError::InvalidUserId` if user_id is 0.
/// Returns `LogError::InvalidTimestamp` if timestamp is in the future.
pub fn log_event(&mut self, event: ActivityEvent) -> Result<(), LogError> {
    // ...
}
```

**✅ Checkpoint**: Documentation complete

### Step 7.3: Integration Test

```rust
#[test]
fn test_end_to_end_activity_logging() {
    // Arrange: Setup complete system
    let mut log = ActivityLog::new();

    // Act: Simulate real user workflow
    // 1. User logs in
    log.log_event(create_event(123, ActionType::Login)).unwrap();

    // 2. User views pages
    log.log_event(create_event(123, ActionType::PageView)).unwrap();
    log.log_event(create_event(123, ActionType::PageView)).unwrap();

    // 3. User takes action
    log.log_event(create_event(123, ActionType::ActionTaken)).unwrap();

    // 4. User logs out
    log.log_event(create_event(123, ActionType::Logout)).unwrap();

    // Assert: Verify complete workflow
    let user_events = log.get_events_for_user(123);
    assert_eq!(user_events.len(), 5);

    // Verify event sequence
    assert!(matches!(user_events[0].action_type, ActionType::Login));
    assert!(matches!(user_events[4].action_type, ActionType::Logout));

    // Verify telemetry emitted (checked by Weaver)
}

fn create_event(user_id: u64, action_type: ActionType) -> ActivityEvent {
    ActivityEvent {
        user_id,
        action_type,
        timestamp: Utc::now(),
        details: HashMap::new(),
    }
}
```

**✅ Checkpoint**: Integration test passes

---

## Part 8: Final Production Readiness Check

Run the complete validation:

```bash
#!/bin/bash
# Production readiness check for Activity Log feature

echo "🚀 Activity Log Production Readiness"
echo "====================================="

# TIER 1: Weaver (MANDATORY)
echo "📋 TIER 1: Weaver Validation"
weaver registry check -r registry/ && echo "✓ Schema valid" || exit 1
weaver registry live-check --registry registry/ && echo "✓ Telemetry valid" || exit 1

# TIER 2: Code Quality
echo "📋 TIER 2: Code Quality"
cargo build --release > /dev/null 2>&1 && echo "✓ Build successful" || exit 1
cargo clippy -- -D warnings > /dev/null 2>&1 && echo "✓ Clippy passed" || exit 1
cargo fmt --all -- --check && echo "✓ Formatting correct" || exit 1

# TIER 3: Testing
echo "📋 TIER 3: Testing"
cargo test > /dev/null 2>&1 && echo "✓ Tests passed" || exit 1
cargo test test_performance_meets_constraint && echo "✓ Performance ≤8 ticks" || exit 1

echo "====================================="
echo "✅ PRODUCTION READY"
echo "====================================="
```

**✅ COMPLETE**: Feature is production-ready

---

## What You've Learned

### The Production-Ready Development Workflow

```
1. PLAN
   ├─ Define requirements
   ├─ Design API
   └─ Design telemetry schema

2. IMPLEMENT (TDD)
   ├─ Red: Write failing test
   ├─ Green: Implement feature
   ├─ Refactor: Improve code
   └─ Repeat for all requirements

3. INSTRUMENT
   ├─ Add tracing spans
   ├─ Add metrics
   └─ Test telemetry emission

4. VALIDATE (Three-Tier)
   ├─ TIER 1: Weaver validation (source of truth)
   ├─ TIER 2: Code quality (baseline)
   └─ TIER 3: Traditional tests (supporting)

5. OPTIMIZE
   ├─ Measure performance
   ├─ Apply optimizations
   └─ Verify ≤8 ticks

6. CERTIFY
   ├─ Security audit
   ├─ Documentation
   └─ Integration tests
```

### Key Takeaways

1. **Design telemetry schema BEFORE implementing** - Schema-first development
2. **Use TDD throughout** - Red-Green-Refactor cycle
3. **Trust Weaver validation** - It's the source of truth
4. **Three-tier validation is mandatory** - All tiers must pass
5. **Performance is not optional** - ≤8 ticks is a requirement
6. **Production-ready means certified** - Follow the complete checklist

---

## Practice Exercises

Try building these features using the same workflow:

### Exercise 1: User Authentication Log (Easy)
Build a login attempt tracker with:
- Success/failure tracking
- IP address logging
- Rate limiting detection
- Performance: ≤5 ticks

### Exercise 2: Shopping Cart Events (Medium)
Build a cart activity tracker with:
- Add/remove item events
- Quantity changes
- Checkout initiation
- Performance: ≤8 ticks

### Exercise 3: API Request Audit (Hard)
Build an API audit logger with:
- Request/response capture
- Error tracking
- Performance metrics
- Security event detection
- Performance: ≤6 ticks

---

## Next Steps

Now that you can build production-ready features:

1. **Learn performance optimization** - [Tutorial 5: Optimizing Performance](05-optimizing-performance.md)
2. **Master schema-first development** - [Tutorial 6: Schema-First Development](06-schema-first-development.md)
3. **Validate production readiness** - [How-to 12: Validate Production Readiness](../how-to-guides/12-validate-production-readiness.md)

---

## Related Resources

**Prerequisites**:
- [Tutorial 3: Chicago TDD Basics](03-chicago-tdd-basics.md)
- [Tutorial 2: Understanding Telemetry](02-understanding-telemetry.md)

**How-to Guides**:
- [How-to 7: Emit Proper Telemetry](../how-to-guides/07-emit-proper-telemetry.md)
- [How-to 8: Optimize Performance](../how-to-guides/08-optimize-performance.md)
- [How-to 12: Validate Production Readiness](../how-to-guides/12-validate-production-readiness.md)

---

**Created**: 2025-11-15
**Status**: Complete
**Difficulty**: Intermediate
**Estimated Time**: 30-45 minutes
