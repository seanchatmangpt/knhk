use knhk_workflow_engine::testing::chicago_tdd::create_test_worklet;

let worklet = create_test_worklet(
    "Exception Handler",
    vec!["resource_unavailable".to_string()]
);