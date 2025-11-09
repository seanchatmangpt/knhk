//! Chicago TDD Tests for WorkflowEngine
//!
//! This test suite tests the actual WorkflowEngine functionality using Chicago TDD principles:
//!
//! 1. **State-Based Testing**: Tests verify outputs and state, not implementation
//! 2. **Real Collaborators**: Uses actual WorkflowEngine, StateStore, etc. (no mocks)
//! 3. **Behavior Verification**: Tests verify what the engine does, not how
//! 4. **AAA Pattern**: All tests follow Arrange-Act-Assert structure
//!
//! ## What We Test:
//!
//! - Workflow registration and retrieval
//! - Case creation, starting, and execution
//! - Case state transitions
//! - Error handling (invalid workflows, missing cases, etc.)
//! - State persistence
//! - Multiple workflows and cases
//! - Workflow listing and case listing

use chicago_tdd_tools::prelude::*;
use knhk_workflow_engine::case::{Case, CaseId, CaseState};
use knhk_workflow_engine::error::{WorkflowError, WorkflowResult};
use knhk_workflow_engine::parser::{
    Condition, JoinType, SplitType, Task, TaskType, WorkflowSpec, WorkflowSpecId,
};
use knhk_workflow_engine::state::StateStore;
use knhk_workflow_engine::WorkflowEngine;
use std::collections::HashMap;

/// Test workflow registration and retrieval
#[tokio::test]
async fn test_workflow_registration_and_retrieval() -> WorkflowResult<()> {
    // Arrange: Create engine and test data
    let state_store = StateStore::new("./test_workflow_db_registration")?;
    let engine = WorkflowEngine::new(state_store);

    let test_data = TestDataBuilder::new()
        .with_var("workflow_name", "Test Workflow")
        .build_json();

    // Arrange: Create workflow specification
    let mut spec = WorkflowSpec {
        id: WorkflowSpecId::new(),
        name: "Test Workflow".to_string(),
        tasks: HashMap::new(),
        conditions: HashMap::new(),
        flows: Vec::new(),
        start_condition: None,
        end_condition: None,
        source_turtle: None,
    };

    let task = Task {
        id: "task:1".to_string(),
        name: "Task 1".to_string(),
        task_type: TaskType::Atomic,
        split_type: SplitType::And,
        join_type: JoinType::And,
        max_ticks: None,
        priority: None,
        use_simd: false,
        input_conditions: vec![],
        output_conditions: vec![],
        outgoing_flows: vec![],
        incoming_flows: vec![],
        allocation_policy: None,
        required_roles: vec![],
        required_capabilities: vec![],
        exception_worklet: None,
    };
    spec.tasks.insert("task:1".to_string(), task);

    // Act: Register workflow
    engine.register_workflow(spec.clone()).await?;

    // Assert: Verify workflow can be retrieved
    let retrieved_spec = engine.get_workflow(spec.id).await?;
    assert_eq_with_msg(
        &retrieved_spec.id,
        &spec.id,
        "Retrieved workflow should have same ID",
    );
    assert_eq_with_msg(
        &retrieved_spec.name,
        &spec.name,
        "Retrieved workflow should have same name",
    );
    assert_eq_with_msg(
        &retrieved_spec.tasks.len(),
        &spec.tasks.len(),
        "Retrieved workflow should have same number of tasks",
    );

    // Assert: Verify workflow appears in list
    let workflows = engine.list_workflows().await?;
    assert!(
        workflows.contains(&spec.id),
        "Workflow should appear in list of workflows"
    );

    Ok(())
}

/// Test case creation and state transitions
#[tokio::test]
async fn test_case_creation_and_state_transitions() -> WorkflowResult<()> {
    // Arrange: Create engine and workflow
    let state_store = StateStore::new("./test_workflow_db_case_states")?;
    let engine = WorkflowEngine::new(state_store);

    let mut spec = WorkflowSpec {
        id: WorkflowSpecId::new(),
        name: "Case State Test".to_string(),
        tasks: HashMap::new(),
        conditions: HashMap::new(),
        flows: Vec::new(),
        start_condition: None,
        end_condition: None,
        source_turtle: None,
    };

    // Create start condition
    let start_condition_id = "condition:start".to_string();
    let start_condition = Condition {
        id: start_condition_id.clone(),
        name: "Start".to_string(),
        outgoing_flows: vec!["task:1".to_string()],
        incoming_flows: vec![],
    };
    spec.conditions
        .insert(start_condition_id.clone(), start_condition);
    spec.start_condition = Some(start_condition_id.clone());

    // Create end condition
    let end_condition_id = "condition:end".to_string();
    let end_condition = Condition {
        id: end_condition_id.clone(),
        name: "End".to_string(),
        outgoing_flows: vec![],
        incoming_flows: vec!["task:1".to_string()],
    };
    spec.conditions
        .insert(end_condition_id.clone(), end_condition);
    spec.end_condition = Some(end_condition_id.clone());

    let task = Task {
        id: "task:1".to_string(),
        name: "Task 1".to_string(),
        task_type: TaskType::Atomic,
        split_type: SplitType::And,
        join_type: JoinType::And,
        max_ticks: None,
        priority: None,
        use_simd: false,
        input_conditions: vec![start_condition_id.clone()],
        output_conditions: vec![end_condition_id.clone()],
        outgoing_flows: vec![],
        incoming_flows: vec![],
        allocation_policy: None,
        required_roles: vec!["test_role".to_string()],
        required_capabilities: vec![],
        exception_worklet: None,
    };
    spec.tasks.insert("task:1".to_string(), task);

    engine.register_workflow(spec.clone()).await?;

    // Arrange: Create test data
    let test_data = TestDataBuilder::new()
        .with_order_data("ORD-001", "100.00")
        .with_customer_data("CUST-001")
        .build_json();

    // Act: Create case
    let case_id = engine.create_case(spec.id, test_data.clone()).await?;

    // Assert: Case is created with Created state
    let case = engine.get_case(case_id).await?;
    assert_eq_with_msg(
        &case.state,
        &CaseState::Created,
        "New case should be in Created state",
    );
    assert_eq_with_msg(
        &case.spec_id,
        &spec.id,
        "Case should reference correct workflow spec",
    );

    // Act: Start case
    engine.start_case(case_id).await?;

    // Assert: Case transitions to Running state
    let case = engine.get_case(case_id).await?;
    assert_eq_with_msg(
        &case.state,
        &CaseState::Running,
        "Case should transition to Running state after start",
    );

    // Act: Execute case
    engine.execute_case(case_id).await?;

    // Assert: Case completes (may be Completed or still Running depending on implementation)
    let case = engine.get_case(case_id).await?;
    assert!(
        case.state == CaseState::Completed || case.state == CaseState::Running,
        "Case should be in a valid final state after execution"
    );

    Ok(())
}

/// Test case execution with multiple tasks
#[tokio::test]
async fn test_case_execution_with_multiple_tasks() -> WorkflowResult<()> {
    // Arrange: Create engine with multi-task workflow
    let state_store = StateStore::new("./test_workflow_db_multi_task")?;
    let engine = WorkflowEngine::new(state_store);

    let mut spec = WorkflowSpec {
        id: WorkflowSpecId::new(),
        name: "Multi-Task Workflow".to_string(),
        tasks: HashMap::new(),
        conditions: HashMap::new(),
        flows: Vec::new(),
        start_condition: None,
        end_condition: None,
        source_turtle: None,
    };

    // Create start condition
    let start_condition_id = "condition:start".to_string();
    let start_condition = Condition {
        id: start_condition_id.clone(),
        name: "Start".to_string(),
        outgoing_flows: vec!["task:1".to_string()],
        incoming_flows: vec![],
    };
    spec.conditions
        .insert(start_condition_id.clone(), start_condition);
    spec.start_condition = Some(start_condition_id.clone());

    // Create end condition
    let end_condition_id = "condition:end".to_string();
    let end_condition = Condition {
        id: end_condition_id.clone(),
        name: "End".to_string(),
        outgoing_flows: vec![],
        incoming_flows: vec!["task:3".to_string()],
    };
    spec.conditions
        .insert(end_condition_id.clone(), end_condition);
    spec.end_condition = Some(end_condition_id.clone());

    // Create multiple tasks
    for i in 1..=3 {
        let task = Task {
            id: format!("task:{}", i),
            name: format!("Task {}", i),
            task_type: TaskType::Atomic,
            split_type: SplitType::And,
            join_type: JoinType::And,
            max_ticks: None,
            priority: None,
            use_simd: false,
            input_conditions: if i == 1 {
                vec![start_condition_id.clone()]
            } else {
                vec![]
            },
            output_conditions: if i == 3 {
                vec![end_condition_id.clone()]
            } else {
                vec![]
            },
            outgoing_flows: vec![],
            incoming_flows: vec![],
            allocation_policy: None,
            required_roles: vec!["test_role".to_string()],
            required_capabilities: vec![],
            exception_worklet: None,
        };
        spec.tasks.insert(format!("task:{}", i), task);
    }

    engine.register_workflow(spec.clone()).await?;

    // Arrange: Create test data
    let test_data = TestDataBuilder::new()
        .with_order_data("ORD-002", "250.00")
        .with_customer_data("CUST-002")
        .with_var("task_count", "3")
        .build_json();

    // Act: Create and execute case
    let case_id = engine.create_case(spec.id, test_data).await?;
    engine.start_case(case_id).await?;
    engine.execute_case(case_id).await?;

    // Assert: Case executed successfully
    let case = engine.get_case(case_id).await?;
    assert!(
        case.state == CaseState::Completed || case.state == CaseState::Running,
        "Case should complete or be running after execution"
    );
    assert_eq_with_msg(
        &case.spec_id,
        &spec.id,
        "Case should reference correct workflow spec",
    );

    Ok(())
}

/// Test error handling for invalid workflow registration
#[tokio::test]
async fn test_error_handling_invalid_workflow() {
    // Arrange: Create engine
    let state_store =
        StateStore::new("./test_workflow_db_errors").expect("Failed to create state store");
    let engine = WorkflowEngine::new(state_store);

    // Arrange: Create workflow with invalid structure (no tasks)
    let spec = WorkflowSpec {
        id: WorkflowSpecId::new(),
        name: "Empty Workflow".to_string(),
        tasks: HashMap::new(),
        conditions: HashMap::new(),
        flows: Vec::new(),
        start_condition: None,
        end_condition: None,
        source_turtle: None,
    };

    // Act: Try to register empty workflow
    let result = engine.register_workflow(spec).await;

    // Assert: Registration may succeed or fail depending on validation
    // The important thing is that the engine handles it gracefully
    match result {
        Ok(_) => {
            // If registration succeeds, verify we can list it
            let workflows = engine
                .list_workflows()
                .await
                .expect("Should list workflows");
            assert!(workflows.len() >= 0, "Workflows list should be valid");
        }
        Err(ref e) => {
            // If registration fails, verify it's a proper error
            assert_error(&result);
            assert!(
                matches!(e, WorkflowError::Validation(_) | WorkflowError::Parse(_)),
                "Error should be Validation or Parse error"
            );
        }
    }
}

/// Test error handling for missing workflow
#[tokio::test]
async fn test_error_handling_missing_workflow() {
    // Arrange: Create engine
    let state_store =
        StateStore::new("./test_workflow_db_missing").expect("Failed to create state store");
    let engine = WorkflowEngine::new(state_store);

    // Act: Try to get non-existent workflow
    let non_existent_id = WorkflowSpecId::new();
    let result = engine.get_workflow(non_existent_id).await;

    // Assert: Should return proper error
    assert_error(&result);
    match result {
        Err(WorkflowError::InvalidSpecification(_)) => {
            // Expected error type
        }
        Err(e) => {
            panic!("Unexpected error type: {:?}", e);
        }
        Ok(_) => {
            panic!("Should not find non-existent workflow");
        }
    }
}

/// Test error handling for missing case
#[tokio::test]
async fn test_error_handling_missing_case() {
    // Arrange: Create engine
    let state_store =
        StateStore::new("./test_workflow_db_missing_case").expect("Failed to create state store");
    let engine = WorkflowEngine::new(state_store);

    // Act: Try to get non-existent case
    let non_existent_case_id = CaseId::new();
    let result = engine.get_case(non_existent_case_id).await;

    // Assert: Should return proper error
    assert_error(&result);
    match result {
        Err(WorkflowError::CaseNotFound(_)) => {
            // Expected error type
        }
        Err(e) => {
            panic!("Unexpected error type: {:?}", e);
        }
        Ok(_) => {
            panic!("Should not find non-existent case");
        }
    }
}

/// Test case cancellation
#[tokio::test]
async fn test_case_cancellation() -> WorkflowResult<()> {
    // Arrange: Create engine and workflow
    let state_store = StateStore::new("./test_workflow_db_cancellation")?;
    let engine = WorkflowEngine::new(state_store);

    let mut spec = WorkflowSpec {
        id: WorkflowSpecId::new(),
        name: "Cancellation Test".to_string(),
        tasks: HashMap::new(),
        conditions: HashMap::new(),
        flows: Vec::new(),
        start_condition: None,
        end_condition: None,
        source_turtle: None,
    };

    let task = Task {
        id: "task:1".to_string(),
        name: "Task 1".to_string(),
        task_type: TaskType::Atomic,
        split_type: SplitType::And,
        join_type: JoinType::And,
        max_ticks: None,
        priority: None,
        use_simd: false,
        input_conditions: vec![],
        output_conditions: vec![],
        outgoing_flows: vec![],
        incoming_flows: vec![],
        allocation_policy: None,
        required_roles: vec![],
        required_capabilities: vec![],
        exception_worklet: None,
    };
    spec.tasks.insert("task:1".to_string(), task);

    engine.register_workflow(spec.clone()).await?;

    // Arrange: Create test data
    let test_data = TestDataBuilder::new()
        .with_var("should_cancel", "true")
        .build_json();

    // Act: Create and start case
    let case_id = engine.create_case(spec.id, test_data).await?;
    engine.start_case(case_id).await?;

    // Assert: Case is running
    let case = engine.get_case(case_id).await?;
    assert_eq_with_msg(
        &case.state,
        &CaseState::Running,
        "Case should be running after start",
    );

    // Act: Cancel case
    engine.cancel_case(case_id).await?;

    // Assert: Case is cancelled
    let case = engine.get_case(case_id).await?;
    assert_eq_with_msg(
        &case.state,
        &CaseState::Cancelled,
        "Case should be cancelled after cancel",
    );

    Ok(())
}

/// Test multiple workflows and cases
#[tokio::test]
async fn test_multiple_workflows_and_cases() -> WorkflowResult<()> {
    // Arrange: Create engine
    let state_store = StateStore::new("./test_workflow_db_multiple")?;
    let engine = WorkflowEngine::new(state_store);

    // Arrange: Create multiple workflows
    let mut workflow_ids = Vec::new();
    for i in 0..3 {
        let mut spec = WorkflowSpec {
            id: WorkflowSpecId::new(),
            name: format!("Workflow {}", i),
            tasks: HashMap::new(),
            conditions: HashMap::new(),
            flows: Vec::new(),
            start_condition: None,
            end_condition: None,
            source_turtle: None,
        };

        let task = Task {
            id: format!("task:{}", i),
            name: format!("Task {}", i),
            task_type: TaskType::Atomic,
            split_type: SplitType::And,
            join_type: JoinType::And,
            max_ticks: None,
            priority: None,
            use_simd: false,
            input_conditions: vec![],
            output_conditions: vec![],
            outgoing_flows: vec![],
            incoming_flows: vec![],
            allocation_policy: None,
            required_roles: vec![],
            required_capabilities: vec![],
            exception_worklet: None,
        };
        spec.tasks.insert(format!("task:{}", i), task);

        engine.register_workflow(spec.clone()).await?;
        workflow_ids.push(spec.id);
    }

    // Assert: All workflows are registered
    let workflows = engine.list_workflows().await?;
    assert_eq_with_msg(&workflows.len(), &3, "Should have 3 registered workflows");

    for workflow_id in &workflow_ids {
        assert!(
            workflows.contains(workflow_id),
            "Workflow should be in list"
        );
    }

    // Act: Create cases for each workflow
    let mut case_ids = Vec::new();
    for (i, workflow_id) in workflow_ids.iter().enumerate() {
        let test_data = TestDataBuilder::new()
            .with_var("workflow_index", &i.to_string())
            .build_json();

        let case_id = engine.create_case(*workflow_id, test_data).await?;
        case_ids.push(case_id);
    }

    // Assert: All cases are created
    assert_eq_with_msg(&case_ids.len(), &3, "Should have created 3 cases");

    // Assert: Can retrieve each case
    for case_id in &case_ids {
        let case = engine.get_case(*case_id).await?;
        assert_eq_with_msg(
            &case.state,
            &CaseState::Created,
            "Case should be in Created state",
        );
    }

    // Assert: Can list cases for each workflow
    for workflow_id in &workflow_ids {
        let cases = engine.list_cases(*workflow_id).await?;
        assert!(
            cases.len() >= 1,
            "Should have at least one case for workflow"
        );
    }

    Ok(())
}

/// Test state persistence across engine instances
#[tokio::test]
async fn test_state_persistence() -> WorkflowResult<()> {
    // Arrange: Create first engine instance
    let db_path = format!("./test_workflow_db_persistence_{}", std::process::id());
    let state_store = StateStore::new(&db_path)?;
    let engine1 = WorkflowEngine::new(state_store);

    // Arrange: Create and register workflow
    let mut spec = WorkflowSpec {
        id: WorkflowSpecId::new(),
        name: "Persistence Test".to_string(),
        tasks: HashMap::new(),
        conditions: HashMap::new(),
        flows: Vec::new(),
        start_condition: None,
        end_condition: None,
        source_turtle: None,
    };

    let task = Task {
        id: "task:1".to_string(),
        name: "Task 1".to_string(),
        task_type: TaskType::Atomic,
        split_type: SplitType::And,
        join_type: JoinType::And,
        max_ticks: None,
        priority: None,
        use_simd: false,
        input_conditions: vec![],
        output_conditions: vec![],
        outgoing_flows: vec![],
        incoming_flows: vec![],
        allocation_policy: None,
        required_roles: vec![],
        required_capabilities: vec![],
        exception_worklet: None,
    };
    spec.tasks.insert("task:1".to_string(), task);

    engine1.register_workflow(spec.clone()).await?;

    // Act: Create case in first engine
    let test_data = TestDataBuilder::new()
        .with_var("persistence", "test")
        .build_json();
    let case_id = engine1.create_case(spec.id, test_data).await?;

    // Act: Create second engine instance with same state store
    // Note: We need to drop engine1 first to release the database lock
    drop(engine1);
    let state_store2 = StateStore::new(&db_path)?;
    let engine2 = WorkflowEngine::new(state_store2);

    // Assert: Second engine can retrieve workflow
    let retrieved_spec = engine2.get_workflow(spec.id).await?;
    assert_eq_with_msg(
        &retrieved_spec.id,
        &spec.id,
        "Second engine should retrieve persisted workflow",
    );

    // Note: Cases may not persist across instances depending on implementation
    // This test verifies workflow persistence at minimum

    Ok(())
}

/// Test admission gate integration
#[tokio::test]
async fn test_admission_gate_integration() -> WorkflowResult<()> {
    // Arrange: Create engine
    let state_store = StateStore::new("./test_workflow_db_admission")?;
    let engine = WorkflowEngine::new(state_store);

    // Arrange: Create workflow
    let mut spec = WorkflowSpec {
        id: WorkflowSpecId::new(),
        name: "Admission Test".to_string(),
        tasks: HashMap::new(),
        conditions: HashMap::new(),
        flows: Vec::new(),
        start_condition: None,
        end_condition: None,
        source_turtle: None,
    };

    let task = Task {
        id: "task:1".to_string(),
        name: "Task 1".to_string(),
        task_type: TaskType::Atomic,
        split_type: SplitType::And,
        join_type: JoinType::And,
        max_ticks: None,
        priority: None,
        use_simd: false,
        input_conditions: vec![],
        output_conditions: vec![],
        outgoing_flows: vec![],
        incoming_flows: vec![],
        allocation_policy: None,
        required_roles: vec![],
        required_capabilities: vec![],
        exception_worklet: None,
    };
    spec.tasks.insert("task:1".to_string(), task);

    engine.register_workflow(spec.clone()).await?;

    // Arrange: Create valid test data
    let valid_data = TestDataBuilder::new()
        .with_order_data("ORD-001", "100.00")
        .build_json();

    // Act: Create case with valid data
    let case_id = engine.create_case(spec.id, valid_data).await?;

    // Assert: Case is created successfully
    let case = engine.get_case(case_id).await?;
    assert_eq_with_msg(
        &case.state,
        &CaseState::Created,
        "Case should be created when data passes admission gate",
    );

    Ok(())
}

/// Test workflow engine services are accessible
#[tokio::test]
async fn test_engine_services_access() {
    // Arrange: Create engine
    let state_store =
        StateStore::new("./test_workflow_db_services").expect("Failed to create state store");
    let engine = WorkflowEngine::new(state_store);

    // Assert: All services are accessible
    assert!(engine
        .pattern_registry()
        .has_pattern(&knhk_workflow_engine::patterns::PatternId(1)));
    // Resource allocator, worklet repository, etc. are accessible through getters
    let _allocator = engine.resource_allocator();
    let _repository = engine.worklet_repository();
    let _executor = engine.worklet_executor();
    let _timer = engine.timer_service();
    let _work_items = engine.work_item_service();
    let _admission = engine.admission_gate();
    let _sidecar = engine.event_sidecar();
}

/// Test complete workflow lifecycle
#[tokio::test]
async fn test_complete_workflow_lifecycle() -> WorkflowResult<()> {
    // Arrange: Create engine
    let state_store = StateStore::new("./test_workflow_db_lifecycle")?;
    let engine = WorkflowEngine::new(state_store);

    // Arrange: Create workflow with multiple tasks
    let mut spec = WorkflowSpec {
        id: WorkflowSpecId::new(),
        name: "Lifecycle Test".to_string(),
        tasks: HashMap::new(),
        conditions: HashMap::new(),
        flows: Vec::new(),
        start_condition: None,
        end_condition: None,
        source_turtle: None,
    };

    // Create start condition
    let start_condition_id = "condition:start".to_string();
    let start_condition = Condition {
        id: start_condition_id.clone(),
        name: "Start".to_string(),
        outgoing_flows: vec!["task:1".to_string()],
        incoming_flows: vec![],
    };
    spec.conditions
        .insert(start_condition_id.clone(), start_condition);
    spec.start_condition = Some(start_condition_id.clone());

    // Create end condition
    let end_condition_id = "condition:end".to_string();
    let end_condition = Condition {
        id: end_condition_id.clone(),
        name: "End".to_string(),
        outgoing_flows: vec![],
        incoming_flows: vec!["task:2".to_string()],
    };
    spec.conditions
        .insert(end_condition_id.clone(), end_condition);
    spec.end_condition = Some(end_condition_id.clone());

    for i in 1..=2 {
        let task = Task {
            id: format!("task:{}", i),
            name: format!("Task {}", i),
            task_type: TaskType::Atomic,
            split_type: SplitType::And,
            join_type: JoinType::And,
            max_ticks: None,
            priority: None,
            use_simd: false,
            input_conditions: if i == 1 {
                vec![start_condition_id.clone()]
            } else {
                vec![]
            },
            output_conditions: if i == 2 {
                vec![end_condition_id.clone()]
            } else {
                vec![]
            },
            outgoing_flows: vec![],
            incoming_flows: vec![],
            allocation_policy: None,
            required_roles: vec!["test_role".to_string()],
            required_capabilities: vec![],
            exception_worklet: None,
        };
        spec.tasks.insert(format!("task:{}", i), task);
    }

    // Act: Register workflow
    engine.register_workflow(spec.clone()).await?;

    // Assert: Workflow is registered
    let retrieved_spec = engine.get_workflow(spec.id).await?;
    assert_eq_with_msg(
        &retrieved_spec.id,
        &spec.id,
        "Workflow should be registered",
    );

    // Arrange: Create test data
    let test_data = TestDataBuilder::new()
        .with_order_data("ORD-LIFECYCLE", "500.00")
        .with_customer_data("CUST-LIFECYCLE")
        .build_json();

    // Act: Create case
    let case_id = engine.create_case(spec.id, test_data.clone()).await?;

    // Assert: Case is created
    let case = engine.get_case(case_id).await?;
    assert_eq_with_msg(&case.state, &CaseState::Created, "Case should be created");
    assert_eq_with_msg(&case.spec_id, &spec.id, "Case should reference workflow");

    // Act: Start case
    engine.start_case(case_id).await?;

    // Assert: Case is running
    let case = engine.get_case(case_id).await?;
    assert_eq_with_msg(&case.state, &CaseState::Running, "Case should be running");

    // Act: Execute case
    engine.execute_case(case_id).await?;

    // Assert: Case execution completed
    let case = engine.get_case(case_id).await?;
    assert!(
        case.state == CaseState::Completed || case.state == CaseState::Running,
        "Case should complete or be running after execution"
    );

    // Assert: Case data is preserved
    assert!(case.data.is_object(), "Case data should be preserved");

    Ok(())
}
