# Introduction

Welcome to the KNHK Workflow Engine documentation!

The KNHK Workflow Engine is a production-ready, enterprise-grade workflow execution engine built in Rust. It provides comprehensive support for workflow patterns, enterprise features, and integrations.

## What is a Workflow Engine?

A workflow engine executes business processes defined as workflows. Workflows consist of:

- **Tasks**: Individual units of work
- **Flows**: Connections between tasks
- **Conditions**: Decision points and branching logic
- **Resources**: Allocation of work to resources

## Why KNHK Workflow Engine?

### Enterprise-Grade

- **Fortune 5 Ready**: SLO tracking, promotion gates, multi-region support
- **Observability**: Full OTEL integration for tracing and metrics
- **Provenance**: Lockchain integration for audit trails
- **Security**: SPIFFE/SPIRE integration, KMS support

### Comprehensive Pattern Support

- **All 43 Van der Aalst Patterns**: Complete workflow pattern coverage
- **YAWL Compatibility**: Parse and execute YAWL workflow definitions
- **Advanced Control Flow**: Complex branching, loops, cancellation

### Developer-Friendly

- **Chicago TDD**: Comprehensive testing framework
- **ggen Integration**: RDF-driven template generation
- **REST & gRPC APIs**: Multiple interface options
- **Rich Documentation**: Comprehensive guides and examples

## Key Concepts

### Workflow Specification

A workflow specification defines the structure of a workflow:

```rust
use knhk_workflow_engine::parser::WorkflowSpec;

let spec = WorkflowSpec {
    id: WorkflowSpecId::new(),
    name: "My Workflow".to_string(),
    tasks: HashMap::new(),
    // ...
};
```

### Workflow Case

A case is an instance of a workflow execution:

```rust
use knhk_workflow_engine::case::{Case, CaseId};

let case_id = engine.create_case(spec_id, data).await?;
let case = engine.get_case(case_id).await?;
```

### Task Execution

Tasks are executed according to workflow patterns:

```rust
engine.execute_case(case_id).await?;
```

## Next Steps

- [Quick Start](quick-start.md) - Get up and running in minutes
- [Installation](installation.md) - Detailed installation instructions
- [Basic Concepts](basic-concepts.md) - Learn core concepts

