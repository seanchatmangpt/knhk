// Workflow with potential deadlock
let spec = WorkflowSpec {
    // ... tasks with circular dependencies ...
};

match engine.register_workflow(spec).await {
    Ok(_) => println!("Workflow registered"),
    Err(e) => println!("Deadlock detected: {}", e),
}