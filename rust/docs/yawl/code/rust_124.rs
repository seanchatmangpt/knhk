use knhk_workflow_engine::testing::chicago_tdd::*;

let role = create_test_role("approver", "Approver");
let capability = create_test_capability("approval", "Approval", 100);
let resource = create_test_resource("User1", vec![role], vec![capability]);