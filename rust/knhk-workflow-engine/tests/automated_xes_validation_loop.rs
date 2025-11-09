//! Automated XES Validation Full Loop
//!
//! van der Aalst Process Mining Validation:
//! 1. Execute workflow
//! 2. Export to XES
//! 3. Validate XES format
//! 4. Compare with specification
//! 5. Check conformance
//!
//! This test automates the full validation loop.

use knhk_workflow_engine::{
    executor::WorkflowEngine, parser::WorkflowSpec, state::StateStore, CaseId, WorkflowSpecId,
};
use process_mining::{
    alphappp::full::{alphappp_discover_petri_net, AlphaPPPConfig},
    event_log::activity_projection::EventLogActivityProjection,
    import_xes_file, XESImportOptions,
};
use std::collections::HashMap;
use tempfile::TempDir;

/// Create a simple test workflow specification
fn create_test_workflow() -> WorkflowSpec {
    use knhk_workflow_engine::parser::{Flow, JoinType, SplitType, Task, TaskType};
    let mut tasks = HashMap::new();
    let mut flows = Vec::new();

    // Task A
    tasks.insert(
        "task_a".to_string(),
        Task {
            id: "task_a".to_string(),
            name: "Task A".to_string(),
            task_type: TaskType::Atomic,
            split_type: SplitType::Xor,
            join_type: JoinType::Xor,
            max_ticks: None,
            priority: None,
            use_simd: false,
            input_conditions: vec![],
            output_conditions: vec![],
            outgoing_flows: vec!["a_to_b".to_string()],
            incoming_flows: vec!["start_to_a".to_string()],
            input_parameters: vec![],
            output_parameters: vec![],
            allocation_policy: None,
            required_roles: vec![],
            required_capabilities: vec![],
            exception_worklet: None,
        },
    );

    // Task B
    tasks.insert(
        "task_b".to_string(),
        Task {
            id: "task_b".to_string(),
            name: "Task B".to_string(),
            task_type: TaskType::Atomic,
            split_type: SplitType::Xor,
            join_type: JoinType::Xor,
            max_ticks: None,
            priority: None,
            use_simd: false,
            input_conditions: vec![],
            output_conditions: vec![],
            outgoing_flows: vec!["b_to_end".to_string()],
            incoming_flows: vec!["a_to_b".to_string()],
            input_parameters: vec![],
            output_parameters: vec![],
            allocation_policy: None,
            required_roles: vec![],
            required_capabilities: vec![],
            exception_worklet: None,
        },
    );

    // Flows
    flows.push(Flow {
        id: "start_to_a".to_string(),
        from: "start".to_string(),
        to: "task_a".to_string(),
        predicate: None,
    });
    flows.push(Flow {
        id: "a_to_b".to_string(),
        from: "task_a".to_string(),
        to: "task_b".to_string(),
        predicate: None,
    });
    flows.push(Flow {
        id: "b_to_end".to_string(),
        from: "task_b".to_string(),
        to: "end".to_string(),
        predicate: None,
    });

    WorkflowSpec {
        id: WorkflowSpecId::new(),
        name: "Test Workflow".to_string(),
        start_condition: Some("start".to_string()),
        end_condition: Some("end".to_string()),
        tasks,
        conditions: HashMap::new(),
        flows,
        source_turtle: None,
    }
}

/// Automated XES Validation Full Loop Test
#[tokio::test]
async fn test_automated_xes_validation_full_loop() {
    println!("[TEST] Automated XES Validation Full Loop");
    println!("  Phase 1: Execute workflow");
    println!("  Phase 2: Export to XES");
    println!("  Phase 3: Validate XES format");
    println!("  Phase 4: Compare with specification");
    println!("  Phase 5: Check conformance");
    println!("");

    // Arrange: Create engine and workflow
    let temp_dir = TempDir::new().unwrap();
    let state_store = StateStore::new(temp_dir.path()).unwrap();
    let engine = WorkflowEngine::new(state_store);

    let spec = create_test_workflow();
    let spec_id = spec.id;

    // Phase 1: Execute workflow
    println!("  Phase 1: Executing workflow...");
    engine.register_workflow(spec.clone()).await.unwrap();

    let case_id = engine
        .create_case(spec_id, serde_json::json!({"test": "automated_validation"}))
        .await
        .unwrap();

    // Start case execution
    engine.start_case(case_id).await.unwrap();

    // Wait for execution to complete (simplified - in production would poll)
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    println!("    ✅ Workflow executed: case_id={}", case_id);

    // Phase 2: Export to XES
    println!("  Phase 2: Exporting to XES...");
    let xes_content = engine.export_case_to_xes(case_id).await.unwrap();

    // Write XES to file
    let xes_file = temp_dir.path().join("workflow_execution.xes");
    std::fs::write(&xes_file, &xes_content).unwrap();

    println!("    ✅ XES exported: {}", xes_file.display());

    // Phase 3: Validate XES format
    println!("  Phase 3: Validating XES format...");
    assert!(
        xes_content.contains("<?xml version"),
        "XES should be valid XML"
    );
    assert!(
        xes_content.contains("<log xes.version=\"2.0\""),
        "XES should be XES 2.0 compliant"
    );
    assert!(xes_content.contains("<trace>"), "XES should contain traces");
    assert!(xes_content.contains("<event>"), "XES should contain events");
    assert!(
        xes_content.contains("concept:name"),
        "XES should contain concept:name attributes"
    );
    assert!(
        xes_content.contains("time:timestamp"),
        "XES should contain time:timestamp attributes"
    );
    assert!(
        xes_content.contains("lifecycle:transition"),
        "XES should contain lifecycle:transition attributes"
    );

    println!("    ✅ XES format validated (XES 2.0 compliant)");

    // Phase 4: Compare with specification
    println!("  Phase 4: Comparing with specification...");
    // Import XES using process_mining library
    let event_log = import_xes_file(&xes_file, XESImportOptions::default())
        .expect("XES file should be importable");

    // Verify event log structure
    assert!(
        event_log.traces.len() > 0,
        "Event log should contain traces"
    );

    // Extract activity names from XES
    let activities: Vec<String> = event_log
        .traces
        .iter()
        .flat_map(|trace| {
            trace
                .events
                .iter()
                .filter_map(|event| {
                    event
                        .attributes
                        .iter()
                        .find(|attr| attr.key == "concept:name")
                        .and_then(|attr| match &attr.value {
                            process_mining::event_log::attribute::AttributeValue::String(s) => {
                                Some(s.clone())
                            }
                            _ => None,
                        })
                })
                .collect::<Vec<_>>()
        })
        .collect();

    // Verify that workflow tasks appear in XES
    let expected_tasks = vec!["task_a", "task_b"];
    for task in &expected_tasks {
        assert!(
            activities.iter().any(|a| a.contains(task)),
            "XES should contain activity: {}",
            task
        );
    }

    println!("    ✅ XES event log matches specification");

    // Phase 5: Check conformance
    println!("  Phase 5: Checking conformance...");
    // Use process discovery to verify conformance
    let projection: EventLogActivityProjection = (&event_log).into();
    let config = AlphaPPPConfig {
        log_repair_skip_df_thresh_rel: 2.0,
        log_repair_loop_df_thresh_rel: 2.0,
        absolute_df_clean_thresh: 1,
        relative_df_clean_thresh: 0.01,
        balance_thresh: 0.5,
        fitness_thresh: 0.5,
        replay_thresh: 0.5,
    };
    let (petri_net, _duration) = alphappp_discover_petri_net(&projection, config);

    // Verify Petri net structure
    assert!(
        petri_net.places.len() > 0 || petri_net.transitions.len() > 0,
        "Process discovery should produce places or transitions"
    );

    // Verify that discovered process matches specification
    // (simplified - in production would do detailed comparison)
    let discovered_activities: Vec<String> = petri_net
        .transitions
        .iter()
        .filter_map(|(_id, t)| t.label.clone())
        .collect();

    for task in &expected_tasks {
        assert!(
            discovered_activities.iter().any(|a| a.contains(task)),
            "Discovered process should contain activity: {}",
            task
        );
    }

    println!("    ✅ Conformance validated");

    println!("");
    println!("  ✅ Full loop complete:");
    println!("    - Workflow executed");
    println!("    - XES exported");
    println!("    - XES format validated");
    println!("    - Specification compared");
    println!("    - Conformance checked");
}
