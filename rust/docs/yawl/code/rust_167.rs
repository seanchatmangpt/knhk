use knhk_workflow_engine::{WorkflowEngine, WorkflowParser, StateStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state_store = StateStore::new("./workflow_db")?;
    let engine = Arc::new(WorkflowEngine::new(state_store));
    
    // Parse approval workflow
    let mut parser = WorkflowParser::new()?;
    let spec = parser.parse_file("approval_workflow.ttl")?;
    
    // Register workflow
    engine.register_workflow(spec.clone()).await?;
    
    // Create case
    let case_id = engine.create_case(
        spec.id,
        serde_json::json!({
            "order_id": "12345",
            "amount": 5000.0,
            "requester": "user@example.com"
        })
    ).await?;
    
    // Start and execute
    engine.start_case(case_id).await?;
    engine.execute_case(case_id).await?;
    
    // Check status
    let case = engine.get_case(case_id).await?;
    println!("Case state: {:?}", case.state);
    
    Ok(())
}