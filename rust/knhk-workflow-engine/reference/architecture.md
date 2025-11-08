# Architecture

High-level architecture of the KNHK Workflow Engine.

## Multi-Layered Abstraction Architecture

The KNHK Workflow Engine uses a **multi-layered abstraction architecture** that provides:

### 1. Facade Layer (Entry Points)
- **LegacyFacade**: YAWL-compatible interface for BPM/ERP systems
- **ReflexFacade**: Hot-path optimized interface (≤8 ticks)
- **EnterpriseFacade**: Fortune-5 features interface
- **ApiFacade**: REST/gRPC interface
- **CliFacade**: Command-line interface

### 2. Service Layer (Business Logic)
- **WorkflowService**: Core workflow operations
- **CaseService**: Case lifecycle management
- **PatternService**: Pattern execution orchestration
- **ProvenanceService**: Receipt generation and validation
- **ResourceService**: Resource allocation coordination

### 3. Builder Layer (Configuration)
- **EngineBuilder**: Fluent API for engine construction
- **ServiceBuilder**: Service configuration builder
- **FacadeBuilder**: Facade configuration builder

### 4. Trait-Based Interfaces (Extensibility)
- **WorkflowExecutor**: Core execution interface
- **RuntimeClassExecutor**: R1/W1/C1 execution interfaces
- **ProvenanceProvider**: Receipt generation interface
- **ResourceProvider**: Resource allocation interface

### 5. Plugin Architecture (Runtime Classes)
- **R1Executor**: Hot-path executor plugin (≤8 ticks)
- **W1Executor**: Warm-path executor plugin (≤1ms)
- **C1Executor**: Cold-path executor plugin (≤500ms)

### 6. Unified Gateway (Routing)
- **WorkflowGateway**: Routes requests to appropriate facade/service
- **RuntimeRouter**: Routes to appropriate runtime class executor

See [ABSTRACTION_ARCHITECTURE_PLAN.md](../ABSTRACTION_ARCHITECTURE_PLAN.md) for detailed architecture documentation.

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
