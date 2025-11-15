# How-to Guide: Emit Proper Telemetry

**Goal**: Add OpenTelemetry instrumentation to your code
**Time**: 15 minutes
**Difficulty**: Beginner

## Overview

Proper telemetry means:
- Every important operation emits traces
- All data points are documented
- Telemetry matches your schema
- Code behavior is provable

## Step 1: Add Tracing Macro

Mark functions that should emit telemetry:

```rust
use tracing::instrument;

#[instrument]
fn my_function(param: &str) -> Result<String> {
    // Automatically creates a span named "my_function"
    Ok(format!("processed: {}", param))
}
```

## Step 2: Use Logging Levels

```rust
use tracing::{debug, info, warn, error};

#[instrument]
fn process_data(input: &str) -> Result<String> {
    info!("Starting data processing");  // INFO level

    if input.is_empty() {
        warn!("Received empty input");  // WARNING level
        return Err("Empty input".into());
    }

    debug!("Processing with algorithm");  // DEBUG level (verbose)

    match validate(input) {
        Ok(data) => {
            info!("Validation successful");
            Ok(data)
        }
        Err(e) => {
            error!("Validation failed: {}", e);  // ERROR level
            Err(e)
        }
    }
}
```

## Step 3: Skip Sensitive Data

```rust
#[instrument(skip(password))]  // Don't log password
fn authenticate(username: &str, password: &str) -> Result<Token> {
    info!("Authenticating user: {}", username);  // OK to log username
    // password is never logged
    verify_password(username, password)
}
```

## Step 4: Add Custom Attributes

```rust
use tracing::field;

#[instrument(fields(user_id, request_id))]
fn handle_request(user_id: u64, request_id: &str) -> Result<Response> {
    info!(
        user_id = user_id,
        request_id = request_id,
        "Processing request"
    );
    Ok(Response::new())
}
```

## Step 5: Record Results

```rust
#[instrument]
fn compute(x: i32, y: i32) -> i32 {
    let result = x + y;
    info!(result = result, "Computation complete");
    result
}
```

## Step 6: Error Handling with Telemetry

```rust
#[instrument]
fn risky_operation() -> Result<String> {
    match dangerous_call() {
        Ok(data) => {
            info!("Operation succeeded");
            Ok(data)
        }
        Err(e) => {
            error!(error = %e, "Operation failed");
            Err(e)
        }
    }
}
```

## Step 7: Performance Tracking

```rust
use std::time::Instant;

#[instrument]
fn expensive_operation() -> Result<Output> {
    let start = Instant::now();

    let result = compute();

    let elapsed = start.elapsed();
    info!(duration_ms = elapsed.as_millis(), "Operation completed");

    result
}
```

## Step 8: Conditional Logging

```rust
#[instrument]
fn process(items: &[Item]) -> Result<()> {
    if items.len() > 1000 {
        warn!(count = items.len(), "Processing large dataset");
    }

    for item in items {
        debug!(item_id = item.id, "Processing item");
    }

    Ok(())
}
```

## Step 9: Structured Data

```rust
#[instrument]
fn record_event(event: &Event) -> Result<()> {
    info!(
        event_type = event.typ,
        timestamp = event.time,
        value = event.value,
        "Event recorded"
    );
    Ok(())
}
```

## Step 10: Integration Example

```rust
use tracing::{instrument, info, warn, error, debug};

#[instrument(skip(password))]
pub fn login(username: &str, password: &str) -> Result<Session> {
    info!(username = username, "Login attempt");

    // Validate input
    if username.is_empty() {
        warn!("Empty username");
        return Err("Invalid username".into());
    }

    // Check credentials
    match verify_credentials(username, password) {
        Ok(user_id) => {
            debug!(user_id = user_id, "Credentials verified");

            // Create session
            let session = Session::new(user_id);
            info!(session_id = %session.id, "Session created");

            Ok(session)
        }
        Err(e) => {
            warn!(username = username, "Authentication failed");
            error!(error = %e, "Login error");
            Err(e)
        }
    }
}
```

## Best Practices

✅ **DO:**
- Use appropriate log levels
- Skip sensitive data
- Record important attributes
- Log errors with context
- Track performance metrics
- Be consistent

❌ **DON'T:**
- Log passwords or secrets
- Log in hot paths (performance impact)
- Over-log (too much noise)
- Skip errors
- Use println! (use tracing instead)

## Log Levels

| Level | When to Use |
|-------|------------|
| **ERROR** | Something failed, user action needed |
| **WARN** | Something unexpected, may cause issues |
| **INFO** | Important events, high-level flow |
| **DEBUG** | Detailed information for debugging |
| **TRACE** | Very detailed, lowest level |

## Configuration

Set log level when running:

```bash
# Only errors
RUST_LOG=error cargo test

# Errors and warnings
RUST_LOG=warn cargo test

# All info and above
RUST_LOG=info cargo test

# Everything
RUST_LOG=debug cargo test
RUST_LOG=trace cargo test
```

## Module-Specific Logging

```bash
# Specific module
RUST_LOG=my_module=debug cargo test

# Multiple modules
RUST_LOG=my_module=debug,other_module=warn cargo test
```

## Common Telemetry Patterns

### Pattern: Function Entry/Exit
```rust
#[instrument]
fn my_function(param: &str) -> Result<String> {
    // Entry logged automatically
    info!("Processing: {}", param);

    let result = do_work(param)?;

    // Exit logged automatically
    Ok(result)
}
```

### Pattern: Loop with Progress
```rust
#[instrument]
fn process_items(items: &[Item]) -> Result<()> {
    for (idx, item) in items.iter().enumerate() {
        debug!(index = idx, item_id = item.id, "Processing item");
        process_one(item)?;
    }
    info!(total = items.len(), "All items processed");
    Ok(())
}
```

### Pattern: Conditional Warnings
```rust
#[instrument]
fn validate(data: &Data) -> Result<()> {
    if data.size > 100 * 1024 * 1024 {
        warn!(size_mb = data.size / 1024 / 1024, "Large dataset");
    }
    // ... validation
    Ok(())
}
```

## Testing Telemetry

Verify telemetry is emitted:

```bash
RUST_LOG=debug cargo test --lib my_test -- --nocapture
```

You should see your log messages in output.

## Next Steps

- **Create schemas**: [How to Create OTel Schemas](06-create-otel-schemas.md)
- **Fix validation**: [How to Fix Weaver Validation Errors](03-fix-weaver-validation-errors.md)
- **Learn more**: [Understanding Telemetry](../tutorials/02-understanding-telemetry.md)

## Key Commands

```bash
# Run with debug logging
RUST_LOG=debug cargo test --lib -- --nocapture

# Specific module
RUST_LOG=my_module=info cargo test

# See all telemetry
RUST_LOG=trace cargo test -- --nocapture
```

---

**Category**: How-to Guides (Task-oriented)
**Framework**: Diátaxis
**Difficulty**: Beginner
**Related**: Creating Schemas, Fixing Validation
