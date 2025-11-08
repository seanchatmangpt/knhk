use knhk_workflow_engine::{WorkflowEngine, WorkflowParser, StateStore};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create state store
    let state_store = StateStore::new("./workflow_db")?;
    
    // Create engine
    let engine = Arc::new(WorkflowEngine::new(state_store));
    
    // Parse workflow from Turtle
    let mut parser = WorkflowParser::new()?;
    let spec = parser.parse_file("workflow.ttl")?;
    
    // Register workflow (validates for deadlocks)
    engine.register_workflow(spec.clone()).await?;
    
    // Create case with input data
    let case_id = engine.create_case(
        spec.id,
        serde_json::json!({
            "customer_id": "12345",
            "order_amount": 1000.0
        })
    ).await?;
    
    // Start case
    engine.start_case(case_id).await?;
    
    // Execute case
    engine.execute_case(case_id).await?;
    
    // Get case status
    let case = engine.get_case(case_id).await?;
    println!("Case state: {:?}", case.state);
    
    Ok(())
}