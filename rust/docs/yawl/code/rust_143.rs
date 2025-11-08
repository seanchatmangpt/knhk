fn create_approval_workflow() -> WorkflowSpec {
    WorkflowSpecBuilder::new("Approval Workflow")
        .add_task(
            TaskBuilder::new("task:approve", "Approve")
                .with_max_ticks(8)
                .build()
        )
        .build()
}