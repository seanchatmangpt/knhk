# How-to Guide 6: Fix Weaver Validation Errors

## Goal

Systematically diagnose and fix OpenTelemetry schema validation errors when Weaver validation fails, ensuring your telemetry matches schema declarations.

**Time Estimate**: 1.5-2 hours
**Prerequisites**: [Create OTel Schemas](05-create-otel-schemas.md), [Understanding Telemetry](../tutorials/02-understanding-telemetry.md)
**Difficulty**: Intermediate
**Outcomes**: All Weaver validation errors resolved, schema matching verified

---

## The Validation Problem

### What Weaver Does

```
Your Code
    ↓
Emits Telemetry (spans, metrics, logs)
    ↓
Weaver Validator
    ├─ Checks: Does emitted telemetry match schema?
    ├─ Validates: All attributes present and correct type?
    └─ Confirms: Nothing unexpected?
    ↓
Result:
  ✓ All match → Feature validated, works correctly
  ✗ Mismatch → Error, feature may not work as intended
```

### When Validation Fails

**Error message example**:
```
Schema validation failed in span 'create_user':
  - Missing required attribute 'email'
  - Attribute 'user_count' not defined in schema
  - Type mismatch: attribute 'age' is int but schema expects string
```

**What this means**:
- Your code emitted something the schema didn't expect
- OR your schema didn't define something the code emits
- OR types don't match

---

## Part 1: Understanding Error Types

### Error Type 1: Missing Required Attribute

**Error message**:
```
Schema validation failed: missing required attribute 'email' in span 'create_user'
```

**What it means**:
```
Schema says: email is required
Code emits:  (no email attribute)
Mismatch:    ✗ Code doesn't match schema
```

**Cause**:
- Forgot to emit attribute
- Attribute name wrong
- Conditional logic prevents emission

**Example**:
```yaml
# Schema says:
spans:
  create_user:
    attributes:
      email:
        required: true  # ← Must always be present
```

```rust
// Code forgot it:
#[instrument]  // ← Won't automatically capture email
fn create_user(email: &str) {
    // Need to explicitly add email to span
    span.add_attribute("email", email);
}
```

### Error Type 2: Unexpected Attribute

**Error message**:
```
Schema validation failed: unexpected attribute 'timestamp' in span 'create_user'
```

**What it means**:
```
Schema says: no 'timestamp' attribute
Code emits:  timestamp = "2025-11-15T10:30:00Z"
Mismatch:    ✗ Code emits something not in schema
```

**Cause**:
- Attribute not defined in schema
- Typo in attribute name
- Schema incomplete

**Example**:
```yaml
# Schema forgot to define it:
spans:
  create_user:
    attributes:
      email:
        type: string
      # Missing: timestamp!
```

```rust
// Code emits it:
info!(
    email = email,
    timestamp = Utc::now(),  // ← Not in schema!
    "User created"
);
```

### Error Type 3: Type Mismatch

**Error message**:
```
Schema validation failed: type mismatch for attribute 'user_id'
  - Schema expects: int
  - Code emits: string
```

**What it means**:
```
Schema says: user_id is int
Code emits:  user_id = "12345" (string)
Mismatch:    ✗ Types don't match
```

**Cause**:
- Type defined wrong in schema
- Code converts to wrong type
- String interpolation instead of number

**Example**:
```yaml
# Schema says int:
spans:
  user_action:
    attributes:
      user_id:
        type: int  # Expects number
```

```rust
// Code sends string:
info!(
    user_id = format!("{}", user_id),  // ← Converts to string!
    "User action"
);
```

### Error Type 4: Metric Type Mismatch

**Error message**:
```
Schema validation failed: metric 'requests_total' has wrong type
  - Schema expects: counter (always increases)
  - Code emits: histogram (values vary)
```

**What it means**:
```
Schema says: This is a counter (e.g., 1, 2, 3, 4...)
Code uses:   As a histogram (e.g., [10, 15, 12, 8])
Mismatch:    ✗ Using metric wrong
```

**Cause**:
- Chose wrong metric type in schema
- Using metric for wrong purpose

### Error Type 5: Missing Span Definition

**Error message**:
```
Schema validation failed: span 'process_payment' not found in schema
```

**What it means**:
```
Schema defines: (no 'process_payment' span)
Code emits:    Span: "process_payment"
Mismatch:      ✗ Span not declared in schema
```

**Cause**:
- Schema incomplete
- Added new span without updating schema
- Typo in span name

---

## Part 2: The Debugging Workflow

### Step 1: Run Validation to Get Error

```bash
# Get detailed error output
weaver registry live-check --registry registry/ --verbose

# Should show all validation failures
# Example output:
# ERROR: Schema validation failed
#   File: registry/schemas/user_operations.yaml
#   Group: user.operations
#   Issue 1: Missing required attribute 'email' in span 'create_user'
#   Issue 2: Unexpected attribute 'timestamp' in span 'create_user'
#   Issue 3: Type mismatch for 'user_id': expected int, got string
```

### Step 2: Read Error Message Carefully

**Parse the error:**
1. **What is failing?** (span name, metric name, etc.)
2. **What's the problem?** (missing, unexpected, type mismatch)
3. **What's expected vs actual?** (compare schema vs code)

**Example parsing**:
```
Error: "unexpected attribute 'timestamp' in span 'create_user'"

Parsing:
  ✓ Failing element: span named 'create_user'
  ✓ Problem type: unexpected attribute
  ✓ Attribute name: 'timestamp'
  ✓ Expected: no 'timestamp' in schema
  ✓ Actual: code is emitting 'timestamp'
```

### Step 3: Identify Root Cause

**Ask diagnostic questions:**

**For missing attributes**:
- Does schema really require it? (Check `required: true`)
- Is code emitting it? (Check instrumentation)
- Is it always present? (Check conditional logic)

**For unexpected attributes**:
- Is it in schema? (Check spelling)
- Should it be in schema? (Is it important?)
- Typo in code or schema? (Compare carefully)

**For type mismatches**:
- What type is schema expecting? (Check `.yaml`)
- What type is code emitting? (Check code)
- Are they compatible? (int vs float, ok; int vs string, not ok)

### Step 4: Decide How to Fix

**Two choices for each error**:

**Option A: Fix the Code**
```
Problem: Code emits more than schema allows
Solution: Remove extra emissions from code
When: Extra attributes not important

Code change:
  - Remove: info!(timestamp = now(), ...)
  + info!("User created")
```

**Option B: Fix the Schema**
```
Problem: Schema missing what code emits
Solution: Add missing definitions to schema
When: Attributes are important

Schema change:
  - Add: timestamp: {type: string, required: false}
```

### Step 5: Apply the Fix

See specific solutions below for each error type.

---

## Part 3: How to Fix Each Error Type

### Fix Missing Required Attribute

**Error**:
```
missing required attribute 'email' in span 'create_user'
```

**Step 1: Check if it's actually required**

```yaml
# registry/schemas/user_operations.yaml
spans:
  create_user:
    attributes:
      email:
        required: true  # ← Yes, it's required
```

**Step 2: Check if code emits it**

```bash
# Search your code:
grep -r "email" src/

# Should find something like:
# info!(email = email, "User created")
```

**Step 3: If code doesn't emit it, add it**

```rust
// BEFORE (broken):
#[instrument]
fn create_user(email: &str, password: &str) {
    // Function captures all args, but not automatically in span
    // ...
}

// AFTER (fixed):
#[instrument(fields(email = %email))]  // Explicitly add to span
fn create_user(email: &str, password: &str) {
    // Now 'email' is captured in the span
    // ...
}
```

Or manually:

```rust
fn create_user(email: &str) {
    info!(email = email, "Creating user");  // Explicit
    // ...
}
```

**Step 4: Verify it works**

```bash
# Run validation again
weaver registry live-check --registry registry/

# Should now pass this check
```

---

### Fix Unexpected Attribute

**Error**:
```
unexpected attribute 'timestamp' in span 'create_user'
```

**Step 1: Decide: is 'timestamp' important?**

**If NO (not important)**:
- Remove from code
- Go to Step 3

**If YES (important)**:
- Add to schema
- Go to Step 2

**Step 2: If important, add to schema**

```yaml
# registry/schemas/user_operations.yaml
spans:
  create_user:
    attributes:
      email:
        type: string
        required: true
      # ADD THIS:
      timestamp:
        description: "Time user was created"
        type: string
        required: false  # Important but optional
```

**Step 3: If not important, remove from code**

```rust
// BEFORE (has timestamp):
fn create_user(email: &str) {
    info!(
        email = email,
        timestamp = Utc::now(),  // ← Extra
        "User created"
    );
}

// AFTER (removed):
fn create_user(email: &str) {
    info!(
        email = email,
        "User created"
    );
}
```

**Step 4: Verify**

```bash
weaver registry live-check --registry registry/
```

---

### Fix Type Mismatch

**Error**:
```
type mismatch for attribute 'user_id'
  - Schema expects: int
  - Code emits: string
```

**Step 1: Check schema**

```yaml
# registry/schemas/user_operations.yaml
spans:
  create_user:
    attributes:
      user_id:
        type: int  # ← Expects integer
```

**Step 2: Check code**

```rust
// What type is being emitted?
fn create_user(user_id: i64) {
    info!(
        user_id = format!("{}", user_id),  // ← String! Type mismatch
        "User created"
    );
}
```

**Step 3: Fix by matching types**

**Option A: Keep schema, fix code to emit int**

```rust
// BEFORE:
info!(user_id = format!("{}", user_id), "Created");  // String

// AFTER:
info!(user_id = user_id as i32, "Created");  // Integer
```

**Option B: Keep code, update schema type**

```yaml
# BEFORE:
user_id:
  type: int

# AFTER:
user_id:
  type: string
```

**Recommendation**: Option A (match code to schema)

**Step 4: Verify**

```bash
weaver registry live-check --registry registry/
```

---

### Fix Missing Span Definition

**Error**:
```
span 'process_payment' not found in schema
```

**Step 1: Check if span should exist**

**If NO (shouldn't emit)**:
- Remove span from code
- Go to Step 3

**If YES (should emit)**:
- Add to schema
- Go to Step 2

**Step 2: Add to schema**

```yaml
# registry/schemas/payment_operations.yaml
spans:
  # ADD NEW SPAN:
  process_payment:
    description: "Process a payment transaction"
    attributes:
      amount:
        type: float
        required: true
      currency:
        type: string
        required: true
      payment_method:
        type: string
        required: true
```

**Step 3: Remove from code (if shouldn't exist)**

```rust
// BEFORE (shouldn't emit):
fn process_payment(amount: f64) {
    info!(amount = amount, "Payment processed");  // ← Remove
}

// AFTER (no telemetry):
fn process_payment(amount: f64) {
    // Silent operation
}
```

**Step 4: Verify**

```bash
weaver registry check -r registry/       # Check schema validity
weaver registry live-check --registry registry/  # Check live
```

---

### Fix Metric Type Issue

**Error**:
```
metric 'requests_total' has wrong type
  - Schema: counter
  - Code: histogram
```

**Step 1: Understand metric types**

```
counter:    Always increases (0 → 1 → 2 → 3 → ...)
histogram:  Values vary (5, 10, 15, 3, 8, ...)
gauge:      Current value (42, 50, 100, ...)
```

**Step 2: Check what you're measuring**

```
requests_total: Total requests ever? → counter
latency_ms: How long do requests take? → histogram
active_users: How many now? → gauge
```

**Step 3: Fix to match**

**If schema is right, change code:**
```rust
// WRONG:
histogram!("requests_total", count);  // histogram

// RIGHT:
counter!("requests_total", 1);  // counter, increment by 1
```

**If code is right, change schema:**
```yaml
# WRONG:
requests_total:
  type: counter

# RIGHT:
request_duration_ms:
  type: histogram
```

---

## Part 4: Common Scenarios

### Scenario 1: Added New Feature, Schema Incomplete

**Problem**:
```
✗ Error: span 'order_checkout' not found in schema
✗ Error: missing required attribute 'cart_id'
✗ Error: metric 'checkout_duration_ms' not in schema
```

**Root cause**: Added feature without updating schema

**Solution**:
1. Add span to schema:
```yaml
order_checkout:
  description: "Process order checkout"
  attributes:
    cart_id:
      type: string
      required: true
    total_amount:
      type: float
      required: true
```

2. Add metrics to schema:
```yaml
checkout_duration_ms:
  type: histogram
  unit: "ms"
```

3. Re-validate:
```bash
weaver registry check -r registry/
weaver registry live-check --registry registry/
```

---

### Scenario 2: Refactored Code, Over-Emitting Telemetry

**Problem**:
```
✗ Error: unexpected attribute 'internal_step_id'
✗ Error: unexpected attribute 'debug_flag'
✗ Error: unexpected attribute 'retry_attempt'
```

**Root cause**: Emitting debug information that schema doesn't expect

**Solution**: Remove unnecessary emissions

```rust
// BEFORE (too verbose):
info!(
    user_id = user_id,
    internal_step_id = step_id,      // ✗ Extra
    debug_flag = is_debug,             // ✗ Extra
    retry_attempt = retry_count,       // ✗ Extra
    "Processing order"
);

// AFTER (clean):
info!(
    user_id = user_id,
    "Processing order"
);
```

---

### Scenario 3: Type Evolution

**Problem**:
```
✗ Error: type mismatch for 'user_id'
  - Schema: int
  - Code: string (UUID format)
```

**Root cause**: Changed ID format from int to UUID

**Solution**: Update schema

```yaml
# BEFORE:
user_id:
  type: int

# AFTER:
user_id:
  type: string
  description: "UUID format user identifier"
```

---

### Scenario 4: Optional vs Required Confusion

**Problem**:
```
✗ Error: missing required attribute 'user_id'
(But code sometimes doesn't have user_id)
```

**Root cause**: Attribute is optional but marked required

**Solution**: Update schema to optional

```yaml
# BEFORE:
user_id:
  type: int
  required: true  # But code doesn't always emit it

# AFTER:
user_id:
  type: int
  required: false  # Now code can omit it
```

---

## Part 5: Validation Workflow

### Complete Debugging Checklist

- [ ] Run validation and get full error report
- [ ] Read error message completely
- [ ] Identify error type (missing, unexpected, type mismatch, etc.)
- [ ] Determine root cause (schema incomplete vs code wrong)
- [ ] Decide fix approach (fix code or fix schema)
- [ ] Apply fix
- [ ] Re-validate
- [ ] Confirm all errors resolved

### Validation Command Reference

```bash
# Basic validation
weaver registry check -r registry/

# Detailed validation with verbose output
weaver registry check -r registry/ --verbose

# Live validation against running code
weaver registry live-check --registry registry/

# Check specific schema file
weaver registry check registry/schemas/user_operations.yaml

# Show all validation warnings (not just errors)
weaver registry check -r registry/ --warnings
```

---

## Part 6: Prevention Strategies

### Strategy 1: Schema-First Development

**Process**:
1. Write schema FIRST
2. Validate schema
3. Write code to match schema
4. Code will emit exactly what schema expects

**Benefits**:
- Prevent mismatches
- Clearer design
- Easier testing

### Strategy 2: Incremental Validation

```bash
# After each code change, validate
cargo build
cargo test
weaver registry live-check --registry registry/

# Catch errors early
```

### Strategy 3: Review Checklist

Before committing:

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Schema is defined for new telemetry
- [ ] Schema validates: `weaver registry check`
- [ ] Live validation passes: `weaver registry live-check`
- [ ] No false positives

### Strategy 4: Code Review

**Reviewer checklist**:
- [ ] New spans defined in schema
- [ ] All attributes documented
- [ ] Types match between code and schema
- [ ] No unexpected telemetry emissions
- [ ] Schema validates

---

## Part 7: Troubleshooting Weaver Itself

### Issue: Weaver Not Installed

**Error**: `weaver: command not found`

**Solution**:
```bash
# Install Weaver
# (Installation varies by system, check KNHK docs)

# Verify installation
weaver --version
```

### Issue: Registry Not Found

**Error**: `Error: registry directory not found: registry/`

**Solution**:
```bash
# Make sure you're in the right directory
pwd
# Should show knhk/

# Check registry exists
ls -la registry/

# If missing, check path in command
weaver registry check -r registry/  # Correct
weaver registry check registry/      # Wrong (missing -r)
```

### Issue: Invalid YAML in Schema

**Error**: `Error: YAML syntax error in registry/schemas/user.yaml`

**Solution**:
```bash
# Check YAML syntax online or locally
# Common issues:
# - Wrong indentation (must be spaces, not tabs)
# - Missing colons
# - Invalid characters
# - Unclosed quotes

# Fix YAML and re-validate
weaver registry check -r registry/
```

---

## Part 8: Quick Reference

### Error Type Decision Tree

```
Weaver validation failed

├─ "missing required attribute"
│  └─ Does code emit it?
│     ├─ No → Add to code
│     └─ Yes → Check schema has required: true

├─ "unexpected attribute"
│  └─ Is it important?
│     ├─ Yes → Add to schema
│     └─ No → Remove from code

├─ "type mismatch"
│  └─ Is schema or code wrong?
│     ├─ Schema → Update schema type
│     └─ Code → Convert code to correct type

├─ "span/metric not found in schema"
│  └─ Should it exist?
│     ├─ Yes → Add to schema
│     └─ No → Remove from code

└─ Other error
   └─ Read error message carefully and identify pattern
```

---

## Summary

### Fix Process

1. **Run validation** → Get error report
2. **Read carefully** → Understand what's wrong
3. **Identify cause** → Schema or code?
4. **Make decision** → Fix which?
5. **Apply fix** → Update schema or code
6. **Re-validate** → Confirm fix works
7. **Repeat** → Until all errors gone

### Key Principle

**Schema and code must match exactly**

If Weaver validation passes, your feature works exactly as declared in the schema.

---

## Next Steps

1. **Run Weaver validation**: Execute live checks
2. **Fix any errors**: Use this guide
3. **Verify complete**: All errors resolved
4. **Move forward**: Ready for deployment

---

**Created**: 2025-11-15
**Status**: Complete
**Difficulty**: Intermediate
**Uses**: [Create OTel Schemas](05-create-otel-schemas.md), [Understanding Telemetry](../tutorials/02-understanding-telemetry.md)
