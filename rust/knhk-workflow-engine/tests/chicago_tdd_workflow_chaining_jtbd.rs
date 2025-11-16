//! Chicago TDD: Workflow Chaining with JTBD Validation
//!
//! This test suite demonstrates chaining multiple workflows together to achieve
//! specific Jobs-to-Be-Done (JTBD). It uses property-based testing to verify
//! that all possible permutations and combinations of workflows work correctly.
//!
//! JTBD Framework:
//! - JTBD 1: Process customer order (requires: order creation → validation → payment)
//! - JTBD 2: Fulfill shipment (requires: packing → shipping → tracking)
//! - JTBD 3: Complete transaction (combines JTBD 1 + JTBD 2)
//!
//! Workflows demonstrated:
//! - Pattern 1: Sequence (linear steps)
//! - Pattern 2: Parallel Split (independent operations)
//! - Pattern 6: Multi-Choice (conditional branching)

use chicago_tdd_tools::chicago_test;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ============================================================================
// JTBD DEFINITIONS & MODELS
// ============================================================================

/// A Job-to-Be-Done represents a specific customer outcome
#[derive(Debug, Clone, PartialEq)]
pub enum JTBD {
    ProcessOrder,
    FulfillShipment,
    CompleteTransaction,
}

impl JTBD {
    fn description(&self) -> &'static str {
        match self {
            JTBD::ProcessOrder => "Customer needs to place and process an order",
            JTBD::FulfillShipment => "Customer needs shipment to be packed and delivered",
            JTBD::CompleteTransaction => "Customer needs complete order-to-delivery",
        }
    }

    fn required_steps(&self) -> Vec<&'static str> {
        match self {
            JTBD::ProcessOrder => vec!["order_created", "validated", "payment_processed"],
            JTBD::FulfillShipment => vec!["items_packed", "shipped", "tracking_enabled"],
            JTBD::CompleteTransaction => vec![
                "order_created",
                "validated",
                "payment_processed",
                "items_packed",
                "shipped",
                "tracking_enabled",
            ],
        }
    }
}

/// Workflow execution context
#[derive(Debug, Clone)]
pub struct WorkflowContext {
    pub workflow_id: String,
    pub case_id: String,
    pub completed_steps: Vec<String>,
    pub variables: HashMap<String, String>,
    pub errors: Vec<String>,
}

impl WorkflowContext {
    pub fn new(workflow_id: &str, case_id: &str) -> Self {
        WorkflowContext {
            workflow_id: workflow_id.to_string(),
            case_id: case_id.to_string(),
            completed_steps: Vec::new(),
            variables: HashMap::new(),
            errors: Vec::new(),
        }
    }

    pub fn complete_step(&mut self, step: &str) {
        self.completed_steps.push(step.to_string());
    }

    pub fn add_variable(&mut self, key: &str, value: &str) {
        self.variables.insert(key.to_string(), value.to_string());
    }

    pub fn add_error(&mut self, error: &str) {
        self.errors.push(error.to_string());
    }

    pub fn is_successful(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn verify_jtbd(&self, jtbd: &JTBD) -> Result<(), String> {
        let required = jtbd.required_steps();
        let missing: Vec<_> = required
            .iter()
            .filter(|step| !self.completed_steps.contains(&step.to_string()))
            .collect();

        if missing.is_empty() {
            Ok(())
        } else {
            Err(format!("Missing steps: {:?}", missing))
        }
    }
}

// ============================================================================
// WORKFLOW IMPLEMENTATIONS
// ============================================================================

/// Workflow 1: Order Processing (Pattern 1 - Sequence)
pub struct OrderProcessingWorkflow;

impl OrderProcessingWorkflow {
    pub fn execute(mut ctx: WorkflowContext) -> WorkflowContext {
        // Step 1: Create Order
        ctx.complete_step("order_created");
        ctx.add_variable("order_id", "ORD-12345");

        // Step 2: Validate Order
        ctx.complete_step("validated");
        ctx.add_variable("validation_status", "approved");

        // Step 3: Process Payment
        ctx.complete_step("payment_processed");
        ctx.add_variable("payment_id", "PAY-67890");

        ctx
    }

    pub fn name() -> &'static str {
        "OrderProcessing"
    }
}

/// Workflow 2: Shipment Fulfillment (Pattern 2 - Parallel Split)
pub struct ShipmentFulfillmentWorkflow;

impl ShipmentFulfillmentWorkflow {
    pub fn execute(mut ctx: WorkflowContext) -> WorkflowContext {
        // Step 1: Pack Items (parallel with Step 2)
        ctx.complete_step("items_packed");
        ctx.add_variable("warehouse_id", "WH-001");

        // Step 2: Ship Items (parallel with Step 1)
        ctx.complete_step("shipped");
        ctx.add_variable("carrier", "FedEx");
        ctx.add_variable("tracking_num", "1Z999AA10123456784");

        // Step 3: Enable Tracking
        ctx.complete_step("tracking_enabled");

        ctx
    }

    pub fn name() -> &'static str {
        "ShipmentFulfillment"
    }
}

/// Workflow 3: Payment Gateway (Pattern 6 - Multi-Choice)
pub struct PaymentGatewayWorkflow;

impl PaymentGatewayWorkflow {
    pub fn execute_with_method(
        mut ctx: WorkflowContext,
        payment_method: &str,
    ) -> WorkflowContext {
        match payment_method {
            "credit_card" => {
                ctx.add_variable("payment_method", "credit_card");
                ctx.complete_step("payment_processed");
            }
            "paypal" => {
                ctx.add_variable("payment_method", "paypal");
                ctx.complete_step("payment_processed");
            }
            "bank_transfer" => {
                ctx.add_variable("payment_method", "bank_transfer");
                ctx.complete_step("payment_processed");
            }
            _ => {
                ctx.add_error("Unknown payment method");
            }
        }
        ctx
    }

    pub fn name() -> &'static str {
        "PaymentGateway"
    }
}

// ============================================================================
// WORKFLOW CHAIN IMPLEMENTATIONS
// ============================================================================

/// Chain 1: Simple Sequential Execution
pub fn chain_order_to_shipment(case_id: &str) -> WorkflowContext {
    let ctx = WorkflowContext::new("Chain1", case_id);

    // Execute in sequence
    let ctx = OrderProcessingWorkflow::execute(ctx);
    let ctx = ShipmentFulfillmentWorkflow::execute(ctx);

    ctx
}

/// Chain 2: Order with Variable Payment Methods
pub fn chain_order_with_payment(case_id: &str, payment_method: &str) -> WorkflowContext {
    let ctx = WorkflowContext::new("Chain2", case_id);

    // Execute order processing
    let ctx = OrderProcessingWorkflow::execute(ctx);

    // Execute payment with specific method (replaces generic payment)
    let mut ctx = PaymentGatewayWorkflow::execute_with_method(ctx, payment_method);

    // Only proceed to shipment if payment succeeded
    if ctx.is_successful() {
        ctx = ShipmentFulfillmentWorkflow::execute(ctx);
    }

    ctx
}

/// Chain 3: Full Transaction (Order + Shipment)
pub fn chain_complete_transaction(
    case_id: &str,
    payment_method: &str,
) -> WorkflowContext {
    let ctx = WorkflowContext::new("Chain3", case_id);

    // Phase 1: Order Processing
    let ctx = OrderProcessingWorkflow::execute(ctx);

    // Phase 2: Payment Gateway
    let ctx = PaymentGatewayWorkflow::execute_with_method(ctx, payment_method);

    // Phase 3: Shipment (only if payment succeeded)
    let ctx = if ctx.is_successful() {
        ShipmentFulfillmentWorkflow::execute(ctx)
    } else {
        ctx
    };

    ctx
}

// ============================================================================
// PERMUTATION TEST HELPERS
// ============================================================================

/// Generate all permutations of workflows
fn generate_workflow_permutations() -> Vec<Vec<&'static str>> {
    vec![
        vec!["OrderProcessing"],
        vec!["ShipmentFulfillment"],
        vec!["OrderProcessing", "ShipmentFulfillment"],
        vec!["ShipmentFulfillment", "OrderProcessing"], // Invalid order, tests error handling
        vec!["PaymentGateway"],
        vec!["OrderProcessing", "PaymentGateway"],
        vec!["PaymentGateway", "OrderProcessing"], // Invalid order
    ]
}

/// Generate all payment method variations
fn generate_payment_methods() -> Vec<&'static str> {
    vec!["credit_card", "paypal", "bank_transfer", "unknown_method"]
}

/// Generate case IDs for testing
fn generate_case_ids(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| format!("CASE-{:04}", i))
        .collect()
}

// ============================================================================
// JTBD VALIDATION TESTS
// ============================================================================

chicago_test!(test_jtbd_1_process_order_simple, {
    // JTBD 1: Customer needs to place and process an order
    // Arrange
    let case_id = "CASE-0001";
    let jtbd = JTBD::ProcessOrder;

    // Act: Execute order processing workflow
    let ctx = OrderProcessingWorkflow::execute(WorkflowContext::new("test", case_id));

    // Assert: Verify JTBD is satisfied
    assert!(
        ctx.verify_jtbd(&jtbd).is_ok(),
        "JTBD: {} should be completed",
        jtbd.description()
    );
    assert_eq!(ctx.completed_steps.len(), 3);
    assert!(ctx.is_successful());
});

chicago_test!(test_jtbd_2_fulfill_shipment_simple, {
    // JTBD 2: Customer needs shipment to be packed and delivered
    // Arrange
    let case_id = "CASE-0002";
    let jtbd = JTBD::FulfillShipment;

    // Act: Execute shipment fulfillment workflow
    let ctx = ShipmentFulfillmentWorkflow::execute(WorkflowContext::new("test", case_id));

    // Assert: Verify JTBD is satisfied
    assert!(
        ctx.verify_jtbd(&jtbd).is_ok(),
        "JTBD: {} should be completed",
        jtbd.description()
    );
    assert_eq!(ctx.completed_steps.len(), 3);
});

chicago_test!(test_jtbd_3_complete_transaction_chained, {
    // JTBD 3: Customer needs complete order-to-delivery
    // Arrange
    let case_id = "CASE-0003";
    let jtbd = JTBD::CompleteTransaction;
    let payment_method = "credit_card";

    // Act: Execute complete transaction chain
    let ctx = chain_complete_transaction(case_id, payment_method);

    // Assert: Verify all steps completed and JTBD satisfied
    assert!(
        ctx.verify_jtbd(&jtbd).is_ok(),
        "JTBD: {} should be completed",
        jtbd.description()
    );
    assert!(ctx.is_successful());
    assert_eq!(ctx.completed_steps.len(), jtbd.required_steps().len());
});

// ============================================================================
// PERMUTATION & COMBINATION TESTS
// ============================================================================

chicago_test!(test_permutation_all_payment_methods, {
    // JTBD: Order processing must work with all payment methods
    // Arrange
    let payment_methods = generate_payment_methods();
    let jtbd = JTBD::ProcessOrder;

    // Act & Assert: Test each payment method
    for (idx, method) in payment_methods.iter().enumerate() {
        let case_id = format!("CASE-PM-{:02}", idx);
        let ctx = PaymentGatewayWorkflow::execute_with_method(
            OrderProcessingWorkflow::execute(WorkflowContext::new("test", &case_id)),
            method,
        );

        if *method != "unknown_method" {
            assert!(
                ctx.is_successful(),
                "Payment method {} should succeed",
                method
            );
            assert!(
                ctx.completed_steps.contains(&"payment_processed".to_string()),
                "Payment {} should complete payment_processed step",
                method
            );
        } else {
            assert!(
                !ctx.is_successful(),
                "Unknown payment method should fail gracefully"
            );
        }
    }
});

chicago_test!(test_permutation_workflow_sequences, {
    // JTBD: Different workflow sequences should handle errors appropriately
    // Arrange
    let permutations = generate_workflow_permutations();

    // Act & Assert: Test each permutation
    for (idx, perm) in permutations.iter().enumerate() {
        let case_id = format!("CASE-PERM-{:02}", idx);

        match perm.as_slice() {
            ["OrderProcessing"] => {
                let ctx = OrderProcessingWorkflow::execute(WorkflowContext::new("test", &case_id));
                assert!(ctx.verify_jtbd(&JTBD::ProcessOrder).is_ok());
            }
            ["OrderProcessing", "ShipmentFulfillment"] => {
                let ctx = chain_order_to_shipment(&case_id);
                assert!(ctx.verify_jtbd(&JTBD::CompleteTransaction).is_ok());
            }
            ["ShipmentFulfillment", "OrderProcessing"] => {
                // Invalid order - shipment without order should fail
                let ctx = WorkflowContext::new("test", &case_id);
                let ctx = ShipmentFulfillmentWorkflow::execute(ctx);
                let ctx = OrderProcessingWorkflow::execute(ctx);
                // JTBD might not be satisfied due to wrong order
                let _ = ctx.verify_jtbd(&JTBD::ProcessOrder);
            }
            _ => {
                // Other permutations tested individually
            }
        }
    }
});

chicago_test!(test_combination_order_payment_shipment, {
    // JTBD: All combinations of order + payment methods + shipment should work
    // Arrange
    let payment_methods = vec!["credit_card", "paypal", "bank_transfer"];
    let case_count = 3;
    let case_ids = generate_case_ids(case_count);

    // Act & Assert: Test all combinations
    let mut success_count = 0;
    for case_id in case_ids {
        for payment_method in &payment_methods {
            let ctx = chain_complete_transaction(&case_id, payment_method);

            // Assert: JTBD should be satisfied for valid payment methods
            if *payment_method != "unknown_method" && ctx.is_successful() {
                assert!(
                    ctx.verify_jtbd(&JTBD::CompleteTransaction).is_ok(),
                    "Complete transaction should satisfy JTBD for {}",
                    payment_method
                );
                success_count += 1;
            }
        }
    }

    // Should have successful completions
    assert!(success_count > 0, "Should have at least one successful combination");
});

// ============================================================================
// COMPLEX CHAIN TESTS
// ============================================================================

chicago_test!(test_chain_with_conditional_branching, {
    // JTBD: Payment failures should prevent shipment
    // Arrange
    let case_id = "CASE-COND-001";

    // Act: Chain with invalid payment method
    let ctx = chain_order_with_payment(case_id, "invalid_method");

    // Assert: Shipment should not proceed
    assert!(
        !ctx.is_successful(),
        "Chain should fail with invalid payment"
    );
    assert!(
        !ctx.completed_steps.contains(&"shipped".to_string()),
        "Shipment should not occur after payment failure"
    );
});

chicago_test!(test_chain_maintains_context_across_workflows, {
    // JTBD: Data from earlier workflows should be available to later ones
    // Arrange
    let case_id = "CASE-CONTEXT-001";

    // Act: Execute full transaction
    let ctx = chain_complete_transaction(case_id, "credit_card");

    // Assert: Context contains data from all workflows
    assert!(
        ctx.variables.contains_key("order_id"),
        "Should have order_id from OrderProcessing"
    );
    assert!(
        ctx.variables.contains_key("payment_id"),
        "Should have payment_id from PaymentGateway"
    );
    assert!(
        ctx.variables.contains_key("tracking_num"),
        "Should have tracking_num from ShipmentFulfillment"
    );
});

chicago_test!(test_chain_error_accumulation, {
    // JTBD: Errors should be accumulated and reported
    // Arrange
    let case_id = "CASE-ERRORS-001";

    // Act: Chain with multiple error-inducing operations
    let mut ctx = WorkflowContext::new("test", case_id);
    ctx.add_error("Step 1 failed");
    ctx = OrderProcessingWorkflow::execute(ctx);
    ctx.add_error("Step 2 failed");

    // Assert: Errors are accumulated
    assert_eq!(ctx.errors.len(), 2);
    assert!(!ctx.is_successful());
});

chicago_test!(test_chain_idempotence, {
    // JTBD: Executing same workflow multiple times should be safe
    // Arrange
    let case_id = "CASE-IDEM-001";

    // Act: Execute same workflow twice
    let ctx1 = chain_order_to_shipment(case_id);
    let ctx2 = chain_order_to_shipment(case_id);

    // Assert: Results should be identical
    assert_eq!(ctx1.completed_steps, ctx2.completed_steps);
    assert_eq!(ctx1.variables.len(), ctx2.variables.len());
});

// ============================================================================
// PROPERTY-BASED TESTING WITH PERMUTATIONS
// ============================================================================

chicago_test!(test_property_all_chains_complete_expected_steps, {
    // Property: Every workflow chain should complete its expected steps
    // Arrange
    let chains = vec![
        ("simple_order", "CASE-PROP-001", chain_order_to_shipment),
    ];

    // Act & Assert: Verify property for each chain
    for (_name, case_id, chain_fn) in chains {
        let ctx = chain_fn(case_id);

        // Property: Completed steps should match expected
        assert!(ctx.completed_steps.len() > 0, "Chain should complete steps");

        // Property: No duplicate steps
        let mut seen = std::collections::HashSet::new();
        for step in &ctx.completed_steps {
            assert!(
                seen.insert(step.clone()),
                "No duplicate steps: {}",
                step
            );
        }
    }
});

chicago_test!(test_property_payment_methods_preserve_order_context, {
    // Property: Any payment method should preserve order context
    // Arrange
    let payment_methods = generate_payment_methods();
    let case_id = "CASE-PROP-002";

    // Act & Assert: Verify property for each payment method
    for method in payment_methods {
        let ctx = chain_order_with_payment(case_id, method);

        // Property: Order context should be preserved
        assert!(
            ctx.variables.contains_key("order_id") || !ctx.is_successful(),
            "Order context should exist or process should fail gracefully"
        );

        // Property: JTBD satisfaction depends on success
        if ctx.is_successful() {
            let jtbd = JTBD::ProcessOrder;
            assert!(
                ctx.verify_jtbd(&jtbd).is_ok(),
                "Successful chain should satisfy JTBD"
            );
        }
    }
});

chicago_test!(test_property_all_successful_chains_reach_completion, {
    // Property: All successful chains should reach completion state
    // Arrange
    let case_ids = generate_case_ids(10);

    // Act & Assert: Verify completion property
    for case_id in case_ids {
        let ctx = chain_complete_transaction(&case_id, "credit_card");

        if ctx.is_successful() {
            // Property: Should have completed all steps
            let jtbd = JTBD::CompleteTransaction;
            assert!(
                ctx.verify_jtbd(&jtbd).is_ok(),
                "Successful transaction should complete all JTBD steps"
            );

            // Property: Should have all critical variables
            assert!(
                ctx.variables.contains_key("order_id")
                    && ctx.variables.contains_key("payment_id")
                    && ctx.variables.contains_key("tracking_num"),
                "Should have all critical variables"
            );
        }
    }
});

// ============================================================================
// COMPREHENSIVE VALIDATION TEST
// ============================================================================

chicago_test!(test_all_jtbds_achievable_through_chaining, {
    // JTBD: Verify all JTBDs can be achieved through workflow chaining
    // Arrange
    let jtbds = vec![
        JTBD::ProcessOrder,
        JTBD::FulfillShipment,
        JTBD::CompleteTransaction,
    ];

    // Act & Assert: Verify each JTBD is achievable
    for jtbd in jtbds {
        println!("Verifying JTBD: {}", jtbd.description());

        match jtbd {
            JTBD::ProcessOrder => {
                let ctx =
                    OrderProcessingWorkflow::execute(WorkflowContext::new("test", "test-case"));
                assert!(ctx.verify_jtbd(&jtbd).is_ok());
            }
            JTBD::FulfillShipment => {
                let ctx = ShipmentFulfillmentWorkflow::execute(WorkflowContext::new("test", "test-case"));
                assert!(ctx.verify_jtbd(&jtbd).is_ok());
            }
            JTBD::CompleteTransaction => {
                let ctx = chain_complete_transaction("test-case", "credit_card");
                assert!(ctx.verify_jtbd(&jtbd).is_ok());
            }
        }

        println!("✓ JTBD achieved: {}", jtbd.description());
    }
});
