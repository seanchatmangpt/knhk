# Tutorial: Understanding Telemetry in KNHK

**Level**: Intermediate
**Time**: 20-25 minutes
**Learning Objectives**: Learn how KNHK validates behavior through telemetry

## What You'll Learn

By the end of this tutorial, you'll understand:
- What OpenTelemetry (OTel) is and why it matters
- How KNHK uses telemetry for validation
- How Weaver schema validation works
- How to read telemetry output

## Prerequisites

- Completed: [Getting Started with KNHK](01-getting-started-with-knhk.md)
- KNHK repository cloned and built
- ~20 minutes of time

## The Problem KNHK Solves

### Traditional Testing

```
Test Case:
  assert(result == expected)  ✅ Test passes

But What Really Happened:
  ❓ Did the code actually do the work?
  ❓ Or did it take a shortcut?
  ❓ Or fake the result?

This is the "false positive" problem
```

### KNHK's Solution: Telemetry-First Validation

```
Telemetry Schema:
  "Function X should emit span 'process_data'"
  "Duration should be ≤8 ticks"
  "Should log INFO level events"

Runtime Execution:
  Function runs and emits telemetry

Weaver Validation:
  ✅ Schema matches actual behavior
  ✅ Behavior is validated, not just tested
```

## Step 1: Understand OpenTelemetry Concepts

OpenTelemetry provides three types of signals:

### 1. Spans (Traces)
Represent operations over time:
```rust
#[tracing::instrument]
fn process_data(input: &str) -> Result<String> {
    // This creates a span
    // When the function completes, the span closes
    // Duration is recorded
    Ok(input.to_uppercase())
}
```

**What gets recorded**:
- Function name
- Start time
- End time
- Duration (ticks/nanoseconds)
- Success or failure
- Any custom attributes

### 2. Metrics
Numerical measurements:
```rust
// Example metrics KNHK tracks:
// - Performance metrics (duration, ticks)
// - Operation counts
// - Error rates
// - Custom business metrics
```

### 3. Logs
Structured text messages:
```rust
use tracing::{info, warn, error};

info!("Processing started");           // Information level
warn!("High memory usage detected");    // Warning level
error!("Failed to process data");       // Error level
```

## Step 2: Review a Telemetry Schema

Let's look at how KNHK documents telemetry:

Navigate to the registry directory:
```bash
cd registry/
```

Review the schema files:
```bash
ls -la
```

You'll see `.yaml` files defining telemetry contracts.

Open one to see the structure:
```bash
cat your-component-schema.yaml
```

**Key sections**:
- `instrumentation` - Component being documented
- `attributes` - Data points collected
- `events` - What happens
- `metrics` - Performance measurements

## Step 3: Run Code with Telemetry

Let's actually see telemetry in action:

```bash
# Run a test with telemetry output
RUST_LOG=debug cargo test --lib one_test_name -- --nocapture
```

**What you'll see**:
- Trace events with timestamps
- Function names (spans)
- Log messages at different levels
- Duration information

## Step 4: Validate with Weaver

Now validate that telemetry matches the schema:

```bash
# Check the schema is valid
weaver registry check -r registry/

# Validate against live runtime
weaver registry live-check --registry registry/
```

**Expected output**:
```
✓ Schema valid
✓ All spans documented
✓ All metrics defined
✓ No undocumented telemetry
```

**If it fails**:
- Some code emits telemetry not in schema
- Or schema references non-existent telemetry
- Fix by updating either the schema or the code

## Step 5: Understand the Chatman Constant

The Chatman Constant (≤8 ticks) is a performance constraint:

### What is a "Tick"?

```
1 tick ≈ 1 CPU cycle (nanosecond on modern CPU)
8 ticks ≈ 8-10 nanoseconds
```

### Why 8 Ticks?

The "Chatman Equation" discovered that:
- Critical path operations should be minimal
- Fortune 500 systems need microsecond performance
- 8 ticks is the threshold for "hot path" operations

### How KNHK Enforces It

```bash
# Performance tests validate this
make test-performance-v04

# Each test verifies:
✓ Operation duration ≤ 8 ticks
✓ No unexpected blocking
✓ Memory efficient
```

## Step 6: See Telemetry in Action

Let's create a simple example:

1. Find a test file: `tests/example_test.rs`
2. Look for functions with `#[tracing::instrument]`
3. Run the test with logging:
```bash
RUST_LOG=trace cargo test your_test_name -- --nocapture --test-threads=1
```

**Observe**:
- Function entry (span start)
- Log statements
- Return values
- Function exit (span end)
- Total duration

## Step 7: Read Telemetry Output

When you see output like:

```
2025-11-15T10:30:45.123Z  INFO process_data: duration=456ns
    at src/lib.rs:42:5
    in knhk
```

**This means**:
- **Timestamp**: 2025-11-15T10:30:45.123Z (when it happened)
- **Level**: INFO (importance level)
- **Message**: "process_data: duration=456ns" (what happened)
- **Location**: src/lib.rs:42:5 (where in code)
- **Duration**: 456ns (how long it took)

## Step 8: Verify Schema Compliance

Let's verify code matches its schema:

### Step 8a: Find a Function
```bash
cd src
grep -n "tracing::instrument" *.rs
```

### Step 8b: Check the Schema
Look in `registry/` for the matching schema file.

### Step 8c: Validate
```bash
# Check that schema documents what the code does
weaver registry check -r registry/
```

## Step 9: The False Positive Problem Solved

Now understand why this matters:

### Traditional Testing (Can Have False Positives)
```
Test: assert(result == expected) ✅
Conclusion: "Code works!"
Reality: ❓ Code might be faking it
```

### KNHK Testing (Detects False Positives)
```
Schema: "Must emit span 'process_data' with duration ≤8 ticks"
Runtime: Code actually emits that span
Weaver: ✅ Validates match
Conclusion: "Code provably works"
```

## Step 10: Understand Validation Hierarchy

KNHK uses three levels of validation:

**Level 1: Weaver Schema Validation** (Most Important)
```bash
weaver registry check -r registry/
weaver registry live-check --registry registry/
```
✅ Proves actual runtime behavior matches specification

**Level 2: Compilation & Code Quality** (Baseline)
```bash
cargo build --release
cargo clippy --workspace -- -D warnings
```
✅ Ensures code is syntactically correct

**Level 3: Traditional Tests** (Supporting Evidence)
```bash
cargo test --workspace
make test-chicago-v04
make test-performance-v04
```
✅ Provides additional validation

**Key Point**: Weaver validation is the SOURCE OF TRUTH

## What You've Learned

Congratulations! You now understand:

1. **OpenTelemetry**: How code emits telemetry (spans, metrics, logs)
2. **Telemetry Validation**: How Weaver validates behavior against schema
3. **False Positives**: Why tests alone can't prove code works
4. **The Chatman Constant**: Why ≤8 tick performance matters
5. **KNHK's Innovation**: Schema validation eliminates false positives

## Key Insight

> "Tests can pass even when features are broken (false positives). KNHK uses OpenTelemetry schema validation to prove that code actually does what it claims."

## Next Steps

### Want to Dive Deeper?

1. **Read the Theory**
   - `docs/papers/explanation/formal-foundations.md`
   - `docs/papers/reference/the_chatman_equation_fortune5_v1.2.0.pdf`

2. **Learn Chicago TDD**
   - Next tutorial: Chicago TDD Basics

3. **Validate Your Code**
   - See: `docs/papers/how-to-guides/03-fix-weaver-validation-errors.md`

### Ready to Implement?

See the how-to guides for:
- How to Create OTel Schemas
- How to Emit Proper Telemetry
- How to Fix Weaver Validation Errors

## Key Commands to Remember

```bash
# Run code with telemetry output
RUST_LOG=debug cargo test your_test_name -- --nocapture

# Validate schema
weaver registry check -r registry/

# Validate runtime behavior
weaver registry live-check --registry registry/

# Check for undocumented telemetry
RUST_LOG=trace cargo test | grep -E "unregistered|unknown"
```

## Resources

- **OpenTelemetry Docs**: https://opentelemetry.io/docs/
- **Weaver Documentation**: https://opentelemetry.io/docs/specs/otel/protocol/exporter/
- **KNHK Papers**: docs/papers/reference/
- **Chatman Equation**: docs/papers/explanation/the_chatman_equation_fortune5.md

---

**You are here**: Tutorial (Learning-oriented)
**Framework**: Diátaxis
**Tutorial Duration**: ~20 minutes
**Difficulty**: Intermediate
**Prerequisites**: Getting Started with KNHK
