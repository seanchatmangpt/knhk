//! Complete example: Execute a workflow from Turtle definition
//!
//! This example demonstrates Covenant 1: **Turtle Is Definition and Cause**.
//!
//! The workflow is completely defined in Turtle, and the runtime
//! executes exactly what the Turtle defines - no more, no less.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example execute_workflow --features rdf
//! ```

use knhk_workflow_engine::executor::{
    WorkflowLoader, WorkflowRuntime, WorkflowState, TaskExecutor,
    TaskDefinition, TaskResult,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Example task executor that simulates work
struct ExampleExecutor;

#[async_trait::async_trait]
impl TaskExecutor for ExampleExecutor {
    async fn execute(
        &self,
        task: &TaskDefinition,
        input: HashMap<String, serde_json::Value>,
    ) -> knhk_workflow_engine::error::WorkflowResult<TaskResult> {
        println!("📋 Executing task: {} ({})", task.name, task.id);
        println!("   Execution mode: {:?}", task.execution_mode);
        if let Some(ref split) = task.split_type {
            println!("   Split type: {:?}", split);
        }
        if let Some(ref join) = task.join_type {
            println!("   Join type: {:?}", join);
        }

        // Simulate some work
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        println!("✅ Task completed: {}", task.name);

        Ok(TaskResult {
            task_id: task.id.clone(),
            success: true,
            output: input, // Pass through input as output
            error: None,
            duration: Some(std::time::Duration::from_millis(100)),
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 Self-Executing YAWL Workflows Example");
    println!("==========================================\n");

    // Example 1: Simple Sequence Workflow
    println!("📝 Example 1: Simple Sequence Workflow");
    println!("---------------------------------------");
    execute_sequence_workflow().await?;

    println!("\n");

    // Example 2: Parallel Workflow
    println!("📝 Example 2: Parallel Split and Join Workflow");
    println!("---------------------------------------------");
    execute_parallel_workflow().await?;

    println!("\n");

    // Example 3: Multi-Choice (OR Split) Workflow
    println!("📝 Example 3: Multi-Choice (OR Split) Workflow");
    println!("----------------------------------------------");
    execute_multichoice_workflow().await?;

    println!("\n🎉 All examples completed successfully!");

    Ok(())
}

/// Execute a simple sequence workflow
async fn execute_sequence_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let turtle = r#"
        @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        <http://example.org/sequence_workflow> a yawl:Specification ;
            rdfs:label "Sequential Process Workflow" ;
            rdfs:comment "Pattern 1: Sequence - A -> B -> C" ;
            yawl:inputCondition <http://example.org/sequence/input> ;
            yawl:outputCondition <http://example.org/sequence/output> .

        <http://example.org/sequence/receive_order> a yawl:Task ;
            rdfs:label "Receive Order" .

        <http://example.org/sequence/process_order> a yawl:Task ;
            rdfs:label "Process Order" .

        <http://example.org/sequence/ship_order> a yawl:Task ;
            rdfs:label "Ship Order" .

        <http://example.org/sequence/flow1> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/sequence/input> ;
            yawl:flowsInto <http://example.org/sequence/receive_order> .

        <http://example.org/sequence/flow2> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/sequence/receive_order> ;
            yawl:flowsInto <http://example.org/sequence/process_order> .

        <http://example.org/sequence/flow3> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/sequence/process_order> ;
            yawl:flowsInto <http://example.org/sequence/ship_order> .

        <http://example.org/sequence/flow4> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/sequence/ship_order> ;
            yawl:flowsInto <http://example.org/sequence/output> .
    "#;

    let mut loader = WorkflowLoader::new()?;
    let definition = loader.load_turtle(turtle)?;

    println!("📄 Loaded workflow: {}", definition.name);
    println!("   Tasks: {}", definition.tasks.len());
    println!("   Flows: {}", definition.flows.len());

    let runtime = WorkflowRuntime::new(definition)
        .with_executor(Arc::new(ExampleExecutor));

    let final_state = runtime.run().await?;

    println!("\n✅ Workflow completed!");
    println!("   State: {:?}", final_state.state);
    println!("   Tasks completed: {}", final_state.completed_tasks.len());
    if let Some(duration) = final_state.end_time.and_then(|end| {
        final_state.start_time.map(|start| end - start)
    }) {
        println!("   Duration: {:?}", duration);
    }

    Ok(())
}

/// Execute a parallel workflow (AND split + AND join)
async fn execute_parallel_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let turtle = r#"
        @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        <http://example.org/parallel_workflow> a yawl:Specification ;
            rdfs:label "Parallel Approval Workflow" ;
            rdfs:comment "Patterns 2+3: Parallel Split + Synchronization" ;
            yawl:inputCondition <http://example.org/parallel/input> ;
            yawl:outputCondition <http://example.org/parallel/output> .

        <http://example.org/parallel/receive_request> a yawl:Task ;
            rdfs:label "Receive Request" ;
            yawl:split <http://www.yawlfoundation.org/yawlschema#AND> .

        <http://example.org/parallel/technical_review> a yawl:Task ;
            rdfs:label "Technical Review" .

        <http://example.org/parallel/budget_approval> a yawl:Task ;
            rdfs:label "Budget Approval" .

        <http://example.org/parallel/finalize> a yawl:Task ;
            rdfs:label "Finalize Decision" ;
            yawl:join <http://www.yawlfoundation.org/yawlschema#AND> .

        <http://example.org/parallel/flow1> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/parallel/input> ;
            yawl:flowsInto <http://example.org/parallel/receive_request> .

        <http://example.org/parallel/flow2> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/parallel/receive_request> ;
            yawl:flowsInto <http://example.org/parallel/technical_review> .

        <http://example.org/parallel/flow3> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/parallel/receive_request> ;
            yawl:flowsInto <http://example.org/parallel/budget_approval> .

        <http://example.org/parallel/flow4> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/parallel/technical_review> ;
            yawl:flowsInto <http://example.org/parallel/finalize> .

        <http://example.org/parallel/flow5> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/parallel/budget_approval> ;
            yawl:flowsInto <http://example.org/parallel/finalize> .

        <http://example.org/parallel/flow6> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/parallel/finalize> ;
            yawl:flowsInto <http://example.org/parallel/output> .
    "#;

    let mut loader = WorkflowLoader::new()?;
    let definition = loader.load_turtle(turtle)?;

    println!("📄 Loaded workflow: {}", definition.name);
    println!("   Tasks: {}", definition.tasks.len());

    let runtime = WorkflowRuntime::new(definition)
        .with_executor(Arc::new(ExampleExecutor));

    let final_state = runtime.run().await?;

    println!("\n✅ Workflow completed!");
    println!("   State: {:?}", final_state.state);
    println!("   Tasks completed: {}", final_state.completed_tasks.len());

    Ok(())
}

/// Execute a multi-choice workflow (OR split)
async fn execute_multichoice_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let turtle = r#"
        @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        <http://example.org/multichoice_workflow> a yawl:Specification ;
            rdfs:label "Multi-Choice Notification Workflow" ;
            rdfs:comment "Pattern 6: Multi-Choice (OR Split)" ;
            yawl:inputCondition <http://example.org/multichoice/input> ;
            yawl:outputCondition <http://example.org/multichoice/output> .

        <http://example.org/multichoice/receive_alert> a yawl:Task ;
            rdfs:label "Receive Alert" ;
            yawl:split <http://www.yawlfoundation.org/yawlschema#OR> .

        <http://example.org/multichoice/notify_email> a yawl:Task ;
            rdfs:label "Send Email Notification" .

        <http://example.org/multichoice/notify_sms> a yawl:Task ;
            rdfs:label "Send SMS Notification" .

        <http://example.org/multichoice/notify_push> a yawl:Task ;
            rdfs:label "Send Push Notification" .

        <http://example.org/multichoice/merge> a yawl:Task ;
            rdfs:label "Complete Notification" ;
            yawl:join <http://www.yawlfoundation.org/yawlschema#OR> .

        <http://example.org/multichoice/flow1> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/multichoice/input> ;
            yawl:flowsInto <http://example.org/multichoice/receive_alert> .

        <http://example.org/multichoice/flow2> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/multichoice/receive_alert> ;
            yawl:flowsInto <http://example.org/multichoice/notify_email> .

        <http://example.org/multichoice/flow3> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/multichoice/receive_alert> ;
            yawl:flowsInto <http://example.org/multichoice/notify_sms> .

        <http://example.org/multichoice/flow4> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/multichoice/receive_alert> ;
            yawl:flowsInto <http://example.org/multichoice/notify_push> .

        <http://example.org/multichoice/flow5> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/multichoice/notify_email> ;
            yawl:flowsInto <http://example.org/multichoice/merge> .

        <http://example.org/multichoice/flow6> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/multichoice/notify_sms> ;
            yawl:flowsInto <http://example.org/multichoice/merge> .

        <http://example.org/multichoice/flow7> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/multichoice/notify_push> ;
            yawl:flowsInto <http://example.org/multichoice/merge> .

        <http://example.org/multichoice/flow8> a yawl:Flow ;
            yawl:flowsFrom <http://example.org/multichoice/merge> ;
            yawl:flowsInto <http://example.org/multichoice/output> .
    "#;

    let mut loader = WorkflowLoader::new()?;
    let definition = loader.load_turtle(turtle)?;

    println!("📄 Loaded workflow: {}", definition.name);
    println!("   Tasks: {}", definition.tasks.len());

    let runtime = WorkflowRuntime::new(definition)
        .with_executor(Arc::new(ExampleExecutor));

    let final_state = runtime.run().await?;

    println!("\n✅ Workflow completed!");
    println!("   State: {:?}", final_state.state);
    println!("   Tasks completed: {}", final_state.completed_tasks.len());

    Ok(())
}
