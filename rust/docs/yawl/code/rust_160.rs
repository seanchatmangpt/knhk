// Deadlock validation happens automatically
engine.register_workflow(spec).await?;
// Returns error if deadlock detected