# Sidecar-Workflow Engine Integration

## Overview

The KNHK sidecar has been integrated with the workflow engine, enabling workflow execution through the sidecar's gRPC interface.

## Architecture

### Dependency Direction
- **Sidecar → Workflow Engine**: The sidecar depends on the workflow engine (not the other way around)
- This avoids circular dependencies
- The workflow engine's sidecar integration is a stub to allow compilation

### Integration Points

1. **Service Integration**: `KgcSidecarService` can be created with a workflow engine
2. **Configuration**: Sidecar config includes workflow engine settings
3. **Execution**: Sidecar can execute workflow patterns via `execute_workflow_pattern()`

## Features

### 1. Workflow Engine in Sidecar Service

```rust
use knhk_sidecar::{KgcSidecarService, SidecarConfig};
use knhk_workflow_engine::{WorkflowEngine, StateStore};
use std::sync::Arc;

// Create workflow engine
let state_store = StateStore::new("./workflow_db")?;
let workflow_engine = WorkflowEngine::new(state_store);
let workflow_engine = Arc::new(workflow_engine);

// Create sidecar service with workflow engine
let config = SidecarConfig::default();
let service = KgcSidecarService::with_workflow_engine(config, workflow_engine);
```

### 2. Configuration

#### Environment Variables
- `KGC_SIDECAR_WORKFLOW_ENABLED`: Enable workflow engine (true/false)
- `KGC_SIDECAR_WORKFLOW_DB_PATH`: Path to workflow database (default: `./workflow_db`)

#### Config Fields
```rust
pub struct SidecarConfig {
    // ... other fields ...
    #[cfg(feature = "workflow")]
    pub workflow_enabled: bool,
    #[cfg(feature = "workflow")]
    pub workflow_db_path: Option<String>,
}
```

### 3. Workflow Execution

```rust
// Execute workflow pattern through sidecar
let result = service.execute_workflow_pattern(
    1, // Pattern ID (1-43)
    context, // PatternExecutionContext
).await?;
```

### 4. Fortune 5 Integration

When both `workflow` and `fortune5` features are enabled:
- Workflow engine is created with Fortune 5 configuration
- SLO tracking is enabled for workflow execution
- Promotion gates are checked before workflow execution
- SPIFFE/SPIRE integration is available

## Usage Example

```rust
use knhk_sidecar::{KgcSidecarService, SidecarConfig};
use knhk_workflow_engine::{WorkflowEngine, StateStore, patterns::*};
use std::sync::Arc;

// Create Fortune 5 config
let fortune5_config = Fortune5Config {
    slo: Some(SloConfig {
        r1_p99_max_ns: 2,
        w1_p99_max_ms: 1,
        c1_p99_max_ms: 500,
        window_size_seconds: 60,
    }),
    promotion: Some(PromotionConfig {
        environment: Environment::Production,
        feature_flags: vec!["workflow-engine".to_string()],
        auto_rollback_enabled: true,
        slo_threshold: 0.99,
        rollback_window_seconds: 300,
    }),
    // ... other config ...
};

// Create workflow engine with Fortune 5
let state_store = StateStore::new("./workflow_db")?;
let workflow_engine = WorkflowEngine::with_fortune5(state_store, fortune5_config)?;
let workflow_engine = Arc::new(workflow_engine);

// Create sidecar service
let config = SidecarConfig::default();
let service = KgcSidecarService::with_workflow_engine(config, workflow_engine);

// Execute workflow pattern
let context = PatternExecutionContext {
    case_id: CaseId::new(),
    workflow_id: WorkflowSpecId::new(),
    variables: HashMap::new(),
};

let result = service.execute_workflow_pattern(1, context).await?;
```

## Features Enabled

### With `workflow` Feature
- Workflow engine integration in sidecar
- Pattern execution through sidecar
- Workflow state persistence

### With `workflow` + `fortune5` Features
- All workflow features
- SLO tracking for workflow execution
- Promotion gates for workflow operations
- SPIFFE/SPIRE integration
- KMS integration
- Multi-region support

## API Methods

### `KgcSidecarService`

#### `with_workflow_engine()`
Create a new sidecar service with a workflow engine.

#### `execute_workflow_pattern()`
Execute a workflow pattern (1-43) with the given context.

#### `workflow_engine()`
Get a reference to the workflow engine (if enabled).

## Benefits

1. **Unified Interface**: Execute workflows through sidecar's gRPC interface
2. **Fortune 5 Ready**: Full Fortune 5 integration when enabled
3. **Enterprise Features**: SLO tracking, promotion gates, SPIFFE/SPIRE
4. **No Circular Dependencies**: Clean dependency graph
5. **Feature Gated**: Workflow features are optional

## Next Steps

1. Add gRPC methods for workflow operations
2. Add workflow state management through sidecar
3. Add workflow monitoring and metrics
4. Add workflow case management through sidecar

