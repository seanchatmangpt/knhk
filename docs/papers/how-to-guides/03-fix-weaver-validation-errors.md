# How-to Guide: Fix Weaver Validation Errors

**Goal**: Understand and resolve OpenTelemetry schema validation failures
**Time**: 15 minutes per error
**Difficulty**: Intermediate

## What is Weaver Validation?

Weaver validates that your code's telemetry matches its declared schema:

```
Your Code → Emits Telemetry → Weaver Checks Schema → ✅ or ❌
```

When Weaver fails, it means:
- Code emits telemetry not documented in schema, OR
- Schema documents telemetry that code doesn't emit, OR
- Telemetry attributes don't match schema definition

## Step 1: Understand the Error

Run Weaver validation:

```bash
weaver registry check -r registry/
```

**Success output**:
```
✓ Schema valid
✓ All components documented
```

**Failure output**:
```
✗ Error: Span 'my_span' not found in registry
  Location: src/lib.rs:42:5
```

## Common Error Types

### Error 1: Undocumented Span

**Error message**:
```
Error: Span 'process_data' not found in registry
```

**What it means**: Code emits span but schema doesn't document it.

**Solution**:

1. Find the offending code:
```bash
grep -n "process_data" src/**/*.rs
```

2. Option A - Add to schema:
```yaml
# Add to registry/my-component.yaml
spans:
  - name: process_data
    description: "Processes input data"
    attributes:
      - input_size: "Size of input in bytes"
      - operation: "Type of operation"
```

3. Option B - Remove from code:
```rust
// Remove the span
#[tracing::instrument]  // ← Remove this line
fn process_data(input: &str) {
    // ... function body
}
```

### Error 2: Undocumented Attribute

**Error message**:
```
Error: Attribute 'request_id' in span 'process_data' not documented
```

**What it means**: Span emits attribute not in schema.

**Solution**:

1. Find the attribute in code:
```bash
grep -n "request_id" src/**/*.rs
```

2. Add to schema:
```yaml
spans:
  - name: process_data
    attributes:
      - request_id: "Unique request identifier"  # ← Add this
      - input_size: "Size of input"
```

### Error 3: Metric Not Found

**Error message**:
```
Error: Metric 'operation_duration' not found in registry
```

**What it means**: Code records metric but schema doesn't document it.

**Solution**:

1. Find the metric:
```bash
grep -n "operation_duration" src/**/*.rs
```

2. Add to schema:
```yaml
metrics:
  - name: operation_duration
    type: histogram
    unit: "ns"
    description: "Duration of operations in nanoseconds"
```

### Error 4: Type Mismatch

**Error message**:
```
Error: Metric 'count' is documented as counter but code emits histogram
```

**What it means**: Metric type in code doesn't match schema.

**Solution**:

Fix the schema to match the code:
```yaml
metrics:
  - name: count
    type: histogram        # ← Change from counter
    description: "..."
```

Or fix the code to match schema:
```rust
// In code, use counter instead of histogram
meter
    .u64_counter("count")  // ← Use counter
    .add(1, &[])
```

## Step 2: Validate Specific Component

Check one component's schema:

```bash
weaver registry check -r registry/ --component my_component
```

## Step 3: Live Validation

Check actual running code:

```bash
weaver registry live-check --registry registry/
```

This connects to running instance and validates emitted telemetry.

**Setup**:
1. Start your application
2. Run the live check
3. It will verify actual telemetry

## Step 4: Common Patterns

### Pattern: Adding a New Span

When adding a traced function:

1. **Add instrumentation**:
```rust
#[tracing::instrument]
fn my_function(param: &str) -> Result<String> {
    info!("Starting process");
    // ... implementation
    Ok(result)
}
```

2. **Document in schema**:
```yaml
spans:
  - name: my_function
    description: "Processes the parameter"
    attributes:
      - param: "Input parameter"
    events:
      - "Starting process"
```

3. **Validate**:
```bash
weaver registry check -r registry/
```

### Pattern: Adding a Metric

When adding a metric:

1. **Add code**:
```rust
let meter = global::meter("my_app");
let counter = meter.u64_counter("operations")
    .init();

counter.add(1, &[]);
```

2. **Document in schema**:
```yaml
metrics:
  - name: operations
    type: counter
    unit: "1"
    description: "Number of operations performed"
```

3. **Validate**:
```bash
weaver registry check -r registry/
```

### Pattern: Adding an Event

When logging an event:

1. **Add code**:
```rust
warn!("unusual_behavior", reason = "threshold exceeded");
```

2. **Document in schema**:
```yaml
spans:
  - name: parent_span
    events:
      - name: unusual_behavior
        attributes:
          - reason: "Why this event occurred"
```

3. **Validate**:
```bash
weaver registry check -r registry/
```

## Step 5: Bulk Validation

Check multiple files:

```bash
# Validate all registry files
weaver registry check -r registry/ --verbose

# Show all issues
weaver registry check -r registry/ --strict
```

## Troubleshooting

### "Registry not found"
```bash
# Verify registry directory exists
ls -la registry/
```

### "Schema file invalid YAML"
```bash
# Check YAML syntax
cat registry/my-schema.yaml | python3 -c "import yaml; yaml.safe_load(__import__('sys').stdin)"
```

### "No registry found"
```bash
# Create basic registry structure
mkdir -p registry
touch registry/my-component.yaml
```

## Schema File Structure

Here's a complete example:

```yaml
# registry/my-component.yaml
instrumentation:
  name: my_component
  version: 1.0.0
  description: "My component telemetry"

spans:
  - name: process_data
    description: "Processes input data"
    attributes:
      - input_size: "Size of input in bytes"
      - operation_type: "Type of operation performed"
    events:
      - name: started
      - name: completed

metrics:
  - name: processing_duration
    type: histogram
    unit: "ns"
    description: "Duration of processing"
    attributes:
      - operation_type: "Type of operation"

logs:
  - name: process_info
    level: INFO
    description: "Processing information"
    attributes:
      - phase: "Current processing phase"
```

## Best Practices

✅ **DO:**
- Document all telemetry before emitting it
- Use consistent names for spans/metrics/logs
- Include descriptions for clarity
- Run validation before commit
- Run live validation before deploy

❌ **DON'T:**
- Emit undocumented telemetry
- Change telemetry without updating schema
- Use vague names like "event1", "metric1"
- Mix span/metric/log names
- Deploy with validation failures

## Validation Workflow

```
1. Write Code
   ↓
2. Add Telemetry (#[tracing::instrument])
   ↓
3. Update Schema (registry/my-component.yaml)
   ↓
4. Run: weaver registry check -r registry/
   ↓
5. If fails: Fix schema or code ↑
   ↓
6. If passes: Continue development
   ↓
7. Before merge: weaver registry live-check
```

## Integration with CI/CD

Add to your CI pipeline:

```bash
# In CI workflow
weaver registry check -r registry/ || exit 1
cargo test --workspace
make test-performance-v04
```

This ensures schema is always valid before merge.

## Next Steps

- **Create schema from scratch**: See [How to Create OTel Schemas](06-create-otel-schemas.md)
- **Emit proper telemetry**: See [How to Emit Proper Telemetry](05-emit-telemetry.md)
- **Optimize performance**: See [How to Optimize Performance](07-optimize-performance.md)

## Key Commands

```bash
# Validate schema
weaver registry check -r registry/

# Validate live telemetry
weaver registry live-check --registry registry/

# Check specific component
weaver registry check -r registry/ --component my_component

# Verbose output
weaver registry check -r registry/ --verbose
```

---

**Category**: How-to Guides (Task-oriented)
**Framework**: Diátaxis
**Difficulty**: Intermediate
**Related**: Creating Schemas, Emitting Telemetry, Performance
