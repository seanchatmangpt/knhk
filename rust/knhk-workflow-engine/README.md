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
- **Enterprise APIs**: REST and gRPC interfaces
- **State Persistence**: Sled-based state store
- **Observability**: OTEL integration for tracing
- **Provenance**: Lockchain integration for audit trails

## Usage

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
engine.register_workflow(spec).await?;

// Create and execute case
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

## License

MIT

