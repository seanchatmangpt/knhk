use knhk_workflow_engine::{WorkflowEngine, WorkflowParser, StateStore};

// Create state store
let state_store = StateStore::new("./workflow_db")?;

// Create engine
let engine = WorkflowEngine::new(state_store);

// Parse workflow from Turtle
let mut parser = WorkflowParser::new()?;
let spec = parser.parse_file("workflow.ttl")?;

// Register workflow
engine.register_workflow(spec.clone()).await?;

// Create and execute case
let case_id = engine.create_case(spec.id, serde_json::json!({})).await?;
engine.start_case(case_id).await?;
engine.execute_case(case_id).await?;