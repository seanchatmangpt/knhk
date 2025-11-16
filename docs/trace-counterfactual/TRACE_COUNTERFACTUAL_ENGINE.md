# Trace-Indexed Counterfactual Engine

## Overview

The Trace-Indexed Counterfactual Engine provides deterministic replay and "what-if" analysis for KNHK's MAPE-K autonomic framework. It enables answering questions like "What would μ have done under different policy/ontology?"

## Architecture

```
TraceId = BLAKE3(O_segment || Σ_snapshot || Q_version)
```

### Components

1. **Observable Segment (O)**: Time-bounded sequence of monitor events
2. **Ontology Snapshot (Σ)**: KnowledgeBase state (goals, rules, facts, policies)
3. **Doctrine Configuration (Q)**: Policy lattice element and runtime configuration
4. **TraceId**: Unique 256-bit identifier for execution trace

## Key Features

- **Deterministic Replay**: Bit-for-bit identical re-execution
- **Counterfactual Simulation**: "What-if" analysis with alternative Σ or Q
- **Comprehensive Diff Analysis**: Actions, invariants, SLOs, timing
- **Lock-Free Hot Path**: Zero-cost trace ID generation
- **Efficient Storage**: LRU eviction, memory-mapped support
- **Pure Functional Design**: No global state

## Usage Examples

### 1. Capture Execution Trace

```rust
use knhk_workflow_engine::autonomic::{
    ExecutionTrace, ObservableSegment, OntologySnapshot,
    DoctrineConfig, TraceStorage, MonitorEvent, KnowledgeBase
};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Create observable segment
    let mut o_segment = ObservableSegment::new(1000, 2000);
    o_segment.add_event(MonitorEvent::new(
        "latency".to_string(),
        150.0,
        "monitor".to_string(),
    ));

    // Capture ontology snapshot
    let kb = KnowledgeBase::new();
    // ... add goals, rules, facts, policies ...
    let sigma = OntologySnapshot::from_knowledge_base(&kb).await;

    // Configure doctrine
    let q = DoctrineConfig::default();

    // Create execution trace
    let trace = ExecutionTrace::new(o_segment, sigma, q).unwrap();
    let trace_id = trace.id;

    // Store trace
    let storage = Arc::new(TraceStorage::new(100));
    storage.store(trace).await.unwrap();

    println!("Trace ID: {}", trace_id);
}
```

### 2. Deterministic Replay

```rust
use knhk_workflow_engine::autonomic::{
    CounterfactualEngine, CounterfactualScenario, TraceStorage
};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let storage = Arc::new(TraceStorage::new(100));
    let engine = CounterfactualEngine::new(storage.clone());

    // Retrieve trace ID from previous execution
    let trace_id = /* ... */;

    // Create replay scenario
    let scenario = CounterfactualScenario::replay(trace_id);

    // Execute replay
    let result = engine.execute(scenario).await.unwrap();

    // Verify exact replay
    if result.is_exact_replay() {
        println!("✓ Replay was bit-for-bit identical");
    } else {
        println!("✗ Replay diverged (non-deterministic behavior detected)");
    }
}
```

### 3. Counterfactual "What-If" Analysis

```rust
use knhk_workflow_engine::autonomic::{
    CounterfactualEngine, CounterfactualScenario, KnowledgeBase,
    OntologySnapshot, Goal, GoalType
};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let storage = Arc::new(TraceStorage::new(100));
    let engine = CounterfactualEngine::new(storage.clone());

    let trace_id = /* ... */;

    // Create alternative ontology with relaxed goal
    let kb_alternative = KnowledgeBase::new();
    let relaxed_goal = Goal::new(
        "relaxed_latency".to_string(),
        GoalType::Performance,
        "latency".to_string(),
        200.0, // Relaxed from 100ms to 200ms
    );
    kb_alternative.add_goal(relaxed_goal).await.unwrap();

    let sigma_alternative = OntologySnapshot::from_knowledge_base(&kb_alternative).await;

    // Create counterfactual scenario
    let scenario = CounterfactualScenario::with_ontology(
        trace_id,
        sigma_alternative,
        "What if we had relaxed latency goal?".to_string(),
    );

    // Execute counterfactual
    let result = engine.execute(scenario).await.unwrap();

    // Analyze results
    println!("Action changes: {}", result.has_action_changes());
    println!("Invariants preserved: {}", result.invariants_preserved());
    println!("SLO improved: {}", result.slo_improved());
    println!("Speedup: {:.2}x", result.timing_comparison.speedup);
}
```

### 4. Policy Lattice Comparison

```rust
use knhk_workflow_engine::autonomic::{
    CounterfactualScenario, DoctrineConfig
};

#[tokio::main]
async fn main() {
    let trace_id = /* ... */;

    // Original doctrine: strict policy
    // Counterfactual: relaxed policy

    let mut q_relaxed = DoctrineConfig::new("1.0.0".to_string(), "relaxed".to_string());
    q_relaxed.set_config("max_instances".to_string(), serde_json::json!(20));
    q_relaxed.set_feature("auto_scaling".to_string(), true);

    let scenario = CounterfactualScenario::with_doctrine(
        trace_id,
        q_relaxed,
        "What if auto-scaling was enabled?".to_string(),
    );

    let result = engine.execute(scenario).await.unwrap();

    println!("Original actions: {}", result.original_trace.execution_results.len());
    println!("Counterfactual actions: {}", result.counterfactual_trace.execution_results.len());
    println!("Action diff: {:+}", result.action_diff.count_diff);
}
```

### 5. Comprehensive Analysis

```rust
use knhk_workflow_engine::autonomic::{
    CounterfactualScenario, KnowledgeBase, OntologySnapshot, DoctrineConfig
};

#[tokio::main]
async fn main() {
    let trace_id = /* ... */;

    // Create complete alternative configuration
    let kb_alt = KnowledgeBase::new();
    // ... configure alternative goals, rules, policies ...
    let sigma_alt = OntologySnapshot::from_knowledge_base(&kb_alt).await;

    let q_alt = DoctrineConfig::new("2.0.0".to_string(), "experimental".to_string());
    // ... configure alternative doctrine ...

    let scenario = CounterfactualScenario::full_counterfactual(
        trace_id,
        sigma_alt,
        q_alt,
        "Complete alternative configuration".to_string(),
    );

    let result = engine.execute(scenario).await.unwrap();

    // Detailed analysis
    println!("\n=== Action Diff Analysis ===");
    println!("Original only: {}", result.action_diff.original_only.len());
    println!("Counterfactual only: {}", result.action_diff.counterfactual_only.len());
    println!("Common: {}", result.action_diff.common.len());
    println!("Change %: {:.1}%", result.action_diff.change_percentage());

    println!("\n=== Invariant Checks ===");
    println!("Goal invariants: {}", result.invariant_checks.goal_invariants.len());
    println!("Policy invariants: {}", result.invariant_checks.policy_invariants.len());
    println!("Violations: {}", result.invariant_checks.violation_count());
    println!("All preserved: {}", result.invariant_checks.all_preserved());

    println!("\n=== SLO Analysis ===");
    println!("Improved: {}", result.slo_analysis.improved);
    println!("Improvement: {:.1}%", result.slo_analysis.improvement_pct);

    println!("\n=== Timing Comparison ===");
    println!("Original: {}µs", result.timing_comparison.tau_original_us);
    println!("Counterfactual: {}µs", result.timing_comparison.tau_counterfactual_us);
    println!("Speedup: {:.2}x", result.timing_comparison.speedup);
    println!("Faster: {}", result.timing_comparison.is_faster());
}
```

## Performance Characteristics

### Hot Path Cost

- **TraceId Generation**: ~50-100µs (BLAKE3 hashing)
- **Trace Storage**: O(1) insertion with LRU eviction
- **Trace Lookup**: O(1) HashMap lookup
- **Lock-Free**: All operations use atomic primitives or immutable data

### Benchmarks

Run benchmarks with:

```bash
cargo bench --bench trace_counterfactual_benchmark
```

Expected results:
- TraceId generation: ~50-100µs
- Trace storage (10 traces): ~10-20µs
- Trace lookup: ~1-5µs
- Replay execution: ~500-1000µs
- Counterfactual simulation: ~1-2ms

## Integration with MAPE-K

### Monitor Integration

```rust
use knhk_workflow_engine::autonomic::{
    Monitor, ObservableSegment, TraceStorage
};

// Capture monitor events for trace
let mut o_segment = ObservableSegment::new(start_time, end_time);

// Monitor emits events
let events = monitor.collect_once().await.unwrap();
for event in events {
    o_segment.add_event(event);
}
```

### Planner Integration

```rust
use knhk_workflow_engine::autonomic::{
    Planner, CounterfactualEngine, CounterfactualScenario
};

// Shadow planning: run counterfactual before committing
let original_plan = planner.plan(&analysis).await?;

// Simulate counterfactual with alternative policy
let scenario = CounterfactualScenario::with_doctrine(
    current_trace_id,
    alternative_doctrine,
    "Shadow plan analysis".to_string(),
);

let cf_result = engine.execute(scenario).await?;

// Choose best plan based on counterfactual analysis
if cf_result.slo_improved() && cf_result.invariants_preserved() {
    // Use counterfactual plan
} else {
    // Use original plan
}
```

## Design Principles

1. **Determinism**: Same inputs always produce same outputs
2. **No Global State**: Pure functional design
3. **Zero-Copy**: Minimize allocations in hot path
4. **Lock-Free**: Use atomic primitives and immutable data
5. **Provable Properties**: Schema-first validation with Weaver
6. **Telemetry-Driven**: All operations emit OTEL spans

## Receipt Integration

Traces can be connected to KNHK's receipt system for complete provenance:

```rust
use knhk_hot::{ReceiptDelta, DeltaComposer};

// Create receipt delta for trace
let receipt_delta = ReceiptDelta {
    hash: trace_id.0[0..4].try_into().unwrap(),
    timestamp: trace.timestamp_ms,
    tick: current_tick,
};

// Fold into receipt system
delta_composer.compose_delta(&receipt_delta);
```

## Testing

Run tests:

```bash
# Unit tests (in trace_index.rs and counterfactual.rs)
cargo test --lib autonomic::trace_index
cargo test --lib autonomic::counterfactual

# Integration tests
cargo test --test trace_replay_tests
cargo test --test counterfactual_analysis_tests
```

## Telemetry

All operations emit OpenTelemetry spans:

- `trace.index.generate_id` - TraceId generation
- `trace.storage.store` - Trace storage
- `trace.storage.retrieve` - Trace retrieval
- `counterfactual.execute` - Counterfactual execution
- `counterfactual.replay` - Replay execution
- `counterfactual.simulate` - Counterfactual simulation

Monitor with:

```bash
weaver registry live-check --registry registry/
```

## Future Enhancements

1. **Persistent Storage**: Memory-mapped files for large trace collections
2. **Distributed Tracing**: Cross-service trace correlation
3. **ML Integration**: Learn optimal policies from counterfactual analysis
4. **Batch Analysis**: Bulk counterfactual comparison
5. **Visualization**: Interactive trace diff UI

## References

- MAPE-K Framework: `/rust/knhk-workflow-engine/src/autonomic/`
- Receipt System: `/rust/knhk-hot/src/receipt_kernels.rs`
- YAWL Patterns: `/docs/yawl.txt`
- Dark Matter 80/20: `/rust/knhk-connectors/src/dark_matter.rs`

## License

MIT - See LICENSE file for details
