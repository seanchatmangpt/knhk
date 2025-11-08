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
- **Enterprise APIs**: REST and gRPC interfaces
- **State Persistence**: Sled-based state store
- **Observability**: OTEL integration for tracing
- **Provenance**: Lockchain integration for audit trails

## Usage

```rust
use knhk_workflow_engine::{WorkflowEngine, WorkflowParser, StateStore, WorkflowSpec, WorkflowSpecId, PatternId};
use knhk_workflow_engine::resource::{Resource, ResourceId, Role, Capability};
use knhk_workflow_engine::worklets::{Worklet, WorkletMetadata, WorkletRule, WorkletId};
use std::collections::HashMap;

// Create state store
let state_store = StateStore::new("./workflow_db")?;

// Create engine (includes resource allocator and worklet repository)
let engine = WorkflowEngine::new(state_store);

// Register resources
let resource_allocator = engine.resource_allocator();
resource_allocator.register_resource(Resource {
    id: ResourceId::new(),
    name: "User1".to_string(),
    roles: vec![Role {
        id: "approver".to_string(),
        name: "Approver".to_string(),
        capabilities: vec!["approval".to_string()],
    }],
    capabilities: vec![Capability {
        id: "approval".to_string(),
        name: "Approval".to_string(),
        level: 100,
    }],
    workload: 0,
    queue_length: 0,
    available: true,
}).await?;

// Register worklets
let worklet_repo = engine.worklet_repository();
let worklet = Worklet {
    metadata: WorkletMetadata {
        id: WorkletId::new(),
        name: "Exception Handler".to_string(),
        description: "Handles resource allocation failures".to_string(),
        version: "1.0.0".to_string(),
        exception_types: vec!["resource_unavailable".to_string()],
        required_context: vec![],
        pattern_ids: vec![PatternId(1)],
        tags: vec!["exception".to_string()],
    },
    workflow_spec: WorkflowSpec {
        id: WorkflowSpecId::new(),
        name: "Exception Handler Workflow".to_string(),
        tasks: HashMap::new(),
        conditions: HashMap::new(),
        start_condition: None,
        end_condition: None,
    },
    rules: vec![WorkletRule {
        id: "rule1".to_string(),
        name: "Resource Unavailable Rule".to_string(),
        condition: "true".to_string(),
        worklet_id: WorkletId::new(),
        priority: 100,
    }],
};
worklet_repo.register(worklet).await?;

// Parse workflow from Turtle (includes deadlock validation)
let mut parser = WorkflowParser::new()?;
let spec = parser.parse_file("workflow.ttl")?;
let spec_id = spec.id;

// Register workflow (includes deadlock validation)
engine.register_workflow(spec).await?;

// Create and execute case (includes resource allocation and worklet support)
let case_id = engine.create_case(spec_id, serde_json::json!({})).await?;
engine.start_case(case_id).await?;
engine.execute_case(case_id).await?;
```

## Pattern Categories

- **Basic Control Flow** (1-5): Sequence, Parallel Split, Synchronization, Exclusive Choice, Simple Merge
- **Advanced Branching** (6-11): Multi-Choice, Structured Synchronizing Merge, Multi-Merge, Discriminator, Arbitrary Cycles, Implicit Termination
- **Multiple Instance** (12-15): Various multiple instance patterns
- **State-Based** (16-18): Deferred Choice, Interleaved Parallel Routing, Milestone
- **Cancellation** (19-25): Various cancellation patterns
- **Advanced** (26-39): Additional control flow patterns
- **Trigger** (40-43): Event-driven trigger patterns

## API

### REST API

- `POST /workflows` - Register workflow specification
- `GET /workflows/{id}` - Get workflow specification
- `POST /cases` - Start workflow case
- `GET /cases/{id}` - Get case status
- `POST /cases/{id}/cancel` - Cancel case
- `GET /cases/{id}/history` - Get execution history

### gRPC API

See `api/grpc.rs` for gRPC service definitions.

## Dependencies

- `knhk-otel` - Observability
- `knhk-lockchain` - Provenance tracking
- `knhk-unrdf` - SPARQL/SHACL validation
- `knhk-connectors` - External system integration
- `oxigraph` - RDF store
- `rio-turtle` - Turtle parser
- `sled` - State persistence
- `axum` - REST API server
- `tonic` - gRPC framework

## Documentation

For comprehensive documentation, see:
- **[YAWL Integration Complete](docs/YAWL_INTEGRATION_COMPLETE.md)** - Complete integration guide with execution flows and API examples
- **[YAWL Implementation Summary](docs/YAWL_IMPLEMENTATION_SUMMARY.md)** - YAWL feature implementation status and integration details
- **[YAWL Feature Comparison](docs/YAWL_FEATURE_COMPARISON.md)** - Comparison with YAWL Java implementation
- **[Gap Analysis](docs/GAP_ANALYSIS_FILLED.md)** - Gap analysis and filling summary
- **[SWIFT FIBO Case Study](docs/SWIFT_FIBO_CASE_STUDY.md)** - Enterprise case study with all 43 patterns
- **[Implementation Plan](FORTUNE5_IMPLEMENTATION_PLAN.md)** - Implementation roadmap and checklist

## License

MIT

