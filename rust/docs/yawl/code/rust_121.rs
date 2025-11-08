use knhk_workflow_engine::testing::chicago_tdd::WorkflowSpecBuilder;

let spec = WorkflowSpecBuilder::new("Test Workflow")
    .with_start_condition("condition:start")
    .with_end_condition("condition:end")
    .add_task(create_test_task())
    .build();