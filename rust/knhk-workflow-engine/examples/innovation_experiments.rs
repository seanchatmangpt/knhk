//! Innovation Experiments Example
//!
//! Demonstrates running innovation experiments:
//! - Deterministic execution experiments
//! - Formal verification experiments
//! - Hardware acceleration experiments
//! - Zero-copy optimization experiments
//! - Integrated experiments (all types)
//!
//! Usage:
//!   cargo run --example innovation_experiments

use knhk_workflow_engine::case::{Case, CaseId, CaseState};
use knhk_workflow_engine::innovation::{ExperimentRunner, ExperimentType};
use knhk_workflow_engine::parser::{
    Flow, JoinType, SplitType, Task, TaskType, WorkflowSpec, WorkflowSpecId,
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Innovation Experiments Example\n");

    // Create experiment runner
    let mut runner = ExperimentRunner::new(42);

    // Create test workflow spec
    let spec = create_test_workflow_spec();

    // Create test case
    let case = create_test_case(&spec.id);

    // Test data for hardware acceleration
    let test_data = b"test data for hardware acceleration experiment";

    println!("📊 Running Innovation Experiments...\n");

    // 1. Deterministic execution experiment
    println!("1️⃣  Running Deterministic Execution Experiment...");
    let deterministic_result = runner
        .run_deterministic_experiment("deterministic_execution".to_string(), &case)
        .await?;
    println!("   ✅ Success: {}", deterministic_result.success);
    println!(
        "   ⏱️  Execution time: {} ns",
        deterministic_result.execution_time_ns
    );
    println!("   📈 Results: {}\n", deterministic_result.results);

    // 2. Formal verification experiment
    println!("2️⃣  Running Formal Verification Experiment...");
    let formal_result =
        runner.run_formal_verification_experiment("formal_verification".to_string(), &spec)?;
    println!("   ✅ Success: {}", formal_result.success);
    println!(
        "   ⏱️  Execution time: {} ns",
        formal_result.execution_time_ns
    );
    println!("   📈 Results: {}\n", formal_result.results);

    // 3. Hardware acceleration experiment
    println!("3️⃣  Running Hardware Acceleration Experiment...");
    let hardware_result = runner
        .run_hardware_acceleration_experiment("hardware_acceleration".to_string(), test_data)?;
    println!("   ✅ Success: {}", hardware_result.success);
    println!(
        "   ⏱️  Execution time: {} ns",
        hardware_result.execution_time_ns
    );
    println!("   📈 Results: {}\n", hardware_result.results);

    // 4. Zero-copy optimization experiment
    println!("4️⃣  Running Zero-Copy Optimization Experiment...");
    let zero_copy_result = runner.run_zero_copy_experiment(
        "zero_copy_optimization".to_string(),
        vec![
            (
                "http://example.org/subject1",
                "http://example.org/predicate1",
                "http://example.org/object1",
            ),
            (
                "http://example.org/subject2",
                "http://example.org/predicate2",
                "http://example.org/object2",
            ),
            (
                "http://example.org/subject3",
                "http://example.org/predicate3",
                "http://example.org/object3",
            ),
        ],
    )?;
    println!("   ✅ Success: {}", zero_copy_result.success);
    println!(
        "   ⏱️  Execution time: {} ns",
        zero_copy_result.execution_time_ns
    );
    println!("   📈 Results: {}\n", zero_copy_result.results);

    // 5. Integrated experiment (all types)
    println!("5️⃣  Running Integrated Experiment (All Types)...");
    let integrated_result = runner
        .run_integrated_experiment("integrated_experiment".to_string(), &spec, &case, test_data)
        .await?;
    println!("   ✅ Success: {}", integrated_result.success);
    println!(
        "   ⏱️  Execution time: {} ns",
        integrated_result.execution_time_ns
    );
    println!("   📈 Results: {}\n", integrated_result.results);

    // Print summary
    println!("📊 Experiment Summary:");
    println!("{}", serde_json::to_string_pretty(&runner.get_summary())?);

    println!("\n✅ All experiments completed!");

    Ok(())
}

fn create_test_workflow_spec() -> WorkflowSpec {
    let spec_id = WorkflowSpecId::new();

    let mut tasks = std::collections::HashMap::new();
    tasks.insert(
        "task1".to_string(),
        Task {
            id: "task1".to_string(),
            name: "Task 1".to_string(),
            task_type: TaskType::Manual,
            incoming_flows: vec!["start".to_string()],
            outgoing_flows: vec!["task2".to_string()],
        },
    );
    tasks.insert(
        "task2".to_string(),
        Task {
            id: "task2".to_string(),
            name: "Task 2".to_string(),
            task_type: TaskType::Automatic,
            incoming_flows: vec!["task1".to_string()],
            outgoing_flows: vec!["end".to_string()],
        },
    );

    let mut flows = std::collections::HashMap::new();
    flows.insert(
        "flow1".to_string(),
        Flow {
            id: "flow1".to_string(),
            from: "start".to_string(),
            to: "task1".to_string(),
            split_type: SplitType::Xor,
            join_type: JoinType::Xor,
        },
    );
    flows.insert(
        "flow2".to_string(),
        Flow {
            id: "flow2".to_string(),
            from: "task1".to_string(),
            to: "task2".to_string(),
            split_type: SplitType::Xor,
            join_type: JoinType::Xor,
        },
    );
    flows.insert(
        "flow3".to_string(),
        Flow {
            id: "flow3".to_string(),
            from: "task2".to_string(),
            to: "end".to_string(),
            split_type: SplitType::Xor,
            join_type: JoinType::Xor,
        },
    );

    WorkflowSpec {
        id: spec_id,
        name: "Test Workflow".to_string(),
        tasks,
        conditions: std::collections::HashMap::new(),
        flows,
        start_condition: Some("start".to_string()),
        end_condition: Some("end".to_string()),
    }
}

fn create_test_case(spec_id: &WorkflowSpecId) -> Case {
    Case {
        id: CaseId::new(),
        spec_id: spec_id.clone(),
        state: CaseState::Created,
        data: json!({
            "test": "data",
            "value": 42,
        }),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}
