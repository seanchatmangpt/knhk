# Workflow Engine Quick Reference

## Common Operations

### Initialize Engine
```rust
let state_store = StateStore::new("./workflow_db")?;
let engine = Arc::new(WorkflowEngine::new(state_store));
```

### Parse and Register Workflow
```rust
let mut parser = WorkflowParser::new()?;
let spec = parser.parse_file("workflow.ttl")?;
engine.register_workflow(spec.clone()).await?;
```

### Create and Execute Case
```rust
let case_id = engine.create_case(spec.id, data).await?;
engine.start_case(case_id).await?;
engine.execute_case(case_id).await?;
```

### Get Case Status
```rust
let case = engine.get_case(case_id).await?;
println!("State: {:?}", case.state);
```

## REST API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/workflows` | Register workflow |
| GET | `/workflows/{id}` | Get workflow |
| POST | `/cases` | Create case |
| GET | `/cases/{id}` | Get case status |
| POST | `/cases/{id}/cancel` | Cancel case |
| GET | `/cases/{id}/history` | Get case history |

## Pattern IDs

### Basic Control Flow (1-5)
- `1` - Sequence
- `2` - Parallel Split
- `3` - Synchronization
- `4` - Exclusive Choice
- `5` - Simple Merge

### Advanced Branching (6-11)
- `6` - Multi-Choice
- `7` - Structured Synchronizing Merge
- `8` - Multi-Merge
- `9` - Discriminator
- `10` - Arbitrary Cycles
- `11` - Implicit Termination

### Multiple Instance (12-15)
- `12` - MI Without Synchronization
- `13` - MI With Synchronization
- `14` - MI With Design-Time Knowledge
- `15` - MI With Runtime Knowledge

### State-Based (16-18)
- `16` - Deferred Choice
- `17` - Interleaved Parallel Routing
- `18` - Milestone

### Cancellation (19-25)
- `19` - Cancel Activity
- `20` - Cancel Case
- `21` - Cancel Region
- `22-25` - MI Cancellation Patterns

### Advanced Control (26-39)
- `26` - Blocking Discriminator
- `27` - Cancelling Discriminator
- `28` - Structured Loop
- `29` - Recursion
- `30-39` - Advanced Patterns

### Trigger Patterns (40-43)
- `40` - Explicit Termination
- `41` - Implicit Termination
- `42` - Termination with Multiple End Events
- `43` - Termination with Cancellation

## Resource Allocation Policies

- `AllocationPolicy::FourEyes` - Dual approval required
- `AllocationPolicy::Chained` - Sequential assignment
- `AllocationPolicy::RoundRobin` - Even distribution
- `AllocationPolicy::ShortestQueue` - Least busy resource
- `AllocationPolicy::RoleBased` - Role matching
- `AllocationPolicy::CapabilityBased` - Capability matching

## Case States

- `CaseState::Created` - Case created, not started
- `CaseState::Running` - Case executing
- `CaseState::Completed` - Case completed successfully
- `CaseState::Cancelled` - Case cancelled
- `CaseState::Failed` - Case failed

## Error Types

- `WorkflowError::InvalidSpecification` - Invalid workflow spec
- `WorkflowError::CaseNotFound` - Case not found
- `WorkflowError::ExecutionFailed` - Execution error
- `WorkflowError::DeadlockDetected` - Deadlock in workflow
- `WorkflowError::ResourceUnavailable` - Resource allocation failed
- `WorkflowError::Parse` - Parsing error

## Turtle Workflow Example

```turtle
@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
@prefix knhk: <https://knhk.org/schema#> .

<workflow:example> a yawl:WorkflowSpec ;
    yawl:name "Example Workflow" ;
    yawl:startCondition <condition:start> ;
    yawl:endCondition <condition:end> ;
    yawl:task <task:task1>, <task:task2> .

<task:task1> a yawl:Task ;
    yawl:name "Task 1" ;
    yawl:splitType yawl:AND ;
    yawl:joinType yawl:AND .

<task:task2> a yawl:Task ;
    yawl:name "Task 2" ;
    yawl:splitType yawl:AND ;
    yawl:joinType yawl:AND .
```

## Worklet Registration

```rust
let worklet = Worklet {
    id: WorkletId::new(),
    name: "Exception Handler".to_string(),
    workflow_spec: spec,
    exception_types: vec!["resource_unavailable".to_string()],
    tags: vec!["exception".to_string()],
    rules: vec![WorkletRule {
        condition: "resource_unavailable".to_string(),
        priority: 100,
    }],
    version: "1.0.0".to_string(),
};

engine.worklet_repository().register(worklet).await?;
```

## Resource Allocation Request

```rust
let request = AllocationRequest {
    task_id: "task:approve".to_string(),
    spec_id,
    required_roles: vec!["approver".to_string()],
    required_capabilities: vec![],
    policy: AllocationPolicy::FourEyes,
    priority: 100,
};

let allocation = engine.resource_allocator().allocate(request).await?;
```

## Deadlock Detection

```rust
use knhk_workflow_engine::validation::DeadlockDetector;

let detector = DeadlockDetector;
let result = detector.validate(&spec)?;

if !result.cycles.is_empty() {
    println!("Deadlock detected: {:?}", result.cycles);
}
```

## Performance Constraints

- Hot path operations: ≤8 ticks (Chatman Constant)
- Pattern execution: ≤8 ticks
- Resource allocation lookup: ≤8 ticks
- Supports 1000+ concurrent cases

## Integration Points

- **OTEL**: Automatic span creation for workflow execution
- **Lockchain**: Automatic receipt recording for provenance
- **Connectors**: External system integration via connector registry
- **REST API**: Axum-based REST server
- **gRPC API**: Tonic-based gRPC service (planned)

---

For detailed documentation, see [WORKFLOW_ENGINE.md](WORKFLOW_ENGINE.md)

