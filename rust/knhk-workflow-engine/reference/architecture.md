# Architecture

High-level architecture of the KNHK Workflow Engine.

## Components

### Workflow Engine

Core execution engine:

- **Parser**: Parse YAWL/Turtle workflows
- **Executor**: Execute workflow cases
- **State Store**: Persist workflow state
- **Pattern Registry**: Execute workflow patterns

### Enterprise Features

- **Fortune 5 Integration**: SLO tracking, promotion gates
- **Observability**: OTEL integration
- **Provenance**: Lockchain integration
- **Security**: SPIFFE/SPIRE, KMS

### APIs

- **REST API**: HTTP/REST interface
- **gRPC API**: gRPC interface
- **Rust API**: Native Rust API

## Data Flow

```
Workflow Spec (Turtle)
    ↓
Parser
    ↓
Workflow Engine
    ↓
State Store
    ↓
Pattern Execution
    ↓
Task Execution
```

## Next Steps

- [Configuration](configuration.md) - Configuration details
- [Error Handling](errors.md) - Error handling guide
