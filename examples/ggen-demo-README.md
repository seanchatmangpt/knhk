# ggen Template Demonstration

This example demonstrates the complete ggen projection layer (Π) for code generation from ontologies.

## Overview

The demo workflow (`ggen-demo-workflow.ttl`) showcases:

- **8 tasks** including parallel split and synchronization
- **7 states** with initial, active, and final states
- **6 transitions** with guard constraints
- **3 guards** enforcing the Chatman Constant (≤8 ticks)
- **3 hooks** for pre/post-conditions and reactive behavior
- **YAWL patterns**: Sequence, Parallel Split, Synchronization, Decision
- **SPARQL integration** for guard checks
- **Full OTEL observability** via generated code

## Quick Start

### 1. Generate Code from Ontology

```bash
# Generate all code
./scripts/ggen/generate-workflows.sh examples/ggen-demo-workflow.ttl

# Output in target/generated/:
#   - Rust code (src/)
#   - YAML config (config/)
#   - Weaver registry (weaver/)
```

### 2. Review Generated Files

```bash
# Task enumeration
cat target/generated/src/task_enum.rs

# State machine
cat target/generated/src/state_machine.rs

# Hooks
cat target/generated/src/hooks.rs

# OTEL spans
cat target/generated/src/otel_spans.rs

# Configuration
cat target/generated/config/workflow.yaml

# Weaver registry (OTEL schema)
cat target/generated/weaver/registry.yaml
```

### 3. Validate Generated Code

```bash
# Rust validation
rustfmt --check target/generated/src/*.rs
cargo clippy --manifest-path target/generated/Cargo.toml -- -D warnings

# YAML validation
yamllint target/generated/config/*.yaml

# Weaver validation (SOURCE OF TRUTH!)
weaver registry check -r target/generated/weaver/
```

### 4. Build and Test

```bash
# Copy to project
mkdir -p rust/demo-workflow
cp target/generated/src/* rust/demo-workflow/src/

# Add Cargo.toml
cat > rust/demo-workflow/Cargo.toml <<EOF
[package]
name = "demo-workflow"
version = "1.0.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
opentelemetry = "0.21"
tracing = "0.1"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
EOF

# Build
cd rust/demo-workflow
cargo build

# Test
cargo test
```

## Generated Code Examples

### Task Enum

```rust
pub enum DemoWorkflowTask {
    /// Initial task that submits a request for processing
    SubmitRequest,

    /// Validates the submitted request
    ValidateRequest,

    /// Splits work into parallel execution paths
    ParallelProcessing,

    /// First parallel processing branch
    ProcessBranchA,

    /// Second parallel processing branch
    ProcessBranchB,

    /// Synchronizes parallel execution paths
    JoinResults,

    /// Manager approves or rejects request
    ApproveRequest,

    /// Final task that completes the workflow
    CompleteWorkflow,
}

impl DemoWorkflowTask {
    pub fn yawl_pattern(&self) -> &'static str {
        match self {
            Self::SubmitRequest => "Basic",
            Self::ValidateRequest => "Sequence",
            Self::ParallelProcessing => "ParallelSplit",
            Self::ProcessBranchA => "Concurrent",
            Self::ProcessBranchB => "Concurrent",
            Self::JoinResults => "Synchronization",
            Self::ApproveRequest => "Decision",
            Self::CompleteWorkflow => "Basic",
        }
    }
}
```

### State Machine

```rust
pub enum DemoWorkflowState {
    Initial,
    Submitted,
    Validating,
    Processing,
    PendingApproval,
    Approved,
    Rejected,
}

impl StateMachine {
    pub fn transition(&mut self, event: &str) -> Result<()> {
        match (&self.current_state, event) {
            (State::Initial, "submit") => {
                self.current_state = State::Submitted;
                Ok(())
            }
            (State::Submitted, "validate") => {
                // Check guard: amount_check
                if !self.check_guard_amount_check()? {
                    return Err(WorkflowError::GuardCheckFailed {
                        guard: "amount_check".to_string(),
                        transition: "submit -> validate".to_string(),
                    });
                }
                self.current_state = State::Validating;
                Ok(())
            }
            // ... more transitions
            _ => Err(WorkflowError::InvalidTransition {
                from: self.current_state.as_str().to_string(),
                event: event.to_string(),
            }),
        }
    }
}
```

### Hooks

```rust
pub async fn on_validate_hook(
    context: &mut HookContext,
    input: Value,
) -> Result<Value> {
    // Guard check: data_valid
    if !check_guard_data_valid(context)? {
        return Err(WorkflowError::GuardCheckFailed {
            guard: "data_valid".to_string(),
            transition: "validate".to_string(),
        });
    }

    // Execute SPARQL query
    // ASK { ?request a :Request ; :hasData ?data . FILTER(strlen(?data) > 0) }

    // Execute hook logic
    Ok(serde_json::json!({
        "hook": "on_validate",
        "status": "executed",
        "timestamp": std::time::SystemTime::now()
    }))
}
```

### OTEL Metrics

```rust
pub struct DemoWorkflowMetrics {
    pub task_executions: Counter<u64>,
    pub task_duration: Histogram<f64>,
    pub guard_checks: Counter<u64>,
    pub hook_executions: Counter<u64>,
}

impl DemoWorkflowMetrics {
    pub fn record_task_execution(&self, task_name: &str, duration: Duration, success: bool) {
        let attributes = vec![
            KeyValue::new("task.name", task_name.to_string()),
            KeyValue::new("task.success", success),
            KeyValue::new("workflow.name", "Demo Workflow"),
        ];

        self.task_executions.add(1, &attributes);
        self.task_duration.record(duration.as_secs_f64() * 1000.0, &attributes);
    }
}
```

## Workflow Configuration

Generated `workflow.yaml` includes:

```yaml
workflow:
  id: "demo-workflow-v1"
  name: "Demo Workflow"
  version: "1.0.0"

  yawl:
    patterns_used:
      - pattern: "Basic"
        count: 2
      - pattern: "Sequence"
        count: 1
      - pattern: "ParallelSplit"
        count: 1
      - pattern: "Synchronization"
        count: 1
      - pattern: "Decision"
        count: 1

tasks:
  - id: "submit-request"
    name: "Submit Request"
    type: "Atomic"
    pattern: "Basic"
    guards:
      - name: "amount_check"
        type: "COMPARE_O_LE"
        expression: "amount <= 10000"
        max_run_len: 8

performance:
  chatman_constant:
    max_ticks: 8
    max_duration_ns: 2
```

## Weaver Registry

Generated OTEL schema:

```yaml
spans:
  - name: "demo.task.submit_request"
    brief: "Initial task that submits a request for processing"
    attributes:
      - name: "task.name"
        type: string
        requirement_level: required
      - name: "task.pattern"
        type: string
        requirement_level: required

metrics:
  - name: "demo.task.executions"
    brief: "Number of task executions"
    instrument: counter
    unit: "{execution}"
    attributes:
      - name: "task.name"
        type: string
        requirement_level: required
```

## Validation Checklist

- [ ] `weaver registry check` passes ✅ (SOURCE OF TRUTH)
- [ ] `cargo build` succeeds
- [ ] `cargo clippy` shows zero warnings
- [ ] `rustfmt --check` passes
- [ ] All tests pass (`cargo test`)
- [ ] YAML validation passes
- [ ] Generated code is deterministic (same ontology → same output)

## Testing the Workflow

```rust
#[tokio::test]
async fn test_demo_workflow() {
    // Create state machine
    let mut sm = DemoWorkflowStateMachine::new();
    assert_eq!(sm.current_state(), &DemoWorkflowState::Initial);

    // Execute workflow
    sm.transition("submit").expect("Submit");
    assert_eq!(sm.current_state(), &DemoWorkflowState::Submitted);

    sm.transition("validate").expect("Validate");
    assert_eq!(sm.current_state(), &DemoWorkflowState::Validating);

    sm.transition("process").expect("Process");
    assert_eq!(sm.current_state(), &DemoWorkflowState::Processing);

    sm.transition("request_approval").expect("Request approval");
    assert_eq!(sm.current_state(), &DemoWorkflowState::PendingApproval);

    sm.transition("approve").expect("Approve");
    assert_eq!(sm.current_state(), &DemoWorkflowState::Approved);

    assert!(sm.current_state().is_final());
}
```

## Next Steps

1. **Customize the ontology**: Modify `ggen-demo-workflow.ttl` to match your domain
2. **Regenerate code**: Run `./scripts/ggen/generate-workflows.sh`
3. **Implement business logic**: Fill in generated hook functions
4. **Add SPARQL queries**: Integrate knowledge graph queries in guards
5. **Deploy with OTEL**: Export telemetry to collector
6. **Validate with Weaver**: Ensure runtime telemetry matches schema

## Resources

- **Template Documentation**: [docs/codegen/ggen-templates.md](../docs/codegen/ggen-templates.md)
- **Quick Start**: [docs/codegen/QUICKSTART.md](../docs/codegen/QUICKSTART.md)
- **Template Files**: [templates/](../templates/)
- **Scripts**: [scripts/ggen/](../scripts/ggen/)

## Key Principles

1. **Schema-first**: Ontology defines structure, code is generated
2. **Deterministic**: Same ontology always produces same code
3. **Type-safe**: Generated Rust code is fully type-checked
4. **Observable**: Full OTEL integration out-of-the-box
5. **Validated**: Weaver validation is the source of truth

---

**Version**: 1.0.0
**Status**: Production Ready ✅
**Last Updated**: 2024-11-16
