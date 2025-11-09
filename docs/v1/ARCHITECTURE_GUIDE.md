# KNHK v1.0 Architecture Guide

**Version**: 1.0.0  
**Status**: Production Architecture Guide  
**Last Updated**: 2025-11-09

---

## Overview

This guide provides a comprehensive overview of the KNHK v1.0 architecture, including system design, component interactions, and architectural patterns.

**Critical Principle**: "Never trust the text, only trust test results" - All architectural claims must be verifiable through tests and OTEL validation.

---

## System Architecture

### Centralized Validation Architecture

**Key Innovation**: All validation and domain logic centralized in `knhk-workflow-engine`. Pure execution in `knhk-hot`.

**Architecture Principle**:
- **knhk-workflow-engine**: ALL data ingress point. Domain logic, validation, guards.
- **knhk-hot**: Pure execution. NO checks. Assumes pre-validated inputs.

**Validation Flow**:
1. Data enters via `knhk-workflow-engine` (API, CLI, `create_case`, `register_workflow`)
2. Guards validate at ingress (`security/guards.rs`)
3. Pre-validated data passed to `knhk-hot` for pure execution
4. `knhk-hot` has ZERO checks - pure execution only

**Benefits**:
- Single source of truth for validation (no scattered checks)
- Hot path performance (no validation overhead)
- Clear separation: ingress validation vs execution
- Domain logic centralized in workflow engine

**Prohibited**: Defensive programming in execution paths (hot path, executor, state). All validation at ingress only.

---

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    KNHK Workflow Engine                      │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   Hot Path   │  │  Warm Path   │  │  Cold Path   │     │
│  │  (≤8 ticks)  │  │  (≤100ms)    │  │  (async)     │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│                                                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              State Manager (Event Sourcing)          │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Pattern Registry (43 Patterns)          │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              API Layer (REST, gRPC, CLI)            │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### Component Architecture

#### 1. Hot Path (≤8 ticks)

**Purpose**: Ultra-low latency operations for critical path execution.

**Components**:
- **SoA Arrays**: 64-byte aligned Structure of Arrays for SIMD operations
- **Hot Path Operations**: ASK_SP, COUNT_SP_GE, ASK_SPO, etc.
- **Tick Budget**: ≤8 ticks (Chatman Constant: 2ns = 8 ticks)

**Key Constraints**:
- Zero-copy operations
- Branchless execution
- NO checks (all validation at ingress in knhk-workflow-engine)
- Guard constraints: max_run_len ≤ 8 (enforced at ingress)
- Pure execution - assumes pre-validated inputs

#### 2. Warm Path (≤100ms)

**Purpose**: Fast operations that don't require hot path optimization.

**Components**:
- **Workflow Execution**: Case creation, task execution
- **Pattern Execution**: 43 Van der Aalst patterns
- **State Management**: Event sourcing and state persistence

**Key Constraints**:
- Async operations allowed
- Database access permitted
- Network calls allowed (with timeouts)

#### 3. Cold Path (async)

**Purpose**: Long-running operations and background tasks.

**Components**:
- **Process Mining**: XES export, Alpha+++ discovery
- **Validation**: Van der Aalst validation, conformance checking
- **Reporting**: Validation reports, performance analysis

**Key Constraints**:
- No time constraints
- Can use external services
- Can perform heavy computations

---

## Core Components

### Workflow Engine

**Purpose**: Central orchestrator for workflow execution.

**Responsibilities**:
- Case lifecycle management (create, start, execute, cancel)
- Workflow specification registration
- Task execution coordination
- State event generation

**Key Interfaces**:
```rust
pub struct WorkflowEngine {
    state_manager: Arc<StateManager>,
    pattern_registry: Arc<PatternRegistry>,
    // ...
}
```

### State Manager

**Purpose**: Event sourcing and state persistence.

**Responsibilities**:
- Event storage and retrieval
- Case history management
- State reconstruction from events
- Event replay

**Key Interfaces**:
```rust
pub enum StateEvent {
    WorkflowRegistered { spec_id: WorkflowSpecId, ... },
    CaseCreated { case_id: CaseId, spec_id: WorkflowSpecId, ... },
    CaseStateChanged { case_id: CaseId, old_state: String, new_state: String, ... },
    TaskStarted { case_id: CaseId, task_id: String, ... },
    TaskCompleted { case_id: CaseId, task_id: String, duration_ms: u64, ... },
}
```

### Pattern Registry

**Purpose**: Management and execution of 43 Van der Aalst workflow patterns.

**Responsibilities**:
- Pattern registration and lookup
- Pattern execution
- Pattern metadata management

**Key Interfaces**:
```rust
pub trait PatternExecutor: Send + Sync {
    fn execute(&self, context: &PatternExecutionContext) -> PatternExecutionResult;
}

pub struct PatternRegistry {
    patterns: HashMap<PatternId, Box<dyn PatternExecutor>>,
}
```

---

## API Architecture

### REST API

**Endpoints**:
- `GET /health` - Health check
- `POST /workflows` - Register workflow
- `POST /cases` - Create case
- `GET /cases/{id}` - Get case status
- `POST /cases/{id}/execute` - Execute case
- `GET /cases/{id}/history` - Get case history

### gRPC API

**Services**:
- `WorkflowService` - Workflow management
- `CaseService` - Case management
- `PatternService` - Pattern execution

### CLI API

**Commands**:
- `knhk workflow parse` - Parse workflow from Turtle file
- `knhk workflow register` - Register a workflow specification
- `knhk workflow create` - Create a new workflow case
- `knhk workflow start` - Start a workflow case
- `knhk workflow execute` - Execute a workflow case
- `knhk workflow cancel` - Cancel a workflow case
- `knhk workflow get` - Get case status
- `knhk workflow list` - List all workflow cases
- `knhk workflow patterns` - List all 43 patterns
- `knhk workflow serve` - Start REST API server
- `knhk workflow import-xes` - Import XES event log
- `knhk workflow export-xes` - Export workflow execution to XES format
- `knhk workflow validate-xes` - Run automated XES validation full loop
- `knhk workflow validate` - Run van der Aalst end-to-end validation framework
- `knhk workflow discover` - Run Alpha+++ process discovery algorithm

---

## Fortune 5 Enterprise Features

### SPIFFE/SPIRE Integration

**Purpose**: Zero-trust identity and authentication.

**Architecture**:
```
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│   SPIRE     │──────│   KNHK      │──────│   Service   │
│   Agent     │      │   Engine    │      │   Mesh      │
└─────────────┘      └─────────────┘      └─────────────┘
```

**Key Components**:
- SPIFFE identity verification
- mTLS certificate management
- Service-to-service authentication

### KMS Integration

**Purpose**: Key management and encryption.

**Architecture**:
```
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│   KNHK      │──────│   KMS       │──────│   Key       │
│   Engine    │      │   Client    │      │   Store     │
└─────────────┘      └─────────────┘      └─────────────┘
```

**Key Components**:
- Key rotation
- Encryption/decryption
- Key metadata management

### Multi-Region Support

**Purpose**: Geographic distribution and disaster recovery.

**Architecture**:
```
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│   Region 1   │──────│   Region 2   │──────│   Region 3  │
│  (Primary)  │      │  (Replica)   │      │  (Replica)  │
└─────────────┘      └─────────────┘      └─────────────┘
```

**Key Components**:
- Last-write-wins conflict resolution
- Cross-region replication
- Region health monitoring

---

## Data Flow

### Workflow Execution Flow

```
1. Parse Workflow (Turtle → WorkflowSpec)
   ↓
2. Register Workflow (WorkflowSpec → StateManager)
   ↓
3. Create Case (WorkflowSpecId → CaseId)
   ↓
4. Start Case (CaseId → StateEvent::CaseStateChanged)
   ↓
5. Execute Tasks (Pattern Execution)
   ↓
6. Complete Case (StateEvent::CaseCompleted)
```

### Pattern Execution Flow

```
1. Pattern Lookup (PatternId → PatternExecutor)
   ↓
2. Create Execution Context (CaseId, TaskId, ...)
   ↓
3. Execute Pattern (PatternExecutor::execute)
   ↓
4. Process Result (PatternExecutionResult)
   ↓
5. Update State (StateEvent generation)
```

---

## Performance Architecture

### Hot Path Optimization

**Techniques**:
- **SoA Layout**: Structure of Arrays for SIMD operations
- **64-byte Alignment**: Cache line alignment
- **Branchless Operations**: Constant-time execution
- **Zero-Copy**: References over clones

**Constraints**:
- ≤8 ticks per operation (Chatman Constant)
- max_run_len ≤ 8 (guard constraint)
- No heap allocations
- No system calls

### Warm Path Optimization

**Techniques**:
- **Async Operations**: Non-blocking I/O
- **Connection Pooling**: Reuse database connections
- **Caching**: In-memory state cache
- **Batch Operations**: Group multiple operations

**Constraints**:
- ≤100ms per operation
- Async operations allowed
- Database access permitted

---

## Security Architecture

### Authentication & Authorization

**Components**:
- SPIFFE identity verification
- mTLS certificate validation
- Role-based access control (RBAC)

### Encryption

**Components**:
- KMS key management
- Data encryption at rest
- Data encryption in transit (TLS)

### Audit & Compliance

**Components**:
- Event sourcing (immutable audit trail)
- OTEL instrumentation (observability)
- Provenance tracking (hash(A) = hash(μ(O)))

---

## Observability Architecture

### OTEL Instrumentation

**Spans**:
- Workflow execution spans
- Task execution spans
- Pattern execution spans
- API request spans

**Metrics**:
- Workflow execution rate
- Task completion rate
- Hot path tick distribution
- Error rate

**Logs**:
- Structured logging (JSON)
- Log levels (trace, debug, info, warn, error)
- Context propagation

### Monitoring Stack

**Components**:
- OTEL Collector
- Prometheus (metrics)
- Grafana (dashboards)
- Jaeger (tracing)

---

## Related Documentation

- [Implementation Guide](./IMPLEMENTATION_GUIDE.md)
- [Deployment Guide](./DEPLOYMENT_GUIDE.md)
- [Operations Guide](./OPERATIONS_GUIDE.md)
- [Definition of Done](./definition-of-done/fortune5-production.md)

---

## Notes

- All architectural claims must be verifiable through tests
- OTEL validation is the source of truth for telemetry
- Performance constraints must be validated with benchmarks
- Security architecture must pass security audits

