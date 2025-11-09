# KNHK Workflow Engine - Quick Start Guide

**⚠️ This document has been consolidated. See the [80/20 Consolidated Guide](../../../docs/WORKFLOW_ENGINE.md) for the single source of truth.**

This file is kept for backward compatibility. All new documentation should reference the consolidated guide.

---

## Overview

Enterprise-grade workflow execution engine with YAWL compatibility, OTEL observability, and lockchain provenance.

**Key Features**:
- ✅ All 43 Van der Aalst workflow patterns
- ✅ YAWL compatibility (Turtle/RDF)
- ✅ REST/gRPC APIs
- ✅ OTEL integration
- ✅ Deadlock detection
- ✅ Resource allocation policies

**📖 [Go to Consolidated Guide](../../../docs/WORKFLOW_ENGINE.md)**

---

## Quick Start

### Installation

```toml
[dependencies]
knhk-workflow-engine = { path = "../knhk-workflow-engine" }
tokio = { version = "1", features = ["full"] }
```

### Basic Usage

```rust
use knhk_workflow_engine::{WorkflowEngine, WorkflowParser, StateStore};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create engine
    let state_store = StateStore::new("./workflow_db")?;
    let engine = Arc::new(WorkflowEngine::new(state_store));
    
    // Parse workflow from Turtle
    let mut parser = WorkflowParser::new()?;
    let spec = parser.parse_file("workflow.ttl")?;
    
    // Register workflow (validates for deadlocks)
    engine.register_workflow(spec.clone()).await?;
    
    // Create and execute case
    let case_id = engine.create_case(
        spec.id,
        serde_json::json!({"customer_id": "12345", "order_amount": 1000.0})
    ).await?;
    
    engine.start_case(case_id).await?;
    engine.execute_case(case_id).await?;
    
    // Get status
    let case = engine.get_case(case_id).await?;
    println!("Case state: {:?}", case.state);
    
    Ok(())
}
```

---

## Core API

### WorkflowEngine

```rust
impl WorkflowEngine {
    /// Register workflow (validates for deadlocks)
    pub async fn register_workflow(&self, spec: WorkflowSpec) -> WorkflowResult<()>;
    
    /// Create case with input data
    pub async fn create_case(
        &self,
        spec_id: WorkflowSpecId,
        data: serde_json::Value
    ) -> WorkflowResult<CaseId>;
    
    /// Start case execution
    pub async fn start_case(&self, case_id: CaseId) -> WorkflowResult<()>;
    
    /// Execute case
    pub async fn execute_case(&self, case_id: CaseId) -> WorkflowResult<()>;
    
    /// Get case status
    pub async fn get_case(&self, case_id: CaseId) -> WorkflowResult<Case>;
    
    /// Cancel case
    pub async fn cancel_case(&self, case_id: CaseId) -> WorkflowResult<()>;
}
```

---

## Workflow Definition (Turtle/RDF)

### Simple Sequence Workflow

```turtle
@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

<workflow:simple-sequence> a yawl:WorkflowSpec ;
    yawl:name "Simple Sequence" ;
    yawl:startCondition <condition:start> ;
    yawl:endCondition <condition:end> ;
    yawl:task <task:task1>, <task:task2> ;
    yawl:condition <condition:start>, <condition:end>, <condition:intermediate> .

<task:task1> a yawl:Task ;
    yawl:name "Task 1" ;
    yawl:splitType yawl:AND ;
    yawl:joinType yawl:AND ;
    yawl:outputCondition <condition:intermediate> .

<task:task2> a yawl:Task ;
    yawl:name "Task 2" ;
    yawl:splitType yawl:AND ;
    yawl:joinType yawl:AND ;
    yawl:inputCondition <condition:intermediate> .

<condition:start> a yawl:Condition ; yawl:name "Start" .
<condition:intermediate> a yawl:Condition ; yawl:name "Intermediate" .
<condition:end> a yawl:Condition ; yawl:name "End" .
```

---

## Critical Patterns (80/20 Focus)

### Pattern 1: Sequence
Sequential task execution.

### Pattern 2: Parallel Split (AND-split)
All branches execute in parallel.

### Pattern 3: Synchronization (AND-join)
Wait for all branches to complete.

### Pattern 4: Exclusive Choice (XOR-split)
Exactly one branch executes.

### Pattern 5: Simple Merge (XOR-join)
Wait for one branch to complete.

**Note**: Full pattern registry supports all 43 Van der Aalst patterns. See `PatternRegistry` for complete list.

---

## Resource Allocation

### Four-Eyes Principle (Dual Approval)

```rust
use knhk_workflow_engine::resource::{AllocationPolicy, AllocationRequest};

let request = AllocationRequest {
    task_id: "task:approve".to_string(),
    spec_id,
    required_roles: vec!["approver".to_string()],
    policy: AllocationPolicy::FourEyes,
    priority: 100,
};

let allocation = engine.resource_allocator().allocate(request).await?;
```

### Other Policies
- `AllocationPolicy::RoundRobin` - Even distribution
- `AllocationPolicy::ShortestQueue` - Least busy resource
- `AllocationPolicy::RoleBased` - Role matching
- `AllocationPolicy::CapabilityBased` - Capability scoring

---

## REST API

### Register Workflow

**POST** `/workflows`

```json
{
  "spec": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "My Workflow",
    "tasks": { ... },
    "conditions": { ... }
  }
}
```

### Create Case

**POST** `/cases`

```json
{
  "spec_id": "550e8400-e29b-41d4-a716-446655440000",
  "data": {"customer_id": "12345"}
}
```

### Get Case

**GET** `/cases/{id}`

Returns case status and data.

### Cancel Case

**POST** `/cases/{id}/cancel`

---

## Deadlock Detection

Automatic validation during workflow registration:

```rust
// Returns error if deadlock detected
engine.register_workflow(spec).await?;
```

Manual validation:

```rust
use knhk_workflow_engine::validation::DeadlockDetector;

let detector = DeadlockDetector;
let result = detector.validate(&spec)?;
if !result.cycles.is_empty() {
    println!("Deadlock detected: {:?}", result.cycles);
}
```

---

## Integration

### OTEL Observability

```rust
use knhk_workflow_engine::integration::otel::OtelIntegration;

let otel = OtelIntegration::new()?;
// Spans automatically created for workflow execution
```

### Lockchain Provenance

```rust
use knhk_workflow_engine::integration::lockchain::LockchainIntegration;

let lockchain = LockchainIntegration::new()?;
// Execution receipts automatically recorded
```

---

## Troubleshooting

### Deadlock Detection Error

**Problem**: Workflow registration fails with deadlock error.

**Solution**: Review workflow structure for cycles.

```rust
let detector = DeadlockDetector;
let result = detector.validate(&spec)?;
println!("Cycles: {:?}", result.cycles);
```

### Resource Allocation Failure

**Problem**: Task execution fails due to resource allocation.

**Solution**: Check resource availability and policy requirements.

```rust
let available = engine.resource_allocator()
    .is_available(resource_id)
    .await?;
```

### Case Not Found

**Problem**: `CaseNotFound` error.

**Solution**: Verify case ID is correct and case exists.

```rust
match engine.get_case(case_id).await {
    Ok(case) => println!("Case: {:?}", case),
    Err(e) => println!("Error: {}", e),
}
```

---

## Performance

- Hot path operations: ≤8 ticks (Chatman Constant)
- Supports 1000+ concurrent cases
- O(1) pattern registry lookup
- Optimized state store queries

---

## Additional Resources

- **Full Pattern Reference**: See `PatternRegistry` for all 43 patterns
- **Advanced Features**: See `docs/INNOVATIONS.md` for visualizer and templates
- **Deployment**: See `docs/FORTUNE5_DEPLOYMENT.md` for enterprise deployment
- **Testing**: See `docs/CHICAGO_TDD_WORKFLOW_ENGINE_TESTS.md` for testing guide

---

**License**: MIT
