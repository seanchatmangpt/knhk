# Integration - 80/20 Guide

**Version**: 1.0  
**Status**: Production-Ready  
**Last Updated**: 2025-01-XX

---

## Overview

KNHK integrates with enterprise systems through connectors, APIs, and workflow engines. This guide covers the critical 20% of integration patterns that provide 80% of value.

**Key Features**:
- ✅ Connector Framework - Enterprise data source integration
- ✅ REST/gRPC APIs - HTTP and gRPC interfaces
- ✅ Workflow Engine - YAWL-compatible workflow execution
- ✅ OTEL Integration - OpenTelemetry observability
- ✅ Lockchain Integration - Cryptographic provenance

---

## Quick Start (80% Use Case)

### Minimal Integration

**Cargo.toml**:
```toml
[dependencies]
knhk-hot = { path = "../knhk-hot", version = "1.0.0" }
knhk-etl = { path = "../knhk-etl", version = "0.1.0" }
knhk-workflow-engine = { path = "../knhk-workflow-engine", version = "1.0.0" }
```

**Basic Usage**:
```rust
use knhk_etl::Pipeline;

let mut pipeline = Pipeline::new(
    vec!["kafka-prod".to_string()],
    "urn:knhk:schema:default".to_string(),
    false,
    vec![],
);

pipeline.run().await?;
```

---

## Core Integration Patterns (80% Value)

### 1. Connector Integration

**Supported Connectors**:
- Kafka - Message queue integration
- Salesforce - CRM integration
- HTTP - REST API integration
- File - File system integration
- SAP - ERP integration

**Basic Pattern**:
```rust
use knhk_connectors::{Connector, KafkaConnector};

let connector = KafkaConnector::new("kafka://localhost:9092/triples")?;
connector.start().await?;

let delta = connector.fetch_delta().await?;
// Process delta...

connector.stop().await?;
```

### 2. ETL Pipeline Integration

**Pipeline Stages**:
1. **Ingest** - Load data from connectors
2. **Transform** - Apply transformations
3. **Load** - Load into knowledge graph
4. **Reflex** - Execute reflexes (hooks)
5. **Emit** - Emit results

**Basic Pattern**:
```rust
use knhk_etl::Pipeline;

let mut pipeline = Pipeline::new(
    vec!["connector1".to_string()],
    "urn:my:schema".to_string(),
    true,  // enable_reflex
    vec![], // reflexes
);

pipeline.run().await?;
```

### 3. Workflow Engine Integration

**Basic Pattern**:
```rust
use knhk_workflow_engine::{WorkflowEngine, WorkflowParser, StateStore};

let state_store = StateStore::new("./workflow_db")?;
let engine = WorkflowEngine::new(state_store);

// Parse workflow
let mut parser = WorkflowParser::new()?;
let spec = parser.parse_file("workflow.ttl")?;

// Register and execute
engine.register_workflow(spec.clone()).await?;
let case_id = engine.create_case(spec.id, serde_json::json!({})).await?;
engine.execute_case(case_id).await?;
```

### 4. REST API Integration

**Endpoints**:
- `POST /workflows` - Register workflow
- `POST /cases` - Create case
- `GET /cases/{id}` - Get case status
- `POST /cases/{id}/cancel` - Cancel case

**Example**:
```bash
# Register workflow
curl -X POST http://localhost:8080/workflows \
  -H "Content-Type: application/json" \
  -d @workflow.json

# Create case
curl -X POST http://localhost:8080/cases \
  -H "Content-Type: application/json" \
  -d '{"spec_id": "...", "data": {...}}'
```

### 5. OTEL Integration

**Automatic Integration**:
- Spans created for all operations
- Metrics emitted for performance
- Logs structured with context

**Custom Spans**:
```rust
use tracing::instrument;

#[instrument]
async fn my_operation() -> Result<()> {
    // Operation automatically traced
    Ok(())
}
```

### 6. Lockchain Integration

**Automatic Integration**:
- Receipts generated for all operations
- Cryptographic verification enabled
- Immutable audit trail maintained

**Manual Receipt Access**:
```rust
let receipt = operation.get_receipt()?;
let hash = receipt.hash();
// Verify hash matches operation result
```

---

## Integration Patterns

### Pattern 1: Kafka Connector

**Use Case**: Real-time data ingestion from Kafka.

**Implementation**:
```rust
use knhk_connectors::KafkaConnector;

let connector = KafkaConnector::new("kafka://localhost:9092/triples")?;
connector.start().await?;

loop {
    let delta = connector.fetch_delta().await?;
    pipeline.process_delta(delta).await?;
}
```

### Pattern 2: Workflow + ETL

**Use Case**: Execute workflows triggered by ETL pipeline events.

**Implementation**:
```rust
// ETL pipeline emits events
pipeline.on_event(|event| {
    // Trigger workflow execution
    workflow_engine.create_case(spec_id, event.data).await?;
});
```

### Pattern 3: REST API + Workflow Engine

**Use Case**: Expose workflow engine via REST API.

**Implementation**:
```rust
// REST handler
async fn create_case(req: CreateCaseRequest) -> Result<CaseResponse> {
    let case_id = workflow_engine
        .create_case(req.spec_id, req.data)
        .await?;
    Ok(CaseResponse { case_id })
}
```

---

## Feature Flags

**Available Features**:
- `kafka` - Kafka connector support
- `salesforce` - Salesforce connector support
- `rego` - Rego policy engine support
- `network` - Network operations support

**Usage**:
```toml
[dependencies]
knhk-connectors = { path = "../knhk-connectors", features = ["kafka"] }
knhk-validation = { path = "../knhk-validation", features = ["rego"] }
```

---

## Error Handling

**Structured Errors**:
```rust
pub enum IntegrationError {
    ConnectorError(ConnectorError),
    PipelineError(PipelineError),
    WorkflowError(WorkflowError),
    // ...
}
```

**Error Handling Pattern**:
```rust
match connector.fetch_delta().await {
    Ok(delta) => process_delta(delta).await?,
    Err(ConnectorError::NetworkError(e)) => {
        // Retry logic
        retry_with_backoff().await?;
    }
    Err(e) => return Err(IntegrationError::ConnectorError(e)),
}
```

---

## Performance Considerations

### Hot Path Integration

**Best Practices**:
- Use C API for hot path operations
- Pre-allocate contexts
- Reuse contexts across operations
- Batch operations when possible

### Warm Path Integration

**Best Practices**:
- Use async/await for I/O
- Batch multiple operations
- Use connection pooling
- Implement circuit breakers

### Cold Path Integration

**Best Practices**:
- Use full SPARQL engine
- Cache query results
- Optimize complex queries
- Use parallel execution

---

## Production Readiness

### ✅ Ready for Production

- **Connector Framework**: Complete (Kafka, HTTP, File, Salesforce, SAP)
- **ETL Pipeline**: Fully functional
- **Workflow Engine**: 82% YAWL parity
- **REST API**: Complete
- **OTEL Integration**: Complete
- **Lockchain Integration**: Complete

### ⚠️ Partial Production Readiness

- **gRPC API**: Proto defined, handlers in progress
- **Workflow Connectors**: Framework exists, execution incomplete

---

## Troubleshooting

### Connector Connection Issues

**Problem**: Connector fails to connect.

**Solution**:
- Verify connector URI format
- Check network connectivity
- Review connector logs
- Verify authentication credentials

### Pipeline Execution Issues

**Problem**: Pipeline fails to execute.

**Solution**:
- Check connector configuration
- Verify schema IRI format
- Review error messages
- Check guard constraints

### Workflow Execution Issues

**Problem**: Workflows fail to execute.

**Solution**:
- Verify Turtle/RDF format
- Check for deadlock cycles
- Review workflow specification
- Check resource allocation

---

## Additional Resources

### Related Consolidated Guides
- **[Architecture Guide](ARCHITECTURE.md)** - System architecture and component design
- **[API Guide](API.md)** - API interfaces and usage patterns
- **[Workflow Engine Guide](WORKFLOW_ENGINE.md)** - Workflow execution and patterns
- **[CLI Guide](CLI.md)** - CLI usage and command reference

### Detailed Documentation
- **Integration Guide**: [Integration Guide](archived/reference-docs/integration-guide.md) (archived - detailed reference) - Complete integration reference
- **unrdf Integration**: [unrdf Integration Plan](v1.0-unrdf-integration-plan.md) - Cold path integration
- **Lockchain Compatibility**: [Lockchain Compatibility](lockchain-unrdf-compatibility-check.md) - Provenance integration

### Code Examples
- **Integration Examples**: `examples/` - Working integration examples
- **Connector Examples**: `rust/knhk-connectors/examples/` - Connector examples
- **Workflow Examples**: `ontology/workflows/` - Workflow examples

### Related Guides
- **Architecture Guide**: [Architecture Guide](ARCHITECTURE.md) - System architecture
- **API Guide**: [API Guide](API.md) - Programmatic API usage
- **CLI Guide**: [CLI Guide](CLI.md) - Command-line interface

---

## License

MIT License

---

**Last Updated**: 2025-01-XX  
**Version**: 1.0  
**Status**: Production-Ready

