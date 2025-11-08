// Initialize engine
let state_store = StateStore::new("./workflow_db")?;
let engine = Arc::new(WorkflowEngine::new(state_store));

// Parse and register workflow
let mut parser = WorkflowParser::new()?;
let spec = parser.parse_file("workflow.ttl")?;
engine.register_workflow(spec.clone()).await?;

// Create and execute case
let case_id = engine.create_case(spec.id, data).await?;
engine.start_case(case_id).await?;
engine.execute_case(case_id).await?;

// Get case status
let case = engine.get_case(case_id).await?;
println!("State: {:?}", case.state);