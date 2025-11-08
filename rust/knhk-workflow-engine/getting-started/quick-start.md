# Quick Start

Get started with the KNHK Workflow Engine in 5 minutes!

## Prerequisites

- Rust 1.70+ installed
- Basic understanding of Rust

## Step 1: Add Dependency

Add to your `Cargo.toml`:

```toml
[dependencies]
knhk-workflow-engine = "1.0.0"
tokio = { version = "1.35", features = ["full"] }
serde_json = "1.0"
```

## Step 2: Create a Workflow

Create a simple workflow specification:

```rust
use knhk_workflow_engine::parser::{WorkflowSpec, WorkflowSpecId, Task, TaskType};
use std::collections::HashMap;

let mut tasks = HashMap::new();
tasks.insert("task:1".to_string(), Task {
    name: "Task 1".to_string(),
    task_type: TaskType::Atomic,
    max_ticks: Some(8),
    // ... other fields
});

let spec = WorkflowSpec {
    id: WorkflowSpecId::new(),
    name: "My First Workflow".to_string(),
    tasks,
    // ... other fields
};
```

## Step 3: Create Engine and Execute

```rust
use knhk_workflow_engine::{WorkflowEngine, StateStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create state store
    let state_store = StateStore::new("./workflow_db")?;
    
    // Create engine
    let engine = WorkflowEngine::new(state_store);
    
    // Register workflow
    let spec_id = engine.register_workflow(spec).await?;
    
    // Create case
    let case_id = engine.create_case(spec_id, serde_json::json!({})).await?;
    
    // Execute case
    engine.start_case(case_id).await?;
    engine.execute_case(case_id).await?;
    
    Ok(())
}
```

## Step 4: Run

```bash
cargo run
```

## Next Steps

- [Installation](installation.md) - Detailed setup instructions
- [Basic Concepts](basic-concepts.md) - Learn workflow concepts
- [Workflow Patterns](core/patterns.md) - Explore workflow patterns

