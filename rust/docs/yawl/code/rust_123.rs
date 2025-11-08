use knhk_workflow_engine::testing::chicago_tdd::TestDataBuilder;

// Simple data
let data = TestDataBuilder::new()
    .with_var("order_id", "ORD-001")
    .with_var("amount", "100.00")
    .build_json();

// Business scenario data
let order_data = TestDataBuilder::new()
    .with_order_data("ORD-2024-001234", "149.99")
    .with_customer_data("CUST-789456")
    .build_json();

// Approval data
let approval_data = TestDataBuilder::new()
    .with_approval_data("REQ-001", "5000.00")
    .build_json();