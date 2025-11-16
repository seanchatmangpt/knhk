# knhk-autonomic: MAPE-K Autonomic Knowledge Integration

**Covenant 3**: Feedback Loops Run at Machine Speed

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

`knhk-autonomic` implements the complete MAPE-K (Monitor, Analyze, Plan, Execute, Knowledge) autonomic feedback loop for self-managing workflows. It enables systems to detect problems, analyze root causes, select recovery actions, execute corrections, and learn from results—all autonomously at machine speed.

## Doctrine Alignment

### Principle: MAPE-K Autonomic Computing

**Covenant**: Covenant 3 - Feedback Loops Run at Machine Speed
**Why This Matters**: Workflows must be self-managing. No human can review problems at microsecond latencies. MAPE-K must be embedded in execution.

### Critical Constraints

- **Latency**: Hot path operations ≤ 8 ticks (Chatman Constant)
- **Autonomy**: No human approval in critical path
- **Mechanistic**: All policies are SPARQL queries (not implicit logic)
- **Observable**: All decisions emit telemetry validated by Weaver
- **Persistent**: Knowledge survives across workflow executions

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                                                     │
│  Monitor (Observe)  → Analyze (Understand)          │
│         ↑                      ↓                    │
│         └──────────────────────┘                    │
│                                                     │
│  Execute (Act)     ← Plan (Decide)                 │
│         ↑                      ↓                    │
│         └──────────────────────┘                    │
│                                                     │
│           Knowledge Base (Learn)                    │
│           - Patterns learned                        │
│           - Successes recorded                      │
│           - Predictions trained                     │
│           - Policies refined                        │
│                                                     │
└─────────────────────────────────────────────────────┘
```

## Features

### Self-Management Properties

- **Self-Healing**: Detects failures and recovers automatically
- **Self-Optimizing**: Monitors performance and improves continuously
- **Self-Configuring**: Adapts to changing conditions dynamically
- **Self-Protecting**: Detects threats and protects automatically
- **Self-Learning**: Learns from experience and improves decisions

### Components

#### Monitor
- Collects performance, reliability, resource, quality, and security metrics
- Detects anomalies in real-time
- Calculates trend directions (improving, degrading, stable)
- Emits observations for events

#### Analyze
- Matches observations to analysis rules (SPARQL pattern matching)
- Identifies root causes
- Assesses problem severity and confidence
- Recommends actions based on learned patterns

#### Plan
- Evaluates autonomic policies against analysis results
- Selects actions based on historical success rates
- Sequences actions logically
- Assesses risk of actions

#### Execute
- Executes actions in sequence
- Monitors action effects
- Captures execution output and errors
- Feeds results to knowledge base

#### Knowledge
- Stores learned patterns (what problems occur)
- Tracks success memories (what actions work when)
- Maintains feedback cycle history
- Calculates pattern reliability and action success rates
- Persists knowledge across system restarts

## Usage

### Basic Example

```rust
use knhk_autonomic::{AutonomicController, Config};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create autonomic controller
    let config = Config::default()
        .with_loop_frequency(Duration::from_secs(5))
        .with_knowledge_path("./knowledge.db");

    let mut controller = AutonomicController::new(config).await?;

    // Setup metrics, rules, policies, actions (see full example)

    // Start MAPE-K loop
    controller.start().await?;

    Ok(())
}
```

### Complete Self-Healing Example

See `examples/self_healing_workflow.rs` for a complete demonstration of:
- Setting up metrics, analysis rules, policies, and actions
- Injecting failures
- Watching MAPE-K detect, analyze, plan, and execute recovery
- Observing learning and improvement over time

Run the example:
```bash
cargo run --example self_healing_workflow
```

## Performance

All hot path operations are benchmarked to verify they meet the **Chatman Constant** (≤8 ticks):

```bash
cargo bench
```

Benchmarks include:
- Monitor metric collection
- Anomaly detection
- Analysis rule matching
- Policy evaluation
- Complete MAPE-K cycle latency

## Testing

Run the test suite:
```bash
cargo test --package knhk-autonomic
```

Tests cover:
- All MAPE-K components
- Knowledge persistence
- Success rate tracking
- Pattern learning
- Feedback cycles

## Integration

### With Workflow Engine

```rust
use knhk_autonomic::AutonomicController;
use knhk_workflow_engine::YawlEngine;

// Create workflow engine
let engine = YawlEngine::new().await?;

// Create autonomic controller
let mut controller = AutonomicController::new(config).await?;

// Connect workflow engine metrics to autonomic monitor
// ...

// Start both
tokio::spawn(async move { engine.start().await });
controller.start().await?;
```

### With OpenTelemetry

All autonomic decisions emit OpenTelemetry spans and metrics:
- Monitor observations → `autonomic.monitor.observation`
- Analysis results → `autonomic.analyze.analysis`
- Plans generated → `autonomic.plan.generated`
- Actions executed → `autonomic.execute.action`
- Knowledge updates → `autonomic.knowledge.update`

## Validation

### Weaver Schema Validation

All autonomic behavior is validated against the MAPE-K ontology:

```bash
weaver registry check -r registry/
weaver registry live-check --registry registry/
```

The implementation conforms to:
- `ontology/mape-k-autonomic.ttl` - Complete MAPE-K ontology
- `ggen-marketplace/knhk-yawl-workflows/queries/mape-k-*.sparql` - MAPE-K queries

### Chatman Constant Enforcement

Hot path operations are continuously benchmarked to ensure ≤8 tick latency:
```bash
make test-performance-v04
```

## Documentation

- [MAPE-K Autonomic Integration](../../MAPE-K_AUTONOMIC_INTEGRATION.md) - Complete specification
- [Doctrine Covenant](../../DOCTRINE_COVENANT.md) - Covenant 3 requirements
- [API Documentation](https://docs.rs/knhk-autonomic) - Full API reference

## License

MIT License - See LICENSE file for details

## Contributing

Contributions must:
1. Satisfy Covenant 3 requirements
2. Include Weaver validation tests
3. Meet Chatman Constant (≤8 ticks for hot path)
4. Include comprehensive tests
5. Emit proper OpenTelemetry instrumentation

## References

- [DOCTRINE_2027.md](../../DOCTRINE_2027.md) - Foundational principles
- [DOCTRINE_COVENANT.md](../../DOCTRINE_COVENANT.md) - Technical covenant
- [autonomic-self-healing-workflow.ttl](../../ontology/workflows/examples/autonomic-self-healing-workflow.ttl) - Reference workflow
