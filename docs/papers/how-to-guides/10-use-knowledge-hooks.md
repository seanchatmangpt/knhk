# How-to Guide 10: Use Knowledge Hooks

## Goal

Master Knowledge Hooks (K-hooks) to create self-documenting code, implement workflow automation, and enable machine-readable task representations in KNHK projects.

**Time Estimate**: 2 hours
**Prerequisites**: [Add New Features](04-add-new-features.md), [Emit Proper Telemetry](07-emit-proper-telemetry.md)
**Difficulty**: Intermediate-Advanced
**Outcomes**: Code enhanced with strategic K-hooks that enable automation

---

## What Are Knowledge Hooks?

### The Core Concept

**Knowledge Hooks (K-hooks)** are structured code annotations that capture domain knowledge at critical points in your codebase, making it machine-readable and actionable.

```
Traditional Code:
  // Some calculation happens here
  result = complex_math()

K-hook Enhanced:
  @knowledge.hook({
    domain: "calculation",
    step: "apply_pricing_formula",
    version: "v2.1",
    inputs: ["quantity", "discount_tier"],
    outputs: ["price"],
    rules: [
      "Tier 1: 0-10 units, no discount",
      "Tier 2: 11-50 units, 5% discount",
      "Tier 3: 50+ units, 10% discount"
    ]
  })
  result = complex_math()
```

### Why K-hooks Matter

K-hooks enable:

1. **Machine Reading**: AI agents can understand code without reading implementation
2. **Automation**: Tools can navigate codebases intelligently
3. **Documentation**: Code documents itself with domain knowledge
4. **Testing**: Rules become testable assertions
5. **Compliance**: Business rules become auditable

### K-hook vs Comments

| Aspect | Comment | K-hook |
|--------|---------|--------|
| **Readability** | Human-focused | Machine-focused |
| **Machine-usable** | No | Yes |
| **Queryable** | No | Yes |
| **Actionable** | For humans | For agents/tools |
| **Structured** | Unstructured text | Structured data |

---

## Part 1: K-hook Anatomy

### Structure

Every K-hook has this structure:

```rust
@knowledge.hook({
  // Metadata (identifies the hook)
  id: "hook_unique_id",
  domain: "business_domain",
  version: "1.0",

  // Description (explains the hook)
  description: "What this knowledge is about",

  // Structure (defines inputs/outputs)
  inputs: ["input1", "input2"],
  outputs: ["output1"],

  // Knowledge (the actual business logic)
  rules: [
    "Rule 1 description",
    "Rule 2 description"
  ],

  // Integration (how to use it)
  related_spans: ["span_name"],
  related_tests: ["test_function_name"]
})
```

### Full Example

```rust
@knowledge.hook({
  id: "order_pricing_calculation",
  domain: "ecommerce.pricing",
  version: "2.1",

  description: "Calculates order total with quantity-based discounts",

  inputs: ["item_price", "quantity", "region"],
  outputs: ["final_price", "discount_applied"],

  rules: [
    "Base price = item_price × quantity",
    "If quantity >= 100: apply 10% discount",
    "If quantity >= 50: apply 5% discount",
    "If quantity >= 10: apply 2% discount",
    "If region == 'premium': add 10% for premium service",
    "Tax calculated after discount (region-specific rate)"
  ],

  thresholds: {
    min_order: 10,
    max_discount: 0.15
  },

  related_spans: [
    "calculate_pricing",
    "apply_discount",
    "calculate_tax"
  ],

  related_tests: [
    "test_pricing_with_quantity_discount",
    "test_premium_region_surcharge"
  ]
})
fn calculate_order_total(
  item_price: f64,
  quantity: i32,
  region: &str
) -> (f64, f64) {
  // Implementation
}
```

---

## Part 2: Common K-hook Patterns

### Pattern 1: Business Rule Hook

Use when you have important business logic:

```rust
@knowledge.hook({
  domain: "payment.processing",
  id: "payment_verification_rules",
  description: "Rules for detecting fraudulent transactions",

  rules: [
    "Flag transaction if amount > $10,000",
    "Flag if multiple transactions in 1 minute",
    "Flag if from new geographic region within 48 hours",
    "Flag if payment method changed unexpectedly"
  ],

  thresholds: {
    amount_limit: 10000.0,
    velocity_limit_minutes: 1,
    geographic_change_hours: 48
  }
})
fn detect_fraud(transaction: &Transaction) -> bool {
  // Implementation
}
```

### Pattern 2: Workflow Step Hook

Use when documenting workflow steps:

```rust
@knowledge.hook({
  domain: "order_fulfillment",
  id: "order_processing_workflow",
  description: "Multi-step order processing workflow",

  steps: [
    {
      number: 1,
      name: "validate_payment",
      timeout_ms: 5000,
      retry_count: 3
    },
    {
      number: 2,
      name: "reserve_inventory",
      timeout_ms: 10000,
      retry_count: 2
    },
    {
      number: 3,
      name: "create_shipment",
      timeout_ms: 15000,
      retry_count: 1
    }
  ],

  error_handling: "Rollback inventory on shipment failure",

  related_spans: [
    "order_validation",
    "inventory_reservation",
    "shipment_creation"
  ]
})
fn process_order(order: &Order) -> Result<()> {
  // Implementation
}
```

### Pattern 3: Constraint Hook

Use when documenting constraints and limits:

```rust
@knowledge.hook({
  domain: "performance.constraints",
  id: "api_endpoint_constraints",
  description: "Performance and resource constraints for API endpoint",

  constraints: [
    "Max response time: 8 ticks (Chatman Constant)",
    "Max concurrent requests: 10,000",
    "Max request body size: 1MB",
    "Max result set: 1000 items"
  ],

  thresholds: {
    max_ticks: 8,
    max_concurrent: 10000,
    max_body_bytes: 1_000_000,
    max_results: 1000
  },

  related_tests: [
    "test_api_response_time",
    "test_api_load_1000_concurrent"
  ]
})
#[instrument]
fn get_user_data(user_id: u64) -> Result<UserData> {
  // Must complete in ≤8 ticks
}
```

### Pattern 4: Data Transformation Hook

Use for complex data transformations:

```rust
@knowledge.hook({
  domain: "data.transformation",
  id: "customer_to_analytics_transform",
  description: "Transform customer data to analytics format",

  source_schema: {
    user_id: "string",
    email: "string",
    created_at: "timestamp",
    last_login: "timestamp?"
  },

  target_schema: {
    user_id_hash: "sha256",
    email_domain: "string",
    account_age_days: "integer",
    is_active: "boolean"
  },

  transformations: [
    "Hash user_id with SHA-256",
    "Extract domain from email",
    "Calculate days since creation",
    "Is active if last_login < 30 days"
  ]
})
fn transform_customer_for_analytics(customer: &Customer) -> AnalyticsRecord {
  // Implementation
}
```

### Pattern 5: Decision Tree Hook

Use for complex conditional logic:

```rust
@knowledge.hook({
  domain: "user.access_control",
  id: "permission_evaluation",
  description: "Hierarchical permission evaluation logic",

  decision_tree: {
    "Is user admin?": {
      yes: "Grant access",
      no: "Continue..."
    },
    "Is user team owner?": {
      yes: "Grant write access",
      no: "Continue..."
    },
    "Is user team member?": {
      yes: "Grant read access",
      no: "Deny access"
    }
  },

  rules: [
    "Admin bypasses all checks",
    "Team owner can read/write team data",
    "Team member can read team data only",
    "Non-member has no access"
  ]
})
fn can_user_access(user: &User, resource: &Resource) -> bool {
  // Implementation
}
```

---

## Part 3: K-hook Usage Patterns

### When to Use K-hooks

**Use K-hooks for:**
- ✓ Business logic and rules
- ✓ Performance-critical paths
- ✓ Security-sensitive operations
- ✓ Complex workflows
- ✓ Data transformations
- ✓ Integration points
- ✓ Constraint documentation

**Don't use K-hooks for:**
- ✗ Simple utility functions
- ✗ Generic helper code
- ✗ Obvious implementations
- ✗ Temporary debugging code

### Strategic K-hook Placement

```rust
// 🎯 LEVEL 1: Entry points (always hook)
#[knowledge.hook(domain: "api", ...)]
pub async fn handle_user_request(request: Request) -> Response {
  // Entry point - ALWAYS hook
}

  // 🎯 LEVEL 2: Major steps (hook if business-critical)
  #[knowledge.hook(domain: "validation", ...)]
  fn validate_input(input: &UserInput) -> Result<()> {
    // Business-critical validation
  }

    // 🎯 LEVEL 3: Helper functions (hook if reused)
    #[knowledge.hook(domain: "calculation", ...)]
    fn calculate_discount(amount: f64, tier: u32) -> f64 {
      // Reused calculation
    }

      // ✗ LEVEL 4: Implementation details (don't hook)
      fn format_number(n: f64) -> String {
        // Too low-level
      }
```

---

## Part 4: Querying K-hooks Programmatically

### Extract K-hooks from Code

Tools can parse K-hooks to understand code:

```bash
# Example: List all business rules
knhk k-hooks query --domain "payment.*" --type "rule"
# Output:
# payment.processing:fraud_detection
# payment.processing:refund_logic
# payment.validation:amount_limits

# Example: Find all performance-constrained functions
knhk k-hooks query --type "constraint" --constraint "max_ticks <= 8"
# Output:
# api.endpoints:get_user_data (≤8 ticks)
# api.endpoints:create_order (≤8 ticks)

# Example: Trace related tests
knhk k-hooks query --id "order_pricing" --show-tests
# Output:
# test_pricing_with_quantity_discount
# test_premium_region_surcharge
# test_pricing_edge_cases
```

### Generate Documentation from K-hooks

```bash
# Generate business rules documentation
knhk k-hooks generate-docs --domain "ecommerce.*" --output rules.md

# Output: rules.md
# # Payment Processing Rules
# ## Fraud Detection Rules
# - Flag transaction if amount > $10,000
# - Flag if multiple transactions in 1 minute
# ...

# Generate test coverage report
knhk k-hooks generate-coverage --show-untested
# Output:
# ✓ order_processing (5 tests)
# ✗ refund_logic (0 tests) ← Missing!
```

### Use K-hooks for Agent Navigation

```rust
// Agent can use K-hooks to navigate code intelligently
let hooks = KHookRegistry::query()
  .domain("order_fulfillment")
  .with_related_tests();

for hook in hooks {
  println!("Function: {}", hook.target_function);
  println!("Tests: {:?}", hook.related_tests);
  println!("Rules: {:?}", hook.rules);

  // Agent can now understand what tests to write!
}
```

---

## Part 5: K-hooks in Testing

### Link Tests to K-hooks

```rust
@knowledge.hook({
  id: "discount_calculation",
  domain: "pricing",

  rules: [
    "10% discount for 100+ items",
    "5% discount for 50-99 items",
    "No discount for <50 items"
  ],

  // Link to tests that verify these rules
  related_tests: [
    "test_discount_10_percent_100_items",
    "test_discount_5_percent_50_items",
    "test_no_discount_under_50"
  ]
})
fn calculate_discount(quantity: i32) -> f64 {
  // Implementation
}

// Tests are then discoverable from the hook
#[test]
fn test_discount_10_percent_100_items() {
  // Verifies: 10% discount for 100+ items
  assert_eq!(calculate_discount(100), 0.10);
}

#[test]
fn test_discount_5_percent_50_items() {
  // Verifies: 5% discount for 50-99 items
  assert_eq!(calculate_discount(50), 0.05);
}

#[test]
fn test_no_discount_under_50() {
  // Verifies: No discount for <50 items
  assert_eq!(calculate_discount(49), 0.00);
}
```

### Generate Test Completeness Reports

```bash
# Check test coverage of all K-hooks
knhk k-hooks validate-test-coverage
# Output:
# ✓ order_processing (3 tests, 100% coverage)
# ⚠ discount_calculation (3 tests, covers only 2 rules)
# ✗ fraud_detection (0 tests, 0% coverage)

# Generate missing test template
knhk k-hooks generate-test-template --hook fraud_detection
# Output: template.rs
#
# #[test]
# fn test_fraud_flag_high_amount() {
//   // Verify: Flag transaction if amount > $10,000
//   assert!(detect_fraud(&high_amount_transaction));
// }
```

---

## Part 6: K-hooks and Telemetry

### Link K-hooks to Telemetry Spans

```rust
@knowledge.hook({
  id: "order_fulfillment_flow",
  domain: "order.fulfillment",

  steps: [
    "Validate payment",
    "Reserve inventory",
    "Create shipment"
  ],

  // Link to telemetry spans
  related_spans: [
    "order_validation",
    "inventory_reservation",
    "shipment_creation"
  ]
})
#[instrument]  // Creates spans matching K-hook
fn fulfill_order(order: &Order) -> Result<()> {
  // Each step has corresponding span
  validate_payment()?;    // Creates: order_validation span
  reserve_inventory()?;   // Creates: inventory_reservation span
  create_shipment()?;     // Creates: shipment_creation span
  Ok(())
}
```

### Validate Telemetry Matches K-hooks

```bash
# Check that emitted telemetry matches K-hook declarations
knhk k-hooks validate-telemetry
# Output:
# ✓ order_validation span found
# ✓ inventory_reservation span found
# ✗ missing_step span declared but never emitted!
# ✗ unexpected_span generated but not in K-hook
```

---

## Part 7: Best Practices

### 1. Keep K-hooks Concise

**❌ WRONG: Over-detailed**
```rust
@knowledge.hook({
  description: "This function calculates the discount based on the quantity
               purchased. The discount is calculated as follows: if the customer
               purchases between 10 and 50 items, they get a 5% discount. If they
               purchase more than 50 items, they get a 10% discount. We need to
               make sure to apply the discount correctly..."
})
```

**✅ CORRECT: Clear and focused**
```rust
@knowledge.hook({
  description: "Calculate volume-based discount",
  rules: [
    "10-50 items: 5% discount",
    ">50 items: 10% discount"
  ]
})
```

### 2. Version Your K-hooks

```rust
@knowledge.hook({
  id: "discount_calculation",
  version: "2.1",  // Track changes

  changelog: [
    "v2.1: Added premium tier 15% discount",
    "v2.0: Changed calculation to percentage-based",
    "v1.0: Initial flat-rate discount"
  ]
})
fn calculate_discount(quantity: i32, tier: &str) -> f64 { }
```

### 3. Link Related Artifacts

```rust
@knowledge.hook({
  id: "order_processing",

  // Link to documentation
  docs_url: "https://docs.knhk.io/order-processing",

  // Link to tests
  related_tests: [
    "test_order_validation",
    "test_order_fulfillment"
  ],

  // Link to spans
  related_spans: [
    "order_validation",
    "order_fulfillment"
  ],

  // Link to rules/standards
  standards: [
    "ISO_8601 for dates",
    "RFC_5322 for emails"
  ]
})
```

### 4. Use Structured Data

```rust
@knowledge.hook({
  id: "payment_processing",

  // ❌ Unstructured
  // rules: "Process payment, check fraud, apply discount"

  // ✅ Structured
  rules: [
    "Process payment",
    "Check fraud",
    "Apply discount"
  ],

  // ❌ Ambiguous thresholds
  // constraints: "Must be fast, cannot be >1MB"

  // ✅ Precise thresholds
  constraints: {
    max_ticks: 8,
    max_bytes: 1_000_000
  }
})
```

---

## Part 8: Step-by-Step: Adding K-hooks to Existing Code

### Step 1: Identify Key Functions

```rust
// Look for functions with:
// - Complex business logic
// - Multiple rules
// - Critical workflows
// - Performance constraints

pub fn process_order(order: &Order) -> Result<()> {
  // ← Candidate for K-hook (complex workflow)
}

pub fn format_date(date: &Date) -> String {
  // ← Don't hook (simple utility)
}
```

### Step 2: Extract Business Knowledge

```rust
// Before: Business logic scattered in code
fn process_order(order: &Order) -> Result<()> {
  // Check if customer is premium
  if order.customer.tier == "premium" {
    // Prime gets 2-day shipping
    order.shipping_days = 2;
  } else if order.customer.tier == "gold" {
    // Gold gets 3-day shipping
    order.shipping_days = 3;
  } else {
    // Standard gets 5-day shipping
    order.shipping_days = 5;
  }

  // Check total amount for discount
  if order.total > 1000.0 {
    order.discount = 0.15;
  } else if order.total > 500.0 {
    order.discount = 0.10;
  }
}

// After: Business logic in K-hook
@knowledge.hook({
  id: "order_shipping_rules",
  domain: "order.shipping",

  rules: [
    "Premium tier: 2-day shipping",
    "Gold tier: 3-day shipping",
    "Standard tier: 5-day shipping",
    "Orders >$1000: 15% discount",
    "Orders >$500: 10% discount"
  ]
})
fn process_order(order: &Order) -> Result<()> {
  // Code is now self-documenting
}
```

### Step 3: Link to Tests and Spans

```rust
@knowledge.hook({
  id: "order_processing",
  domain: "order",

  rules: [ /* ... */ ],

  // Link to existing tests
  related_tests: [
    "test_premium_tier_shipping",
    "test_gold_tier_shipping",
    "test_discount_thresholds"
  ],

  // Link to existing telemetry spans
  related_spans: [
    "order_validation",
    "shipping_calculation",
    "discount_application"
  ]
})
#[instrument]
fn process_order(order: &Order) -> Result<()> { }
```

### Step 4: Validate and Iterate

```bash
# Check K-hook validity
knhk k-hooks validate

# Check test coverage
knhk k-hooks validate-test-coverage --hook order_processing

# Check telemetry alignment
knhk k-hooks validate-telemetry --hook order_processing
```

---

## Part 9: K-hooks for Team Collaboration

### Share Knowledge Across Teams

```rust
// Frontend team adds K-hook to backend API they consume
@knowledge.hook({
  id: "user_api_contract",
  domain: "api.users",

  // Declares what the API does
  endpoints: [
    "GET /users/{id}",
    "POST /users",
    "PATCH /users/{id}"
  ],

  // Declares the contract (inputs/outputs)
  schemas: {
    user: {
      id: "integer",
      email: "string",
      created_at: "timestamp"
    }
  },

  // Declares expected behavior
  behavior: [
    "Returns 404 for non-existent user",
    "Email must be unique",
    "created_at is immutable"
  ]
})
pub async fn handle_user_request(req: &Request) -> Response { }
```

### Generate API Documentation from K-hooks

```bash
# Generate API docs from K-hooks
knhk k-hooks generate-api-docs --domain "api.*" --format "openapi"
# Creates: api-spec.yaml

# Validate client against K-hook spec
knhk k-hooks validate-client --spec api-spec.yaml
# Ensures frontend is using API correctly
```

---

## Summary

### K-hooks Checklist

- [ ] Identify business-critical functions
- [ ] Extract business rules into K-hook
- [ ] Link to related tests
- [ ] Link to related telemetry spans
- [ ] Document constraints and thresholds
- [ ] Version your K-hooks
- [ ] Validate test coverage
- [ ] Generate documentation
- [ ] Share knowledge across teams

### Key Takeaways

✓ K-hooks make business logic machine-readable
✓ Link K-hooks to tests and telemetry
✓ Use K-hooks for cross-team communication
✓ Extract knowledge without changing code
✓ Enable automation and agent navigation

---

**Created**: 2025-11-15
**Status**: Complete
**Difficulty**: Intermediate-Advanced
**Related**: [Emit Proper Telemetry](07-emit-proper-telemetry.md), [Implement Workflow Patterns](11-implement-workflow-patterns.md)
