# How-to Guide 7: Emit Proper Telemetry

## Goal

Properly instrument your code with OpenTelemetry to emit meaningful, performant telemetry that enables Weaver validation while respecting the ≤8 tick performance constraint.

**Time Estimate**: 2-3 hours
**Prerequisites**: [Understanding Telemetry](../tutorials/02-understanding-telemetry.md), [Create OTel Schemas](05-create-otel-schemas.md), [Add New Features](04-add-new-features.md)
**Difficulty**: Intermediate
**Outcomes**: Production-ready instrumented code with validated telemetry

---

## What "Proper" Telemetry Means

### The Three Aspects of Proper Telemetry

```
1. COMPLETE: Emits all declared telemetry
   └─ If schema says span, code emits span
   └─ If schema says metric, code records metric

2. CORRECT: Types and values match schema
   └─ All attributes present with right types
   └─ All metrics use correct type (counter/histogram/gauge)
   └─ All logs have declared fields

3. PERFORMANT: Overhead is minimal (≤8 ticks total)
   └─ Instrumentation adds <1 tick overhead
   └─ No excessive allocations
   └─ Strategic measurement, not exhaustive
```

### Proper vs. Improper Telemetry

**❌ Improper (Common Mistakes)**:
```rust
// Too much logging (excessive overhead)
for item in items {
    info!("Processing item: {:?}", item);  // Called 1000x per second
}

// Measuring trivial operations
#[instrument]
fn add_numbers(a: i32, b: i32) -> i32 {  // This is too fine-grained
    a + b
}

// Not measuring critical operations
async fn fetch_from_database() {
    // No telemetry!
    db.query().await
}

// Wrong metric types
counter!("latency_ms", duration_ms);  // Should be histogram!
histogram!("total_requests", 1);      // Should be counter!
```

**✅ Proper (Best Practices)**:
```rust
// Strategic logging (one per batch)
info!(item_count = items.len(), "Processing batch");
for item in items {
    process(item);  // Silent, measured at batch level
}

// Measuring where it matters
#[instrument]
async fn fetch_from_database() {
    // Overhead: ~0.2 ticks
    // Value: Reveals database performance bottleneck
    db.query().await
}

// Correct metric types
histogram!("query_latency_ms", duration_ms);  // Distribution
counter!("queries_total", 1);                 // Always-increasing count
gauge!("connections_active", conn_count);    // Current value
```

---

## Part 1: The Instrumentation API

### Method 1: `#[instrument]` Macro (Simplest)

**Best for**: Functions that should always be traced

```rust
use tracing::instrument;

#[instrument]  // Automatically creates span
fn process_order(order_id: u64, amount: f64) -> Result<()> {
    validate_order(order_id)?;
    charge_customer(amount)?;
    ship_order(order_id)?;
    Ok(())
}

#[instrument(skip(data))]  // Skip large data structures
fn store_data(order_id: u64, data: &[u8]) -> Result<()> {
    // Won't log `data` parameter (could be huge)
    database.store(order_id, data)?;
    Ok(())
}

#[instrument(fields(correlation_id))]  // Add custom fields
fn handle_request(req: Request) {
    Span::current().record("correlation_id", &req.id);
    // ...
}
```

**What `#[instrument]` captures automatically**:
- ✅ Function name as span name
- ✅ All parameters as attributes (unless skipped)
- ✅ Return value (Ok/Err) as status
- ✅ Execution duration

**Overhead**: ~0.1-0.2 ticks

### Method 2: Manual Spans (Fine Control)

**Best for**: Complex operations with conditional logic

```rust
use tracing::{info_span, Instrument};

async fn complex_workflow() {
    let span = info_span!("workflow", user_id = 123);

    async {
        step_one().await;
        step_two().await;
        step_three().await;
    }
    .instrument(span)
    .await
}
```

Or for synchronous code:

```rust
fn complex_operation() {
    let span = info_span!("operation");
    let _guard = span.enter();

    // Everything here is in the span
    step_one();
    step_two();
    step_three();
}  // Span exits when _guard is dropped
```

**Overhead**: ~0.15-0.3 ticks

### Method 3: Events (Logging)

**Best for**: Recording important events within a span

```rust
use tracing::{info, warn, error, debug};

#[instrument]
fn process(id: u64) {
    debug!("Processing started");  // Debug level

    if !validate(id) {
        warn!(id = id, "Validation failed");  // Warning level
        return;
    }

    match execute(id) {
        Ok(result) => {
            info!(result = ?result, "Processing complete");
        }
        Err(e) => {
            error!(error = %e, "Processing failed");
        }
    }
}
```

**Overhead**: ~0.05 ticks per event

### Method 4: Metrics (Measurements)

**Best for**: Quantitative measurements

```rust
use metrics::{counter, histogram, gauge};

#[instrument]
fn handle_request() {
    let start = Instant::now();

    // Do work...

    let duration = start.elapsed().as_millis() as f64;

    // Record metrics
    counter!("requests_total", 1);
    histogram!("request_duration_ms", duration);
    gauge!("requests_active", active_count());
}
```

**Overhead**: ~0.05 ticks per metric

---

## Part 2: Strategic Instrumentation

### Where to Instrument: The Pyramid

```
┌─────────────────────────────┐
│  High-Value Operations      │ ← Instrument aggressively
│  (API endpoints, DB, I/O)   │
├─────────────────────────────┤
│  Medium-Value Operations    │ ← Instrument selectively
│  (Business logic)           │
├─────────────────────────────┤
│  Low-Value Operations       │ ← Don't instrument
│  (Utility functions)        │
└─────────────────────────────┘
```

### Tier 1: Always Instrument

```rust
// API endpoints
#[instrument]
async fn create_user(req: CreateUserRequest) -> Result<Response> { }

// Database operations
#[instrument]
async fn query_users(filter: &Filter) -> Result<Vec<User>> { }

// External service calls
#[instrument]
async fn call_payment_provider(amount: Money) -> Result<Receipt> { }

// State transitions
#[instrument]
fn activate_account(account: &mut Account) -> Result<()> { }
```

### Tier 2: Selective Instrumentation

```rust
// Complex business logic (not every helper)
#[instrument]
fn calculate_discount(customer: &Customer, amount: Money) -> Money { }

// Retry/fallback operations
#[instrument]
async fn fetch_with_retry(url: &str, max_retries: u32) -> Result<Data> { }

// Long-running operations
#[instrument]
fn process_batch(items: &[Item]) -> Result<Vec<Output>> { }
```

### Tier 3: Don't Instrument

```rust
// Simple getters/setters (too much noise)
fn get_user_id(&self) -> u64 { self.id }

// Trivial calculations
fn add(a: i32, b: i32) -> i32 { a + b }

// Already instrumented by caller
fn validate_email(email: &str) -> bool {  // Called by already-instrumented function
    email.contains('@')
}
```

---

## Part 3: Performance-Conscious Instrumentation

### Principle: Measure, Don't Log Everything

```rust
// ❌ WRONG: Logs inside loop (expensive)
for order in orders {
    info!("Processing order: {}", order.id);  // Called 1000x
    process_order(order);
}

// ✅ CORRECT: One log for batch
info!(order_count = orders.len(), "Processing batch");
for order in orders {
    process_order(order);  // Silent
}
```

### Conditional Instrumentation

```rust
// Only trace when enabled (zero cost if disabled)
if tracing::enabled!(target: "slow_queries") {
    debug!("Slow query execution");
}

// Sample: Trace only 1% of requests
if rand::random::<f32>() < 0.01 {
    span.record("sampled", true);
}
```

### Async Instrumentation

```rust
use tracing::Instrument;

// Instrument async operations
async fn fetch_data() {
    let span = info_span!("fetch");
    async_work().instrument(span).await
}

// Or with macro
#[instrument]
async fn fetch_data() {
    async_work().await
}
```

### Attribute Recording

```rust
// Record attributes efficiently
#[instrument(skip(large_data))]  // Skip parameter
fn process(id: u64, large_data: &[u8]) {
    // Good: Just the ID is recorded
    //
    // Record additional data only if needed
    if id > 1000 {
        Span::current().record("large_id", true);
    }
}
```

---

## Part 4: Common Patterns

### Pattern 1: Request/Response Cycle

```rust
#[instrument(skip(body))]
async fn handle_request(req: Request) -> Response {
    info!("Request received");

    match process(req.body) {
        Ok(result) => {
            counter!("requests_success_total", 1);
            info!("Request completed successfully");
            Response::ok(result)
        }
        Err(e) => {
            counter!("requests_error_total", 1);
            error!(error = %e, "Request failed");
            Response::error(e)
        }
    }
}
```

### Pattern 2: Retry with Backoff

```rust
#[instrument]
async fn fetch_with_retries(url: &str) -> Result<Data> {
    for attempt in 1..=3 {
        match fetch(url).await {
            Ok(data) => {
                info!(attempt = attempt, "Fetch succeeded");
                return Ok(data);
            }
            Err(e) if attempt < 3 => {
                warn!(
                    attempt = attempt,
                    error = %e,
                    "Fetch failed, retrying"
                );
                tokio::time::sleep(Duration::from_secs(2_u64.pow(attempt))).await;
            }
            Err(e) => {
                error!(attempt = attempt, error = %e, "Fetch failed permanently");
                return Err(e);
            }
        }
    }
    Ok(Default::default())
}
```

### Pattern 3: Multi-Stage Operation

```rust
#[instrument]
fn multi_stage_process(input: Input) -> Result<Output> {
    debug!("Starting multi-stage process");

    let stage1 = {
        let _span = info_span!("stage_1").entered();
        stage_one(input)?
    };

    let stage2 = {
        let _span = info_span!("stage_2").entered();
        stage_two(stage1)?
    };

    let result = {
        let _span = info_span!("stage_3").entered();
        stage_three(stage2)?
    };

    info!("Multi-stage process completed");
    Ok(result)
}
```

### Pattern 4: Error Tracking

```rust
#[instrument]
fn operation_with_errors() -> Result<Success> {
    match step_one() {
        Ok(val) => {
            match step_two(val) {
                Ok(result) => {
                    counter!("operations_success_total", 1);
                    Ok(result)
                }
                Err(e) => {
                    counter!("operations_error_stage2_total", 1);
                    error!(stage = "stage_2", error = %e, "Operation failed");
                    Err(e)
                }
            }
        }
        Err(e) => {
            counter!("operations_error_stage1_total", 1);
            error!(stage = "stage_1", error = %e, "Operation failed");
            Err(e)
        }
    }
}
```

---

## Part 5: Validating Your Instrumentation

### Step 1: Code Review Checklist

- [ ] All public functions have `#[instrument]`
- [ ] Database queries instrumented
- [ ] External service calls instrumented
- [ ] Error paths logged with context
- [ ] No excessive logging in loops
- [ ] Sensitive data not logged

### Step 2: Schema Validation

```bash
# Verify schema defines all your telemetry
weaver registry check -r registry/

# Expected: All spans/metrics/logs validated
```

### Step 3: Live Testing

```bash
# Run with trace logging
RUST_LOG=trace cargo test

# Verify output:
# - Correct span names
# - All attributes present
# - Correct types
# - Reasonable overhead
```

### Step 4: Weaver Live Validation

```bash
# Run code and validate telemetry
weaver registry live-check --registry registry/

# Should pass without errors
```

### Step 5: Performance Validation

```bash
# Verify ≤8 ticks constraint
make test-performance-v04

# Check for telemetry overhead
# Should be <1 tick added to operation time
```

---

## Part 6: Troubleshooting

### Issue: Missing Telemetry

**Problem**: Code runs but Weaver validation fails

**Solution**:
```rust
// Verify span/metric is emitted
#[instrument]
fn operation() {
    // Span automatically created
    counter!("operations_total", 1);  // Metric emitted
    info!("Operation complete");  // Log emitted
}

// Run with trace output
RUST_LOG=trace cargo test
// Should see: Span created, metric recorded, log emitted
```

### Issue: Type Mismatch

**Problem**: Weaver validation fails with type error

**Solution**:
```rust
// Check types match schema
#[instrument]
fn operation(user_id: u64) {
    // Schema expects: user_id as int
    // Code emits: user_id = 12345 ✓
    // Types match!
}
```

### Issue: Excessive Overhead

**Problem**: Telemetry adds more than 1 tick

**Solution**:
```rust
// Check for expensive operations in hot paths
#[instrument(skip(large_vec))]  // Skip expensive clones
fn process(items: &[Item], large_vec: Vec<Item>) {
    // Don't log every item
    info!(count = items.len(), "Processing");
    for item in items {
        process_one(item);  // Silent
    }
}
```

### Issue: Incomplete Attributes

**Problem**: Schema requires attribute but code doesn't emit

**Solution**:
```rust
// Schema defines as required:
// user_id: {required: true}

// Make sure code captures it:
#[instrument]  // Auto-captures parameters
fn operation(user_id: u64) {
    // user_id automatically captured
}

// Or manually:
#[instrument(fields(user_id))]
fn operation(id: u64) {
    Span::current().record("user_id", &id);
}
```

---

## Part 7: Integration with Features

### Using in [How-to: Add New Features](04-add-new-features.md)

When you create a feature, remember:

1. **Schema First**
   - Define what telemetry in schema
   - See [How-to: Create OTel Schemas](05-create-otel-schemas.md)

2. **Instrument**
   - Use this guide to emit telemetry
   - Match schema exactly

3. **Validate**
   - Run Weaver validation
   - Check Chatman Constant (≤8 ticks)

4. **Verify**
   - See [How-to: Fix Weaver Validation Errors](06-fix-weaver-validation-errors.md) if issues

---

## Part 8: Best Practices Summary

### DO

✅ Use `#[instrument]` for most functions
✅ Instrument at logical boundaries
✅ Skip large data structures
✅ Use appropriate log levels
✅ Record metrics at operation completion
✅ Include error context in logs
✅ Test instrumentation with RUST_LOG=trace
✅ Validate with Weaver

### DON'T

❌ Log inside tight loops (too much overhead)
❌ Log sensitive data (passwords, tokens, PII)
❌ Use wrong metric types (counter vs histogram)
❌ Instrument trivial operations
❌ Skip important operations
❌ Assume telemetry works without validation
❌ Exceed ≤8 tick constraint

---

## Quick Reference

| Situation | Solution | Example |
|-----------|----------|---------|
| **Need simple span** | Use `#[instrument]` | `#[instrument]` fn operation() {} |
| **Need fine control** | Manual span | `info_span!("name").entered()` |
| **Record event** | Use `info!`, `warn!`, `error!` | `info!("Something happened")` |
| **Measure quantity** | Use metrics | `counter!("total", 1)` |
| **Skip parameter** | `skip()` | `#[instrument(skip(large))]` |
| **Custom field** | `fields()` | `#[instrument(fields(id))]` |
| **Async function** | `#[instrument]` works | `#[instrument] async fn() {}` |
| **Conditional log** | Check `enabled!()` | `if tracing::enabled!(debug) {}` |

---

## Summary

### Proper Telemetry

- **Complete**: Everything schema declares is emitted
- **Correct**: Types and values match
- **Performant**: <1 tick overhead per operation

### Implementation

1. Use `#[instrument]` for main functions
2. Add events (`info!`, `warn!`, `error!`)
3. Record metrics at key points
4. Validate with Weaver
5. Test performance

### The Rule

**If something matters, measure it. If it doesn't matter, don't measure it.**

---

**Created**: 2025-11-15
**Status**: Complete
**Difficulty**: Intermediate
**Next**: [How-to: Optimize Performance](08-optimize-performance.md)
