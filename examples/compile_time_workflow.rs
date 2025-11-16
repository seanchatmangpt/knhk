//! Comprehensive example of compile-time workflow validation
//!
//! This example demonstrates:
//! - Workflow DSL with compile-time validation
//! - Type-safe state machines
//! - Const evaluation and assertions
//! - GADT-style workflow stages
//!
//! Run with: cargo run --example compile_time_workflow

use knhk_workflow_engine::compile_time::*;

fn main() {
    println!("=== Compile-Time Workflow Validation Example ===\n");

    // Example 1: Type-safe state machine
    println!("Example 1: Type-Safe State Machine");
    println!("-----------------------------------");

    let workflow = TypedWorkflow::<Initial>::new("workflow-001".to_string());
    println!("Created workflow: {} in state: {}", workflow.id(), workflow.state_name());
    println!("Is initial: {}", workflow.is_initial());
    println!("Is terminal: {}", workflow.is_terminal());

    // Type-safe transition: can only call methods valid for current state
    let workflow = workflow.validate_email();
    println!("Transitioned to: {}", workflow.state_name());

    let workflow = workflow.create_account();
    println!("Transitioned to: {}", workflow.state_name());

    let workflow = workflow.complete();
    println!("Transitioned to: {}", workflow.state_name());
    println!("Is terminal: {}", workflow.is_terminal());

    // These would be compile errors:
    // workflow.validate_email(); // ERROR: method not found for TypedWorkflow<Complete>
    // workflow.create_account(); // ERROR: method not found for TypedWorkflow<Complete>

    println!();

    // Example 2: Failure path
    println!("Example 2: Failure Path");
    println!("-----------------------");

    let workflow = TypedWorkflow::<Initial>::new("workflow-002".to_string());
    let workflow = workflow.validate_email();
    let workflow = workflow.reject_invalid(); // Failed validation
    println!("Workflow ended in state: {}", workflow.state_name());
    println!("Is terminal: {}", workflow.is_terminal());

    println!();

    // Example 3: Context management
    println!("Example 3: Context Management");
    println!("-----------------------------");

    let workflow = TypedWorkflow::<Initial>::new("workflow-003".to_string())
        .with_context("email".to_string(), serde_json::json!("user@example.com"))
        .with_context("user_agent".to_string(), serde_json::json!("Mozilla/5.0"));

    println!("Workflow context: {:?}", workflow.context());

    let workflow = workflow.validate_email()
        .with_context("validated_at".to_string(), serde_json::json!(chrono::Utc::now().to_rfc3339()));

    println!("Updated context: {:?}", workflow.context());

    println!();

    // Example 4: Compile-time metrics
    println!("Example 4: Compile-Time Metrics");
    println!("--------------------------------");

    type MyWorkflow = CompileTimeWorkflow<5, 6>;

    const COMPLEXITY: usize = MyWorkflow::complexity();
    const IS_SIMPLE: bool = MyWorkflow::is_simple();
    const MEMORY: usize = MyWorkflow::memory_usage();

    println!("Workflow complexity: {}", COMPLEXITY);
    println!("Is simple: {}", IS_SIMPLE);
    println!("Estimated memory: {} bytes", MEMORY);

    // Compile-time assertions (checked at compile time!)
    const_assert!(COMPLEXITY <= 100);
    const_assert!(MEMORY < 1_000_000);

    println!();

    // Example 5: Performance estimation
    println!("Example 5: Performance Estimation");
    println!("----------------------------------");

    const TRANSITIONS: usize = 4;
    const AVG_TICKS_PER_TRANSITION: usize = 2;
    const ESTIMATED_TICKS: usize = estimate_execution_ticks(TRANSITIONS, AVG_TICKS_PER_TRANSITION);

    println!("Transitions: {}", TRANSITIONS);
    println!("Average ticks per transition: {}", AVG_TICKS_PER_TRANSITION);
    println!("Estimated total ticks: {}", ESTIMATED_TICKS);
    println!("Meets Chatman Constant (≤8 ticks): {}", ESTIMATED_TICKS <= 8);

    const_assert!(ESTIMATED_TICKS <= 8, "Exceeds Chatman Constant");

    println!();

    // Example 6: GADT-style workflow with stage-specific data
    println!("Example 6: GADT Workflow with Stage-Specific Data");
    println!("--------------------------------------------------");

    let gadt_workflow = WorkflowStage::<stage::Initial>::new();
    println!("Created GADT workflow");

    let gadt_workflow = gadt_workflow.validate_email("alice@example.com".to_string());
    println!("Validating email: {}", gadt_workflow.email());

    let gadt_workflow = gadt_workflow.create_account("alice123".to_string());
    let (email, username) = gadt_workflow.account_details();
    println!("Creating account - Email: {}, Username: {}", email, username);

    let gadt_workflow = gadt_workflow.complete(789);
    let (user_id, email, username) = gadt_workflow.account_data();
    println!("Account created - ID: {}, Email: {}, Username: {}", user_id, email, username);

    println!();

    // Example 7: Type-level natural numbers
    println!("Example 7: Type-Level Natural Numbers");
    println!("--------------------------------------");

    use const_eval::nat::*;

    println!("N0 = {}", N0::VALUE);
    println!("N1 = {}", N1::VALUE);
    println!("N8 (Chatman Constant) = {}", N8::VALUE);
    println!("N10 = {}", N10::VALUE);

    println!();

    // Example 8: Workflow metrics
    println!("Example 8: Workflow Metrics");
    println!("---------------------------");

    let metrics = WorkflowMetrics::new(
        5,      // state_count
        6,      // transition_count
        3,      // complexity
        2,      // max_parallelism
        false,  // has_cycles
    );

    println!("States: {}", metrics.state_count);
    println!("Transitions: {}", metrics.transition_count);
    println!("Complexity: {}", metrics.complexity);
    println!("Max parallelism: {}", metrics.max_parallelism);
    println!("Has cycles: {}", metrics.has_cycles);
    println!("Is simple: {}", metrics.is_simple());
    println!("Is complex: {}", metrics.is_complex());
    println!("Estimated memory: {} bytes", metrics.estimated_memory_bytes());

    println!();

    println!("=== All examples completed successfully! ===");
}
