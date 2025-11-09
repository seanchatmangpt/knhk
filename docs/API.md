# API - 80/20 Guide

**Version**: 1.0  
**Status**: Production-Ready  
**Last Updated**: 2025-01-XX

---

## Overview

KNHK provides APIs in C, Rust, and Erlang for high-performance knowledge graph operations. This guide covers the critical 20% of API usage that provides 80% of value.

**Key Features**:
- ✅ C API (Hot Path) - ≤8 tick operations
- ✅ Rust API - ETL pipeline and workflow engine
- ✅ Erlang API - Reflexive control layer
- ✅ Unified interface - Consistent patterns across languages

---

## Quick Start (80% Use Case)

### C API (Hot Path)

**Include Header**:
```c
#include "knhk.h"  // Includes all API components
```

**Basic Usage**:
```c
// Initialize context
knhk_ctx_t ctx;
knhk_init_ctx(&ctx, 8);  // max_run_len = 8

// Load RDF data
knhk_load_rdf(&ctx, "data.ttl");

// Execute query (ASK operation)
knhk_op_t op = KNHK_OP_ASK_SP;
uint64_t s = 0xC0FFEE;
uint64_t p = 0xDEADBEEF;
knhk_receipt_t receipt;
int result = knhk_eval_bool(&ctx, op, s, p, 0, &receipt);

// Check result
if (result == 1) {
    printf("Triple exists\n");
}
```

### Rust API (ETL Pipeline)

**Basic Usage**:
```rust
use knhk_etl::Pipeline;

let mut pipeline = Pipeline::new(
    vec!["connector1".to_string()],
    "urn:my:schema".to_string(),
    false,
    vec![],
);

pipeline.run().await?;
```

### Rust API (Workflow Engine)

**Basic Usage**:
```rust
use knhk_workflow_engine::{WorkflowEngine, WorkflowParser, StateStore};

let state_store = StateStore::new("./workflow_db")?;
let engine = WorkflowEngine::new(state_store);
let mut parser = WorkflowParser::new()?;
let spec = parser.parse_file("workflow.ttl")?;
engine.register_workflow(spec).await?;
```

---

## Core API (80% Value)

### C API Operations

**Hot Path Operations** (≤8 ticks):
- `KNHK_OP_ASK_SP` - ASK(S,P) existence check
- `KNHK_OP_COUNT_SP_GE` - COUNT(S,P) >= k
- `KNHK_OP_COUNT_SP_LE` - COUNT(S,P) <= k
- `KNHK_OP_COUNT_SP_EQ` - COUNT(S,P) == k
- `KNHK_OP_ASK_SPO` - ASK(S,P,O) triple matching
- `KNHK_OP_ASK_OP` - ASK(O,P) reverse lookup
- `KNHK_OP_UNIQUE_SP` - UNIQUE(S,P) exactly one value
- `KNHK_OP_COMPARE_O_LT` - COMPARE(O < value)
- `KNHK_OP_COMPARE_O_GE` - COMPARE(O >= value)
- `KNHK_OP_VALIDATE_DATATYPE_SP` - VALIDATE datatype

**Key Functions**:
```c
// Initialize context
int knhk_init_ctx(knhk_ctx_t *ctx, size_t max_run_len);

// Load RDF data
int knhk_load_rdf(knhk_ctx_t *ctx, const char *filename);

// Execute boolean query
int knhk_eval_bool(knhk_ctx_t *ctx, knhk_op_t op, 
                   uint64_t s, uint64_t p, uint64_t o, 
                   knhk_receipt_t *receipt);

// Execute CONSTRUCT8 query
int knhk_eval_construct8(knhk_ctx_t *ctx, knhk_op_t op,
                         uint64_t s, uint64_t p,
                         knhk_receipt_t *receipt);
```

### Rust API (ETL Pipeline)

**Core Types**:
```rust
pub struct Pipeline {
    connectors: Vec<String>,
    schema_iri: String,
    // ...
}

impl Pipeline {
    pub fn new(connectors: Vec<String>, schema_iri: String, 
               enable_reflex: bool, reflexes: Vec<Reflex>) -> Self;
    
    pub async fn run(&mut self) -> Result<()>;
}
```

### Rust API (Workflow Engine)

**Core Types**:
```rust
pub struct WorkflowEngine {
    state_store: Arc<StateStore>,
    // ...
}

impl WorkflowEngine {
    pub async fn register_workflow(&self, spec: WorkflowSpec) -> WorkflowResult<()>;
    pub async fn create_case(&self, spec_id: WorkflowSpecId, data: Value) -> WorkflowResult<CaseId>;
    pub async fn execute_case(&self, case_id: CaseId) -> WorkflowResult<()>;
}
```

---

## Error Handling

### C API

**Return Codes**:
- `0` - Success
- `-1` - Error (check `errno` or error context)

**Error Checking**:
```c
int result = knhk_eval_bool(&ctx, op, s, p, 0, &receipt);
if (result < 0) {
    fprintf(stderr, "Error: %s\n", knhk_get_error(&ctx));
    return -1;
}
```

### Rust API

**Result Types**:
```rust
// ETL Pipeline
pub type Result<T> = std::result::Result<T, PipelineError>;

// Workflow Engine
pub type WorkflowResult<T> = std::result::Result<T, WorkflowError>;
```

**Error Handling**:
```rust
match pipeline.run().await {
    Ok(()) => println!("Pipeline completed"),
    Err(e) => eprintln!("Error: {}", e),
}
```

---

## Performance Considerations

### Hot Path (C API)

**Performance Targets**:
- ≤8 ticks for all hot path operations
- ≤2ns per operation
- Zero allocations in hot path

**Best Practices**:
- Pre-allocate context with `knhk_init_ctx()`
- Reuse context across operations
- Use SoA layout for data (Structure-of-Arrays)

### Warm Path (Rust API)

**Performance Targets**:
- ≤500ms for CONSTRUCT8 operations
- Batch operations when possible

**Best Practices**:
- Use async/await for I/O operations
- Batch multiple operations
- Reuse pipeline instances

---

## Production Readiness

### ✅ Ready for Production

- **C API**: Fully functional (hot path operations)
- **Rust ETL API**: Complete (pipeline operations)
- **Rust Workflow API**: Complete (workflow engine)
- **Error Handling**: Comprehensive (Result types)

### ⚠️ Partial Production Readiness

- **Erlang API**: Complete but less documented
- **gRPC API**: Proto defined, handlers in progress

---

## Troubleshooting

### C API Issues

**Problem**: Context initialization fails.

**Solution**: 
- Check `max_run_len ≤ 8` (guard constraint)
- Verify memory allocation
- Check error context with `knhk_get_error()`

### Rust API Issues

**Problem**: Pipeline fails to start.

**Solution**:
- Verify connector configuration
- Check schema IRI format
- Review error messages for specific issues

### Workflow API Issues

**Problem**: Workflow registration fails.

**Solution**:
- Verify Turtle/RDF format
- Check for deadlock cycles
- Review workflow specification

---

## Additional Resources

### Related Consolidated Guides
- **[Architecture Guide](ARCHITECTURE.md)** - System architecture and component design
- **[Performance Guide](PERFORMANCE.md)** - Performance considerations and optimization
- **[CLI Guide](CLI.md)** - CLI usage and command reference
- **[Integration Guide](INTEGRATION.md)** - Integration patterns and connectors

### Detailed Documentation
- **Complete API Reference**: [API Documentation](api.md) - Complete API reference with all functions
- **C API Details**: `c/include/knhk/` - Header files with full documentation
- **Rust API Details**: `rust/knhk-*/src/` - Rust crate documentation
- **Erlang API Details**: `erlang/knhk_rc/src/` - Erlang module documentation

### Code Examples
- **C Examples**: `c/tests/` - C API test examples
- **Rust Examples**: `rust/knhk-*/examples/` - Rust API examples
- **Workflow Examples**: `ontology/workflows/` - Workflow examples

### Architecture
- **Architecture Guide**: [Architecture Guide](ARCHITECTURE.md) - System architecture
- **Performance Guide**: [Performance Guide](PERFORMANCE.md) - Performance optimization

---

## License

MIT License

---

**Last Updated**: 2025-01-XX  
**Version**: 1.0  
**Status**: Production-Ready
