# knhk-yawl

Core YAWL (Yet Another Workflow Language) data structures and execution engine for KNHK.

## Doctrine Alignment

This package implements YAWL foundations following DOCTRINE_2027 principles:

- **Covenant 1** (O ⊨ Σ): All workflow definitions are observable via telemetry
- **Covenant 2** (Q ⊨ Implementation): Respects Q3 (max_run_length ≤ 8 ticks)
- **Covenant 4** (Σ ⊨ Completeness): All 43 W3C patterns expressible via permutations
- **Covenant 5** (Q3 ⊨ Boundedness): Hot path operations ≤ 8 ticks (Chatman constant)
- **Covenant 6** (O ⊨ Discovery): Full OpenTelemetry instrumentation

## Features

- **Core Data Structures**: Workflow, Task, Transition, NetState, ExecutionContext
- **Pattern Support**: All 43+ W3C workflow patterns
- **Execution Engine**: WorkflowExecutor, TaskActor, TokenManager
- **Telemetry**: Full OpenTelemetry integration for observability
- **Performance**: Designed for ≤8 tick execution (Chatman constant)

## Module Structure

- `core`: Core YAWL data types
- `patterns`: W3C workflow pattern implementations
- `engine`: Workflow execution engine
- `telemetry`: OpenTelemetry integration

## Usage

```rust
use knhk_yawl::core::{Workflow, Task, TaskType};
use knhk_yawl::engine::WorkflowExecutor;

// Create a task
let task = Task::builder()
    .id("task1")
    .name("Process Order")
    .task_type(TaskType::Atomic)
    .build();

// Create a workflow
let workflow = Workflow::builder()
    .id("order-processing")
    .name("Order Processing Workflow")
    .version("1.0.0")
    .add_task(task)
    .build();

// Execute
let executor = WorkflowExecutor::new();
executor.register_workflow(workflow)?;
let context = executor.start_workflow("order-processing", "inst-001")?;
```

## Testing

```bash
cargo test --package knhk-yawl
```

## Performance

All hot path operations are designed to complete within 8 ticks (Chatman constant).
Performance is validated via Chicago TDD harness.

## License

MIT
