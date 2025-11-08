fn create_order_test_data(order_id: &str, amount: &str) -> serde_json::Value {
    TestDataBuilder::new()
        .with_order_data(order_id, amount)
        .with_customer_data("CUST-001")
        .with_var("payment_method", "credit_card")
        .build_json()
}