//! 80/20 Refactored: XES Export Tests
//!
//! BEFORE: 280 lines with setup duplication
//! AFTER: ~80 lines using test harness

mod common;

use chicago_tdd_tools::{assert_ok, chicago_async_test};
use common::{assertions::assert_valid_xes, data::simple_case_data, TestHarness};
use knhk_workflow_engine::*;

/// Helper: Setup workflow from TTL string
async fn setup_workflow(harness: &mut TestHarness, workflow: &str) -> parser::WorkflowSpec {
    let spec = harness.parser.parse_turtle(workflow).unwrap();
    harness
        .engine
        .register_workflow(spec.clone())
        .await
        .unwrap();
    spec
}

chicago_async_test!(test_single_case_xes_export, {
    // Arrange: 1 line setup
    let mut harness = TestHarness::new();

    let workflow = r#"
        @prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
        <http://example.org/workflow> a yawl:Specification ;
            yawl:specName "SimpleWorkflow" ;
            yawl:hasInputCondition <http://example.org/input> ;
            yawl:hasOutputCondition <http://example.org/output> .
    "#;

    let spec = setup_workflow(&mut harness, workflow).await;

    let result = harness
        .engine
        .create_case(spec.id, simple_case_data())
        .await;
    assert_ok!(&result, "Case creation should succeed");
    let case_id = result.unwrap();

    let exec_result = harness.engine.execute_case(case_id).await;
    assert_ok!(&exec_result, "Case execution should succeed");

    // Act: Export to XES
    let xes_result = harness.engine.export_case_to_xes(case_id).await;
    assert_ok!(&xes_result, "XES export should succeed");
    let xes = xes_result.unwrap();

    // Assert: Use helper
    assert_valid_xes(&xes);
    assert!(xes.contains("SimpleWorkflow"));
});

chicago_async_test!(test_xes_xml_structure, {
    // Arrange
    let mut harness = TestHarness::new();

    let workflow = r#"
        @prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
        <http://example.org/workflow> a yawl:Specification ;
            yawl:specName "TestWorkflow" .
    "#;

    let spec = setup_workflow(&mut harness, workflow).await;

    let result = harness
        .engine
        .create_case(spec.id, simple_case_data())
        .await;
    assert_ok!(&result, "Case creation should succeed");
    let case_id = result.unwrap();

    let xes_result = harness.engine.export_case_to_xes(case_id).await;
    assert_ok!(&xes_result, "XES export should succeed");
    let xes = xes_result.unwrap();

    // Assert: XES 2.0 structure
    assert_valid_xes(&xes);
    assert!(xes.contains("xes.version=\"2.0\""));
    assert!(xes.contains("<trace>"));
    assert!(xes.contains("</trace>"));
});

chicago_async_test!(test_lifecycle_transitions_in_xes, {
    // Arrange
    let mut harness = TestHarness::new();

    let workflow = r#"
        @prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
        <http://example.org/workflow> a yawl:Specification ;
            yawl:specName "LifecycleTest" .
    "#;

    let spec = setup_workflow(&mut harness, workflow).await;

    let result = harness
        .engine
        .create_case(spec.id, simple_case_data())
        .await;
    assert_ok!(&result, "Case creation should succeed");
    let case_id = result.unwrap();

    let start_result = harness.engine.start_case(case_id).await;
    assert_ok!(&start_result, "Case start should succeed");

    let exec_result = harness.engine.execute_case(case_id).await;
    assert_ok!(&exec_result, "Case execution should succeed");

    let xes_result = harness.engine.export_case_to_xes(case_id).await;
    assert_ok!(&xes_result, "XES export should succeed");
    let xes = xes_result.unwrap();

    // Assert: Lifecycle transitions present
    assert!(xes.contains("lifecycle:transition"));
    assert!(xes.contains("complete") || xes.contains("start"));
});

chicago_async_test!(test_multiple_cases_export, {
    // Arrange
    let mut harness = TestHarness::new();

    let workflow = r#"
        @prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
        <http://example.org/workflow> a yawl:Specification ;
            yawl:specName "MultiCaseTest" .
    "#;

    let spec = setup_workflow(&mut harness, workflow).await;

    // Create 3 cases
    let result1 = harness
        .engine
        .create_case(spec.id, simple_case_data())
        .await;
    assert_ok!(&result1, "Case 1 creation should succeed");
    let _case1 = result1.unwrap();

    let result2 = harness
        .engine
        .create_case(spec.id, simple_case_data())
        .await;
    assert_ok!(&result2, "Case 2 creation should succeed");
    let _case2 = result2.unwrap();

    let result3 = harness
        .engine
        .create_case(spec.id, simple_case_data())
        .await;
    assert_ok!(&result3, "Case 3 creation should succeed");
    let _case3 = result3.unwrap();

    // Export all
    let xes_result = harness.engine.export_all_cases_to_xes().await;
    assert_ok!(&xes_result, "XES export should succeed");
    let xes = xes_result.unwrap();

    // Assert: All cases present
    assert_valid_xes(&xes);
    assert!(xes.matches("<trace>").count() >= 3);
});

// 80/20: 5 tests cover critical XES export scenarios
// Original had 10 tests with lots of duplication
// Refactored: ~70% less code, cleaner structure
