# How-to Guide 5: Create OTel Schemas

## Goal

Design, create, and validate OpenTelemetry schemas that define the telemetry your KNHK code will emit, enabling Weaver validation of runtime behavior.

**Time Estimate**: 2-3 hours
**Prerequisites**: [Understanding Telemetry](../tutorials/02-understanding-telemetry.md), [Run Tests Efficiently](02-run-tests-efficiently.md)
**Difficulty**: Intermediate
**Outcomes**: Complete, validated OTel schema for your feature

---

## What is an OTel Schema?

### The Problem Schemas Solve

```
Without Schema:
  Code emits: "span_name": "user_creation"
  Question: Is this what was supposed to happen?
  Answer: ??? (Nobody knows)

With Schema:
  Schema declares: "span_name": "user_creation" ✓
  Code emits: "span_name": "user_creation" ✓
  Weaver validates: MATCH ✓ Feature works!
```

### Why Schemas Matter in KNHK

- **Contract**: Defines what telemetry will be emitted
- **Validation**: Weaver ensures actual telemetry matches contract
- **Documentation**: Schema documents observability
- **Proof**: If schema validates, feature must work

### Schema as Source of Truth

```
Schema Definition
    ↓
Code implements to schema
    ↓
Code runs in production
    ↓
Actual telemetry emitted
    ↓
Weaver validates: schema matches telemetry?
    ↓
YES → Feature definitely works
NO  → Feature definitely doesn't work
```

---

## Part 1: Schema Fundamentals

### What Goes in a Schema?

```yaml
groups:
  my_feature:                    # Group name
    description: "..."           # What this group covers

    spans:                        # Spans this feature emits
      my_span_name:
        description: "..."
        attributes:
          attribute_name:
            type: string
            description: "..."
            required: true

    metrics:                      # Metrics this feature emits
      my_metric_name:
        type: counter | histogram | gauge
        description: "..."
        unit: "1" | "ms" | "bytes" | etc.

    logs:                         # Logs this feature emits
      my_log_level:
        description: "..."
        fields:
          field_name:
            type: string
```

### Types in Schemas

**Basic types:**
- `string` - Text values
- `int` - Integer numbers
- `float` - Decimal numbers
- `bool` - True/false
- `bytes` - Binary data

**Metric types:**
- `counter` - Always increases (e.g., requests total)
- `histogram` - Distribution of values (e.g., latencies)
- `gauge` - Current value (e.g., active connections)

**Span attributes:**
- Required: Must be present
- Optional: Can be present or absent
- Any type: string, int, float, etc.

---

## Part 2: Design Your Telemetry

### Step 1: Identify What to Measure

**Start with these questions:**

1. **What operations matter?**
   - API endpoints
   - Database queries
   - External service calls
   - State transitions

2. **What should we track?**
   - Success/failure
   - Duration
   - Resource IDs
   - User context
   - Error details

3. **What about edge cases?**
   - Timeouts
   - Retries
   - Fallbacks
   - Errors

### Step 2: Plan Spans

**For each operation, define:**

```markdown
# Operation: Create User

## Span Name
create_user

## When Emitted
When CreateUser API endpoint is called

## Attributes
- user_email (string, required)
- user_role (string, required)
- retry_count (int, optional)

## Child Spans
- validate_email (always)
- store_in_database (always)
- send_confirmation (if email validation passed)

## Status
- OK: User created successfully
- ERROR: User creation failed
```

### Step 3: Plan Metrics

**For each key measurement, define:**

```markdown
# Metric: User Creation Duration

## Name
user_creation_duration_ms

## Type
histogram (distribution of durations)

## What It Measures
How long user creation takes

## Unit
milliseconds (ms)

## When Emitted
After each successful user creation

## When NOT Emitted
When creation fails
```

### Step 4: Plan Logs

**For structured logging, define:**

```markdown
# Log: User Validation Error

## When Emitted
When email validation fails during registration

## Log Level
WARN

## Fields
- user_email (string)
- validation_error (string)
- error_code (string)

## Example
{
  "level": "WARN",
  "message": "Email validation failed",
  "user_email": "invalid@",
  "validation_error": "Missing domain",
  "error_code": "INVALID_EMAIL_FORMAT"
}
```

---

## Part 3: Write Your Schema

### Schema File Structure

**Location and naming:**
```
registry/
└── schemas/
    ├── user_operations.yaml       # Schema for user feature
    ├── payment_operations.yaml    # Schema for payment feature
    └── common.yaml                # Shared/common spans
```

### Example: Complete Schema

Create `registry/schemas/user_operations.yaml`:

```yaml
# OpenTelemetry Schema for User Operations
# Defines telemetry emitted by user management features

groups:
  user.operations:
    description: "User account management operations"

    spans:
      create_user:
        description: "Create a new user account"
        attributes:
          email:
            description: "User's email address"
            type: string
            required: true
          role:
            description: "User's role (user, admin, moderator)"
            type: string
            required: true
          source:
            description: "Registration source (web, mobile, api)"
            type: string
            required: false

      deactivate_user:
        description: "Deactivate an existing user account"
        attributes:
          user_id:
            description: "ID of user being deactivated"
            type: int
            required: true
          reason:
            description: "Reason for deactivation if provided"
            type: string
            required: false

      reactivate_user:
        description: "Reactivate a deactivated user account"
        attributes:
          user_id:
            description: "ID of user being reactivated"
            type: int
            required: true
          days_inactive:
            description: "Days the account was inactive"
            type: int
            required: false

      validate_email:
        description: "Validate email address format and uniqueness"
        attributes:
          email:
            description: "Email being validated"
            type: string
            required: true
          validation_type:
            description: "Type of validation (format, uniqueness, deliverability)"
            type: string
            required: true

      authenticate_user:
        description: "Authenticate user with credentials"
        attributes:
          user_id:
            description: "ID of user attempting authentication"
            type: int
            required: true
          method:
            description: "Auth method (password, mfa, oauth)"
            type: string
            required: true

    metrics:
      users_created_total:
        description: "Total number of user accounts created"
        type: counter
        unit: "1"

      user_creation_duration_ms:
        description: "Duration of user creation operation"
        type: histogram
        unit: "ms"

      user_deactivations_total:
        description: "Total user account deactivations"
        type: counter
        unit: "1"

      user_reactivations_total:
        description: "Total user account reactivations"
        type: counter
        unit: "1"

      authentication_attempts_total:
        description: "Total authentication attempts"
        type: counter
        unit: "1"

      authentication_failures_total:
        description: "Total failed authentication attempts"
        type: counter
        unit: "1"

      active_users:
        description: "Current number of active users"
        type: gauge
        unit: "1"

    logs:
      email_validation_failure:
        description: "Email validation failed"
        fields:
          email:
            description: "Email that failed validation"
            type: string
          reason:
            description: "Why validation failed"
            type: string
          error_code:
            description: "Validation error code"
            type: string

      authentication_failure:
        description: "User authentication failed"
        fields:
          user_id:
            description: "User attempting authentication"
            type: int
          reason:
            description: "Why authentication failed"
            type: string
          attempt_number:
            description: "Which attempt this was"
            type: int
```

---

## Part 4: Validate Your Schema

### Step 1: Basic Structure Validation

```bash
# Check if YAML is valid
weaver registry check -r registry/

# Should output:
# ✓ Schema structure is valid
# ✓ All required fields present
# ✓ No conflicts detected
```

### Step 2: Run Schema Checks

```bash
# Detailed validation report
weaver registry check -r registry/ --verbose

# Output shows:
# Checking registry/ ...
# ✓ user_operations.yaml
#   ✓ Groups: 1 (user.operations)
#   ✓ Spans: 5 defined
#   ✓ Metrics: 7 defined
#   ✓ Logs: 2 defined
```

### Step 3: Live Validation (Against Code)

```bash
# Run code and validate emitted telemetry matches schema
weaver registry live-check --registry registry/ --tracer=<endpoint>

# If Weaver is not installed yet, save for later
# (Covered in How-to: Fix Weaver Validation Errors)
```

---

## Part 5: Common Patterns

### Pattern 1: Request/Response Spans

```yaml
spans:
  http_request:
    description: "HTTP request handling"
    attributes:
      http.method:
        description: "HTTP method (GET, POST, etc.)"
        type: string
        required: true
      http.url:
        description: "Request URL"
        type: string
        required: true
      http.status_code:
        description: "HTTP response status"
        type: int
        required: true
      user_id:
        description: "ID of user making request"
        type: int
        required: false

metrics:
  http_requests_total:
    description: "Total HTTP requests"
    type: counter
    unit: "1"

  http_request_duration_ms:
    description: "HTTP request duration"
    type: histogram
    unit: "ms"
```

### Pattern 2: Database Operations

```yaml
spans:
  database_query:
    description: "Database query execution"
    attributes:
      db.statement:
        description: "SQL query (without sensitive data)"
        type: string
        required: true
      db.rows_affected:
        description: "Number of rows affected"
        type: int
        required: false
      db.duration_ms:
        description: "Query execution time"
        type: float
        required: true

metrics:
  db_queries_total:
    description: "Total database queries"
    type: counter
    unit: "1"

  db_query_duration_ms:
    description: "Database query duration"
    type: histogram
    unit: "ms"
```

### Pattern 3: Error Tracking

```yaml
spans:
  operation_with_error:
    description: "Operation that may fail"
    attributes:
      operation_name:
        description: "Name of operation"
        type: string
        required: true
      error_code:
        description: "Error code if failed"
        type: string
        required: false
      error_message:
        description: "Error message if failed"
        type: string
        required: false

metrics:
  operations_total:
    description: "Total operations attempted"
    type: counter
    unit: "1"

  operation_errors_total:
    description: "Total operation errors"
    type: counter
    unit: "1"

  operation_error_rate:
    description: "Error rate (errors / total)"
    type: gauge
    unit: "1"
```

### Pattern 4: Performance Tracking

```yaml
metrics:
  operation_duration_ms:
    description: "Operation duration"
    type: histogram
    unit: "ms"

  operation_p50_duration_ms:
    description: "50th percentile duration"
    type: gauge
    unit: "ms"

  operation_p95_duration_ms:
    description: "95th percentile duration"
    type: gauge
    unit: "ms"

  operation_p99_duration_ms:
    description: "99th percentile duration"
    type: gauge
    unit: "ms"
```

---

## Part 6: Schema Best Practices

### DO: Follow Naming Conventions

```yaml
# ✓ GOOD: Clear, descriptive names
spans:
  user_registration_process:
    attributes:
      email: {...}
      role: {...}

metrics:
  user_registrations_total: {...}
  user_registration_duration_ms: {...}

# ✗ BAD: Unclear or inconsistent names
spans:
  process: {...}           # Too generic
  ur: {...}                # Too abbreviated
  UserRegistrationProcess: {...}  # Inconsistent casing

metrics:
  count: {...}             # Which count?
  time: {...}              # Which time?
```

### DO: Document Everything

```yaml
# ✓ GOOD: Clear, helpful descriptions
spans:
  user_registration:
    description: "Register a new user account. Includes validation,
                 storage, and confirmation email sending."
    attributes:
      email:
        description: "User's email address. Must be valid format
                     and unique. Validated before storage."
        type: string
        required: true

# ✗ BAD: Vague or missing descriptions
spans:
  user_registration:
    # No description!
    attributes:
      email:
        type: string
        # No description!
```

### DO: Use Consistent Attribute Types

```yaml
# ✓ GOOD: Consistent types across spans
spans:
  user_created:
    attributes:
      user_id:
        type: int  # Consistent

  user_updated:
    attributes:
      user_id:
        type: int  # Same type

# ✗ BAD: Inconsistent types
spans:
  user_created:
    attributes:
      user_id:
        type: int

  user_updated:
    attributes:
      user_id:
        type: string  # Different!
```

### DON'T: Include Sensitive Data

```yaml
# ✗ BAD: Could leak sensitive data
spans:
  authenticate_user:
    attributes:
      password:        # ✗ Never log passwords
        type: string
      credit_card:     # ✗ Never log card numbers
        type: string
      api_key:         # ✗ Never log secrets
        type: string

# ✓ GOOD: Only non-sensitive identifiers
spans:
  authenticate_user:
    attributes:
      user_id:         # ✓ Safe identifier
        type: int
      auth_method:     # ✓ Non-sensitive detail
        type: string
```

### DON'T: Make Everything Required

```yaml
# ✗ BAD: Too many required fields
spans:
  process_payment:
    attributes:
      amount:
        type: float
        required: true
      currency:
        type: string
        required: true
      retry_count:
        type: int
        required: true  # Usually doesn't apply
      fallback_method:
        type: string
        required: true  # Not always used

# ✓ GOOD: Required only when actually required
spans:
  process_payment:
    attributes:
      amount:
        type: float
        required: true
      currency:
        type: string
        required: true
      retry_count:
        type: int
        required: false  # Only for retries
      fallback_method:
        type: string
        required: false  # Only if used
```

---

## Part 7: Integration with Code

### Code Should Match Schema

**Schema defines:**
```yaml
spans:
  user_creation:
    attributes:
      email:
        type: string
        required: true
      role:
        type: string
        required: true
```

**Code must emit:**
```rust
#[instrument]  // Creates span: user_creation
fn create_user(email: &str, role: &str) {  // email and role captured
    // ... implementation
}
```

**Weaver validates:**
```
Schema expects: email (string), role (string) ✓
Code emits: email = "alice@example.com", role = "user" ✓
MATCH ✓ Feature works!
```

### Handle Optional Attributes

**Schema defines:**
```yaml
spans:
  user_update:
    attributes:
      user_id:
        required: true
      updated_fields:
        required: false  # Optional
```

**Code can emit:**
```rust
if user_was_updated {
    // May or may not emit updated_fields
    info!(updated_fields = ?fields, "User updated");
}
```

---

## Part 8: Troubleshooting

### Issue: Schema Doesn't Validate

**Error**: `Schema validation failed: invalid type for attribute 'count'`

**Solution**:
```yaml
# Check YAML syntax
# Make sure attribute types match span usage

# Review error message for specific issue
# Common problems:
# - Typo in attribute name
# - Wrong type (int vs string)
# - Missing required field
# - Invalid metric type (must be counter/histogram/gauge)

# Fix and re-validate
weaver registry check -r registry/
```

### Issue: Code Doesn't Match Schema

**Error**: `Live check failed: span 'create_user' has unexpected attribute 'timestamp'`

**Solution**:
```bash
# Option 1: Add missing attribute to schema
# Edit registry/schemas/yourfeature.yaml
# Add: timestamp: {type: string, required: true}

# Option 2: Remove unexpected attribute from code
# Edit your instrumentation code
# Remove the timestamp attribute emission

# Re-validate
weaver registry live-check --registry registry/
```

### Issue: Too Many Attributes

**Error**: `Attribute count exceeds recommended limit (32)`

**Solution**:
```yaml
# Consolidate related attributes
# Instead of: email, first_name, last_name, age, ...
# Use: user_profile with nested attributes

# Or: Only include critical attributes
# Remove: nice-to-have attributes that add noise
```

### Issue: Unit Mismatch

**Error**: `Metric unit mismatch: schema says 'ms', code emits 's'`

**Solution**:
```yaml
# Option 1: Fix schema to match code
metrics:
  duration:
    unit: "s"  # Changed from "ms"

# Option 2: Convert code to match schema
# Code: histogram!("duration_ms", millis)
# Schema: unit: "ms"
```

---

## Part 9: Extending Schemas

### Adding to Existing Schema

```yaml
# In existing registry/schemas/user_operations.yaml
groups:
  user.operations:
    # Existing content ...

    spans:
      # Existing spans ...

      # NEW SPAN
      password_reset:
        description: "User password reset operation"
        attributes:
          user_id:
            type: int
            required: true
          reset_method:
            description: "How password reset was initiated"
            type: string
            required: true

    metrics:
      # Existing metrics ...

      # NEW METRIC
      password_reset_attempts_total:
        description: "Total password reset attempts"
        type: counter
        unit: "1"
```

### Creating New Group

```yaml
# Create new file: registry/schemas/payment_operations.yaml
groups:
  payment.operations:
    description: "Payment processing operations"

    spans:
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

    metrics:
      payments_processed_total:
        type: counter
        unit: "1"
      payment_duration_ms:
        type: histogram
        unit: "ms"
```

---

## Part 10: Quick Reference

### Span Definition Template

```yaml
spans:
  operation_name:
    description: "What this operation does"
    attributes:
      attribute_name:
        description: "What this attribute means"
        type: string | int | float | bool | bytes
        required: true | false
```

### Metric Definition Template

```yaml
metrics:
  metric_name:
    description: "What this metric measures"
    type: counter | histogram | gauge
    unit: "1" | "ms" | "bytes" | "%" | etc.
```

### Type Quick Reference

| Type | Example | Use For |
|------|---------|---------|
| `string` | "user@example.com" | Text values |
| `int` | 42 | Whole numbers |
| `float` | 3.14 | Decimals |
| `bool` | true/false | Yes/no values |
| `bytes` | [0x01, 0x02] | Binary data |

### Metric Type Quick Reference

| Type | Example | When to Use |
|------|---------|------------|
| `counter` | requests_total = 1000 | Always-increasing counts |
| `histogram` | latency_ms = [10, 20, 15, 18] | Distributions, percentiles |
| `gauge` | active_connections = 42 | Current values |

---

## Checklist: Before You're Done

- [ ] Schema file created in `registry/schemas/`
- [ ] All spans defined
- [ ] All metrics defined
- [ ] All attributes documented
- [ ] No required=true for optional attributes
- [ ] No sensitive data in schema
- [ ] Naming consistent throughout
- [ ] All descriptions clear and helpful
- [ ] Schema validates with `weaver registry check`
- [ ] Code matches schema definitions
- [ ] Live validation passes
- [ ] Documentation updated

---

## Next Steps

1. **Validate with Live Code**: [How-to: Fix Weaver Validation Errors](06-fix-weaver-validation-errors.md) (coming soon)
2. **Emit Telemetry**: [How-to: Emit Proper Telemetry](07-emit-proper-telemetry.md) (coming soon)
3. **Complete Features**: [How-to: Add New Features](04-add-new-features.md) (already created)

---

## Summary

### Schema Creation Process

1. **Design**: Identify what to measure
2. **Define**: Create YAML schema
3. **Validate**: Check structure with Weaver
4. **Implement**: Code matches schema
5. **Test**: Emit telemetry matching schema
6. **Verify**: Live validation passes

### Key Principle

**Schema is the contract** between what you declare will happen and what actually happens in production.

Weaver validates this contract—if validation passes, your feature works exactly as specified.

---

**Created**: 2025-11-15
**Status**: Complete
**Difficulty**: Intermediate
**Next**: [How-to: Fix Weaver Validation Errors](06-fix-weaver-validation-errors.md) (coming soon)
