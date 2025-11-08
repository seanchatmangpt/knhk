# knhk-workflow-engine

Enterprise workflow engine with full 43-pattern YAWL support.

## Overview

This crate provides a complete workflow engine that:
- Parses Turtle/YAWL workflow definitions
- Executes all 43 Van der Aalst workflow patterns
- Provides enterprise APIs (REST + gRPC)
- Manages workflow cases with state persistence
- Integrates with KNHK infrastructure (OTEL, lockchain, connectors)

## Features

- **Full Pattern Support**: All 43 Van der Aalst workflow patterns
- **YAWL Compatibility**: Parses and executes YAWL workflow definitions
- **Resource Allocation**: Advanced allocation policies (Four-eyes, Chained, Round-robin, Shortest queue, Role-based, Capability-based)
- **Worklets**: Dynamic workflow adaptation with worklet repository and exception handling
- **Deadlock Detection**: Design-time deadlock detection with Petri net analysis
- **Workflow Visualization**: Generate visual diagrams from workflow specs (DOT, SVG, HTML)
- **Template Library**: Pre-built workflow templates for common patterns
- **Performance Analyzer**: Profile workflows and identify optimization opportunities
- **Chicago TDD Framework**: Comprehensive testing framework with builders, helpers, and macros
- **Enterprise APIs**: REST and gRPC interfaces
- **State Persistence**: Sled-based state store
- **Observability**: OTEL integration for tracing
- **Provenance**: Lockchain integration for audit trails

## Quick Start

### CLI Usage

The workflow engine is integrated into the main `knhk` CLI:

```bash
# Parse a workflow
knhk workflow parse examples/simple-sequence.ttl

# Register workflow
knhk workflow register examples/simple-sequence.ttl

# Create and execute a case
knhk workflow create <spec-id> --data '{"input":"test"}'
knhk workflow start <case-id>
knhk workflow execute <case-id>

# List workflows and cases
knhk workflow list              # List all workflows
knhk workflow list <spec-id>    # List cases for workflow

# List all patterns
knhk workflow patterns

# Start REST API server
knhk workflow serve --port 8080
```

See **[CLI Commands](../knhk-cli/docs/WORKFLOW_COMMANDS.md)** for complete CLI documentation.

### Rust Usage

```rust
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
```

## Quick Reference

### Common Operations

```rust
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
```

### REST API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/workflows` | Register workflow |
| GET | `/workflows/{id}` | Get workflow |
| POST | `/cases` | Create case |
| GET | `/cases/{id}` | Get case status |
| POST | `/cases/{id}/cancel` | Cancel case |

### Pattern Categories

- **Basic Control Flow (1-5)**: Sequence, Parallel Split, Synchronization, Exclusive Choice, Simple Merge
- **Advanced Branching (6-11)**: Multi-Choice, Structured Synchronizing Merge, Multi-Merge, Discriminator, Arbitrary Cycles, Implicit Termination
- **Multiple Instance (12-15)**: MI Without Sync, MI With Design-Time Knowledge, MI With Runtime Knowledge
- **State-Based (16-18)**: Deferred Choice, Interleaved Parallel Routing, Milestone
- **Cancellation (19-25)**: Cancel Activity, Cancel Case, Cancel Region, Cancel MI Activity
- **Advanced Patterns (26-39)**: Blocking Discriminator, Cancelling Discriminator, Structured Loop, Recursion
- **Trigger Patterns (40-43)**: Explicit Termination, Implicit Termination, Termination with Multiple End Events

### Resource Allocation Policies

- `AllocationPolicy::FourEyes` - Dual approval required
- `AllocationPolicy::Chained` - Sequential assignment
- `AllocationPolicy::RoundRobin` - Even distribution
- `AllocationPolicy::ShortestQueue` - Least busy resource
- `AllocationPolicy::RoleBased` - Role matching
- `AllocationPolicy::CapabilityBased` - Capability matching

### Case States

- `CaseState::Created` - Case created, not started
- `CaseState::Running` - Case executing
- `CaseState::Completed` - Case completed successfully
- `CaseState::Cancelled` - Case cancelled
- `CaseState::Failed` - Case failed

## Documentation

- **[Complete Documentation](docs/WORKFLOW_ENGINE.md)** - Comprehensive guide with API reference, examples, and integration guides
- **[CLI Commands](../knhk-cli/docs/WORKFLOW_COMMANDS.md)** - Complete CLI command reference

## Performance

- Hot path operations: ≤8 ticks (Chatman Constant)
- Pattern execution: ≤8 ticks
- Resource allocation lookup: ≤8 ticks
- Supports 1000+ concurrent cases

## Integration

- **OTEL**: Automatic span creation for workflow execution
- **Lockchain**: Automatic receipt recording for provenance
- **Connectors**: External system integration via connector registry
- **REST API**: Axum-based REST server
- **gRPC API**: Tonic-based gRPC service (planned)

## License

MIT

