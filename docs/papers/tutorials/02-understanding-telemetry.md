# Tutorial 2: Understanding Telemetry

## Learning Objectives

By the end of this tutorial, you will:
- Understand why telemetry is critical to KNHK's validation approach
- Know the difference between traditional testing and telemetry validation
- Understand OpenTelemetry fundamentals (spans, metrics, logs)
- Know where and how to add telemetry to KNHK code
- Understand how telemetry enables Weaver validation
- Be able to emit and verify basic telemetry

**Estimated Time**: 2 hours
**Prerequisites**: [Tutorial 1: Your First KNHK Workflow](01-getting-started.md) completed
**Difficulty**: Beginner to Intermediate

---

## Part 1: The Telemetry Problem

### Traditional Testing: The False Positive Problem

In traditional software development, we validate code like this:

```rust
#[test]
fn test_feature() {
    let result = my_feature(input);
    assert_eq!(result, expected);  // ✓ Test passes
}
```

**What this proves:**
- ✅ The test code executed
- ✅ The assertion was true
- ❌ Does NOT prove the feature actually works in production
- ❌ Does NOT prove proper behavior
- ❌ Can pass while feature is broken

**Why?** The test only validates test logic, not production behavior.

### KNHK's Solution: Telemetry as Source of Truth

```
Production Code
    ↓
Emits Telemetry (Spans, Metrics, Logs)
    ↓
Telemetry sent to Weaver
    ↓
Weaver validates against schema
    ↓
Schema violation? → Feature doesn't work
    ↓
Schema match? → Feature actually works ✓
```

**Why this is better:**
- ✅ Validates actual runtime behavior
- ✅ Not fooled by test mocks
- ✅ Proves feature works in production
- ✅ Cannot pass if feature is broken

---

## Part 2: What is Telemetry?

### Definition

**Telemetry** = Data your application emits about its behavior during execution

**Three types of telemetry:**
1. **Spans** - Track execution flow (request → response)
2. **Metrics** - Quantitative measurements (counters, histograms, gauges)
3. **Logs** - Event records with context

### Why Telemetry Matters

```
Without Telemetry:
  Code runs → Unclear what happened
  Test passes → Might be false positive
  Production bug → Mystery where it broke

With Telemetry:
  Code runs → Detailed execution trace visible
  Test passes → Validated by schema conformance
  Production bug → Root cause visible in traces
```

### Real-World Example

**Scenario**: User submits a request to create an account

**With Traditional Testing**:
```rust
#[test]
fn test_create_account() {
    let user = create_account("alice@example.com");
    assert!(user.is_some());  // ✓ Passes
}
// But: Does it validate email? Does it hash password? Unknown.
```

**With Telemetry**:
```
Span: "create_account"
  ├─ Attribute: email = "alice@example.com"
  ├─ Metric: account_creation_duration_ms = 45
  ├─ Span: "validate_email" (child)
  │   └─ Status: OK
  ├─ Span: "hash_password" (child)
  │   └─ Status: OK
  └─ Status: OK

Schema validates this structure
✓ Proves all steps executed
✓ Proves proper order
✓ Proves timing is correct
```

---

## Part 3: OpenTelemetry Fundamentals

### What is OpenTelemetry (OTEL)?

**OpenTelemetry** = Industry standard for application telemetry

**Key characteristics:**
- ✅ Vendor-neutral
- ✅ Multiple languages supported
- ✅ Proven at scale (millions of deployments)
- ✅ Designed for production use
- ✅ Excellent documentation and community

### Why OpenTelemetry?

```
Old Way (Application-Specific):
  My logging library
  My metrics library
  My tracing library
  → Hard to integrate, vendor lock-in

OpenTelemetry Way:
  Single standard (OTEL)
  Works with any backend
  Export format (OTLP)
  → Flexible, portable, standard

KNHK + OpenTelemetry:
  OTEL provides telemetry
  KNHK validates with schemas
  Weaver ensures compliance
  → Bulletproof validation
```

### OTEL Architecture

```
Your Code
    ↓
OTEL Instrumentation (SDK)
    ├─ Spans
    ├─ Metrics
    └─ Logs
    ↓
OTEL Exporter
    (converts to standard format)
    ↓
Backend (Weaver, Jaeger, Datadog, etc.)
    ↓
Validation/Analysis/Visualization
```

---

## Part 4: The Three Pillars

### Pillar 1: Spans (Distributed Tracing)

**What is a span?**
A span represents a single operation within a request.

**Real-world analogy**: Like putting checkpoints along a road
```
Start: 9:00:00 AM
  ├─ Checkpoint A: 9:00:05 AM (duration: 5s)
  ├─ Checkpoint B: 9:00:12 AM (duration: 7s)
  ├─ Checkpoint C: 9:00:15 AM (duration: 3s)
End: 9:00:15 AM (total: 15s)
```

**Code example**:
```rust
use tracing::{info, instrument};

#[instrument]  // ← Automatically creates a span
fn process_request(user_id: u64) {
    info!("Starting request processing");  // ← Event in span

    validate_user(user_id);
    load_data();
    transform();
    save_result();

    info!("Request processing complete");
}

#[instrument]  // ← Nested span
fn validate_user(user_id: u64) {
    // This becomes a child span
}
```

**Span structure**:
```
Span: "process_request"
├─ Start time: 2025-11-15T10:30:00Z
├─ Duration: 150ms
├─ Attributes:
│   ├─ user_id: 12345
│   ├─ request_id: abc-def-123
│   └─ user_role: "admin"
├─ Events:
│   ├─ "validation_passed"
│   ├─ "data_loaded"
│   └─ "result_saved"
├─ Status: OK
└─ Child spans:
    ├─ validate_user (25ms)
    ├─ load_data (80ms)
    ├─ transform (20ms)
    └─ save_result (15ms)
```

### Pillar 2: Metrics (Quantitative Measurements)

**What are metrics?**
Numerical measurements of system behavior.

**Types of metrics**:
1. **Counter**: Always increases (e.g., requests received)
   ```rust
   counter!("requests_total", 1);  // Increments by 1
   ```

2. **Histogram**: Distribution of values (e.g., request latency)
   ```rust
   histogram!("request_duration_ms", 145);  // Records value
   ```

3. **Gauge**: Current value (e.g., active connections)
   ```rust
   gauge!("active_connections", 42);  // Sets to value
   ```

**Real example**:
```
Metric: "http_requests_total"
├─ Value: 15,234
├─ Attributes:
│   ├─ method: "GET"
│   ├─ status: "200"
│   └─ endpoint: "/api/users"

Metric: "request_duration_ms" (histogram)
├─ Values: [45, 52, 48, 51, 49, ...]
├─ Statistics:
│   ├─ Mean: 50ms
│   ├─ P95: 55ms
│   ├─ P99: 60ms
│   └─ Max: 75ms
```

### Pillar 3: Logs (Event Records)

**What are logs?**
Structured event records with context.

**Old way (bad)**:
```
2025-11-15 10:30:45 ERROR: Something failed
```

**OTEL way (good)**:
```rust
use tracing::error;

error!(
    user_id = 12345,
    request_id = "abc-123",
    error_code = "VALIDATION_FAILED",
    details = "Email format invalid",
    "User validation failed"
);

// Produces structured log:
// {
//   "timestamp": "2025-11-15T10:30:45Z",
//   "level": "ERROR",
//   "message": "User validation failed",
//   "user_id": 12345,
//   "request_id": "abc-123",
//   "error_code": "VALIDATION_FAILED",
//   "details": "Email format invalid"
// }
```

**Why structured logs matter**:
- ✅ Searchable and filterable
- ✅ Context preserved
- ✅ Machine-readable
- ✅ Correlatable with traces

### How They Work Together

```
Request arrives
    ↓
Span: "request_handler" starts
├─ Log: "Request received" {endpoint: "/api/users"}
├─ Span: "authorization" (child)
│   ├─ Log: "Checking auth" {token: "..."}
│   └─ Status: OK
├─ Span: "fetch_user_data" (child)
│   ├─ Log: "Database query started"
│   ├─ Metric: "db_query_duration_ms" = 45
│   └─ Log: "Database query complete"
├─ Log: "Processing request" {user_id: 123}
└─ Metric: "request_duration_ms" = 120

Request completes (total: 120ms)

Result: Complete trace of what happened, when, and how long
```

---

## Part 5: KNHK Instrumentation Patterns

### Pattern 1: Automatic with `#[instrument]`

**Simplest approach:**
```rust
use tracing::instrument;

#[instrument]  // ← Automatically creates span
fn process_order(order_id: u64) {
    validate_order();
    charge_payment();
    ship_order();
}

#[instrument]  // ← Child span
fn validate_order() {
    // ...
}
```

**What `#[instrument]` does**:
- ✅ Creates span with function name
- ✅ Captures function arguments as attributes
- ✅ Records function result (success/error)
- ✅ Measures execution duration

**Output**:
```
Span: "process_order"
├─ Attributes:
│   └─ order_id: 12345
├─ Duration: 250ms
├─ Status: OK
└─ Child spans:
    ├─ validate_order
    ├─ charge_payment
    └─ ship_order
```

### Pattern 2: Manual Spans for Fine Control

```rust
use tracing::{Span, info};

fn complex_operation() {
    let span = tracing::span!(tracing::Level::INFO, "operation");
    let _guard = span.enter();

    info!(step = 1, "Starting step 1");
    step1();

    info!(step = 2, "Starting step 2");
    step2();

    info!(step = 3, "Starting step 3");
    step3();
}
```

### Pattern 3: Metrics for Performance

```rust
use metrics::{counter, histogram};

fn request_handler() {
    let start = std::time::Instant::now();

    // Do work
    process_request();

    let duration = start.elapsed().as_millis() as f64;

    // Record metrics
    counter!("requests_total", 1);
    histogram!("request_duration_ms", duration);
}
```

### Pattern 4: Structured Logging with Context

```rust
use tracing::{info, error, warn};

fn handle_user_action(user_id: u64, action: &str) {
    info!(
        user_id = user_id,
        action = action,
        "User action started"
    );

    match execute_action(action) {
        Ok(result) => {
            info!(
                user_id = user_id,
                action = action,
                result = ?result,
                "Action completed successfully"
            );
        }
        Err(e) => {
            error!(
                user_id = user_id,
                action = action,
                error = %e,
                "Action failed"
            );
        }
    }
}
```

---

## Part 6: KNHK-Specific Instrumentation

### Where to Add Telemetry

**Rule**: Every user-visible operation should have telemetry

```
API Endpoint
    ↓ [Span: "handle_request"]
    ├─ Authorization
    │   ↓ [Span: "authorize_request"]
    ├─ Database Query
    │   ↓ [Span: "fetch_user"]
    │   ↓ [Metric: "db_query_duration"]
    ├─ Processing
    │   ↓ [Span: "process_data"]
    ├─ Validation
    │   ↓ [Span: "validate_result"]
    └─ Response
        ↓ [Metric: "response_time"]
        ↓ [Log: "Request completed"]
```

### Performance Constraint: ≤8 Ticks

**Remember**: KNHK enforces ≤8 ticks (Chatman Constant)

**Telemetry overhead**:
- Span creation: ~0.1 ticks
- Metric recording: ~0.05 ticks
- Log emission: ~0.05 ticks
- Total overhead: Minimal if done right

**Good practice**:
```rust
#[instrument]  // Minimal overhead
fn operation() {
    counter!("operations_total", 1);  // Fast
    histogram!("duration", elapsed);  // Fast
}
```

**Bad practice**:
```rust
#[instrument]
fn operation() {
    for i in 0..1000000 {
        info!("Loop iteration {}", i);  // ✗ Excessive overhead
    }
}
```

---

## Part 7: Hands-on Example

### Build a Simple Telemetry Feature

Let's create a user registration feature with proper telemetry.

### Step 1: Create the Feature Module

```rust
// src/registration.rs
use tracing::{instrument, info, error};
use metrics::{counter, histogram};

#[instrument]
pub fn register_user(email: &str, password: &str) -> Result<User, RegistrationError> {
    info!("User registration started");

    let start = std::time::Instant::now();

    // Validation
    validate_email(email)?;
    validate_password(password)?;

    // Storage
    let user = create_user_in_db(email, password)?;

    let duration_ms = start.elapsed().as_millis() as f64;

    // Metrics
    counter!("user_registrations_total", 1);
    histogram!("registration_duration_ms", duration_ms);

    info!("User registration completed");

    Ok(user)
}

#[instrument]
fn validate_email(email: &str) -> Result<(), RegistrationError> {
    if email.contains('@') {
        info!("Email validation passed");
        Ok(())
    } else {
        error!("Email validation failed");
        Err(RegistrationError::InvalidEmail)
    }
}

#[instrument]
fn validate_password(password: &str) -> Result<(), RegistrationError> {
    if password.len() >= 8 {
        info!("Password validation passed");
        Ok(())
    } else {
        error!("Password too short");
        Err(RegistrationError::WeakPassword)
    }
}

#[instrument]
fn create_user_in_db(email: &str, password: &str) -> Result<User, RegistrationError> {
    // Simulate database operation
    std::thread::sleep(std::time::Duration::from_millis(10));

    info!("User created in database");

    Ok(User {
        id: 123,
        email: email.to_string(),
    })
}
```

### Step 2: Write Tests with Telemetry Validation

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_successful_registration() {
        // Initialize tracing for the test
        // (normally done once at startup)

        let result = register_user("alice@example.com", "password123");

        assert!(result.is_ok());

        // Telemetry emitted:
        // ✓ Span: "register_user"
        // ✓ Span: "validate_email" (child)
        // ✓ Span: "validate_password" (child)
        // ✓ Span: "create_user_in_db" (child)
        // ✓ Metric: "user_registrations_total" = 1
        // ✓ Metric: "registration_duration_ms" = ~20ms
    }

    #[test]
    fn test_invalid_email() {
        let result = register_user("invalid-email", "password123");

        assert!(result.is_err());

        // Telemetry shows:
        // ✓ Span: "register_user"
        // ✓ Span: "validate_email" (FAILED)
        // ✗ Span: "validate_password" (not reached)
        // ✗ Span: "create_user_in_db" (not reached)
    }

    #[test]
    fn test_weak_password() {
        let result = register_user("alice@example.com", "short");

        assert!(result.is_err());

        // Telemetry shows:
        // ✓ Span: "register_user"
        // ✓ Span: "validate_email" (passed)
        // ✓ Span: "validate_password" (FAILED)
        // ✗ Span: "create_user_in_db" (not reached)
    }
}
```

### Step 3: Verify Telemetry Output

Run the code and verify telemetry:

```bash
RUST_LOG=info cargo test test_successful_registration -- --nocapture

# Output shows:
# INFO registration: User registration started
# INFO registration: validate_email: Email validation passed
# INFO registration: validate_password: Password validation passed
# INFO registration: create_user_in_db: User created in database
# INFO registration: User registration completed
```

### Step 4: Validate with Weaver

After implementing telemetry, Weaver validates:

```bash
weaver registry check -r registry/

# Checks:
# ✓ Schema defines "register_user" span
# ✓ Schema defines "validate_email" span
# ✓ Schema defines "validate_password" span
# ✓ Schema defines "user_registrations_total" metric
# ✓ Schema defines "registration_duration_ms" metric
# ✓ All attributes match schema
# ✓ All metrics match schema
```

---

## Part 8: Troubleshooting

### Issue: Telemetry Not Showing Up

**Problem**: You added instrumentation but see no output

**Solution**:
```rust
// Make sure initialization is done at startup
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .init();

// Or with RUST_LOG environment variable
RUST_LOG=info cargo run
```

### Issue: Missing Child Spans

**Problem**: Expected nested spans but they're not appearing

**Solution**:
```rust
// Wrong: No span context passed
fn outer() {
    inner();  // inner's span won't be a child
}

#[instrument]
fn inner() {
    // This becomes a root span, not a child
}

// Correct: Using #[instrument] creates proper hierarchy
#[instrument]
fn outer() {
    inner();  // inner's span is now a child
}

#[instrument]
fn inner() {
    // This becomes a child span
}
```

### Issue: Excessive Overhead

**Problem**: Telemetry is slowing down code (exceeds 8 ticks)

**Solution**:
```rust
// Bad: Logging in hot loop
for item in items {
    info!("Processing: {:?}", item);  // Too much logging
}

// Good: Log once with count
info!("Processing {} items", items.len());
for item in items {
    // Process without logging each iteration
}

// Or: Use sampling
if i % 100 == 0 {
    debug!("Processed {} items", i);  // Log every 100th
}
```

### Issue: Weaver Validation Fails

**Problem**: Telemetry emitted but Weaver validation fails

**Solution**:
```bash
# Step 1: Check schema validity
weaver registry check -r registry/

# Step 2: Check what telemetry is actually emitted
RUST_LOG=debug cargo test  # View spans/metrics

# Step 3: Update schema to match code
# Edit registry/schemas/*.yaml
# Add missing spans/metrics/attributes

# Step 4: Re-validate
weaver registry live-check --registry registry/
```

### Issue: Performance Regression

**Problem**: Added telemetry and now tests exceed 8 ticks

**Solution**:
1. Profile with flamegraph
2. Identify telemetry hotspots
3. Move instrumentation out of hot loops
4. Use conditional instrumentation
5. Optimize metric recording

```rust
// Before: Excessive instrumentation in hot path
for i in 0..10000 {
    counter!("iterations", 1);  // ✗ Called 10k times
}

// After: Instrument loop, not iteration
counter!("loop_count", iterations);  // ✓ Called once
```

---

## Part 9: Best Practices

### DO: Instrument Strategically
```rust
✓ API endpoints
✓ Database operations
✓ External service calls
✓ Error paths
✓ State changes
```

### DON'T: Over-instrument
```rust
✗ Inside tight loops
✗ Per-iteration operations
✗ Trivial calculations
✗ Frequently-called helpers
✗ Non-critical operations
```

### DO: Use Meaningful Attributes
```rust
✓ User IDs
✓ Request IDs
✓ Resource identifiers
✓ Operation status
✓ Duration/timing
```

### DON'T: Log Sensitive Data
```rust
✗ Passwords
✗ API tokens
✗ Credit card numbers
✗ PII (personal info)
✗ Unencrypted secrets
```

---

## Part 10: What You've Learned

### Key Concepts
✅ Why telemetry matters for validation
✅ How OpenTelemetry works
✅ The three pillars (Spans, Metrics, Logs)
✅ KNHK instrumentation patterns
✅ Performance constraints (≤8 ticks)
✅ How to build and test telemetry

### Practical Skills
✅ Use `#[instrument]` macro
✅ Emit metrics
✅ Create structured logs
✅ Build proper span hierarchy
✅ Test telemetry validation

### Next Steps
1. **Practice**: Add telemetry to a simple feature
2. **Reference**: [How-to: Emit Proper Telemetry](../how-to-guides/07-emit-proper-telemetry.md) (coming soon)
3. **Validate**: Learn schemas with [How-to: Create OTel Schemas](../how-to-guides/05-create-otel-schemas.md) (coming soon)
4. **Build**: Use in [How-to: Add New Features](../how-to-guides/04-add-new-features.md) (coming soon)

---

## Summary

### Understanding Telemetry in KNHK

**Traditional approach:**
```
Code → Test → Pass/Fail
      (May be false positive)
```

**KNHK approach:**
```
Code → Emit Telemetry → Weaver Validates → Proof it works
      (Impossible to fake)
```

**Three pillars:**
- **Spans**: Track execution flow
- **Metrics**: Measure quantities
- **Logs**: Record events

**Best practice:**
- Instrument strategically
- Avoid excessive overhead
- Keep ≤8 ticks constraint
- Validate with Weaver

---

## Resources

**Quick Reference**:
- OTEL Docs: https://opentelemetry.io
- Tracing Crate: https://docs.rs/tracing
- Metrics Crate: https://docs.rs/metrics

**KNHK Specific**:
- [How-to: Create OTel Schemas](../how-to-guides/05-create-otel-schemas.md) (coming soon)
- [How-to: Fix Weaver Errors](../how-to-guides/06-fix-weaver-validation-errors.md) (coming soon)
- [How-to: Emit Proper Telemetry](../how-to-guides/07-emit-proper-telemetry.md) (coming soon)
- [Reference: Chatman Equation](../reference/the_chatman_equation_fortune5_v1.2.0.pdf)

---

## Practice Exercise

### Challenge: Instrument a Simple Feature

1. **Create** a new function: `calculate_fibonacci(n: u32) -> u32`
2. **Add telemetry**:
   - Span for function execution
   - Metric for calculation time
   - Log for start and end
   - Log for errors
3. **Write tests** that verify telemetry
4. **Run with Weaver** validation (once schemas are added)

**Expected output**:
```
Span: "calculate_fibonacci"
├─ Attributes:
│   └─ n: 10
├─ Metric: "fibonacci_duration_ms" = 0.5
└─ Logs:
    ├─ "Calculating fibonacci"
    └─ "Fibonacci calculation complete: 55"
```

---

**Created**: 2025-11-15
**Level**: Beginner to Intermediate
**Status**: Complete
**Next Tutorial**: [Tutorial 3: Chicago TDD Basics](03-chicago-tdd-basics.md) (coming soon)
