# Doctrine-Aware Failure Modes: Architecture Documentation

## Overview

The Doctrine-Aware Failure Modes system implements safe ceiling behavior for KNHK's MAPE-K autonomic loop. When the autonomic system itself becomes degraded or unhealthy, the system automatically limits its scope of action to prevent compounding problems.

## Core Philosophy

**"What happens when MAPE-K is degraded?"**

Traditional autonomic systems assume their own decision-making apparatus is reliable. KNHK takes a more defensive stance:

1. **Do less, not more, when uncertain** - If we can't trust our analysis, we shouldn't make aggressive changes
2. **Prefer safety over throughput when degraded** - Better to run conservatively than risk catastrophic failures
3. **Explicit failure > silent incorrect behavior** - Mode changes are observable and receipted
4. **Self-limiting behavior** - The system constrains itself when it detects its own degradation

## Mode Hierarchy

```
┌──────────────────────────────────────────────────┐
│ Normal Mode                                      │
│ - Full MAPE-K loop operational                   │
│ - All adaptations allowed                        │
│ - Complex actions (scaling, migration, ΔΣ)       │
│ - Overlays permitted                             │
└──────────────────────────────────────────────────┘
                    ↓ (health degrades)
┌──────────────────────────────────────────────────┐
│ Conservative Mode                                │
│ - Runtime tuning only                            │
│ - No structural changes                          │
│ - No ΔΣ modifications                            │
│ - Limited to low-risk actions                    │
│ - Alerting on degradation                        │
└──────────────────────────────────────────────────┘
                    ↓ (health critical)
┌──────────────────────────────────────────────────┐
│ Frozen Mode                                      │
│ - Read-only observation                          │
│ - Alerting and receipt emission only             │
│ - No adaptations of any kind                     │
│ - Fail-safe mode                                 │
└──────────────────────────────────────────────────┘
```

## Health Signal Architecture

Each MAPE-K component emits health signals that the ModeManager aggregates:

### Monitor Health
- **Metric**: Completeness (% of expected metrics received)
- **Indicators**: Collection success rate, metric staleness
- **Threshold**: <0.6 → Conservative, <0.3 → Frozen

### Analyzer Health
- **Metric**: Confidence in anomaly detection
- **Indicators**: Analysis quality, goal evaluation success
- **Threshold**: <0.6 → Conservative, <0.3 → Frozen

### Planner Health
- **Metric**: Viability (can generate valid plans)
- **Indicators**: Plan generation success, action validity
- **Threshold**: Planner failure degrades to Conservative

### Executor Health
- **Metric**: Reliability (action success rate)
- **Indicators**: Execution success rate, error frequency
- **Threshold**: High failure rate signals system instability

### Knowledge Staleness
- **Metric**: Data age (ms since last update)
- **Indicators**: Fact timestamps, update frequency
- **Impact**: Stale data degrades overall health score

## Mode Derivation Algorithm

```rust
// Weighted health score calculation
overall_score =
    monitor_completeness * 0.3 +
    analyzer_confidence * 0.3 +
    planner_viability * 0.2 +
    executor_reliability * 0.2

// Mode determination
if monitor_completeness < 0.3 || analyzer_confidence < 0.3 {
    return Frozen;  // Critical component failure
}

if overall_score < 0.3 {
    return Frozen;  // Overall critical
}

if overall_score < 0.6 {
    return Conservative;  // Degraded
}

if monitor_completeness < 0.6 || analyzer_confidence < 0.6 {
    return Conservative;  // Key component degraded
}

return Normal;  // Healthy
```

## Mode-Aware Policy Lattice

### Action Annotations

Each action type is statically annotated with its minimum required mode:

```rust
// Structural changes require Normal mode
ScaleInstances → MinimumMode::Normal
MigrateRuntime → MinimumMode::Normal

// Safe runtime tuning allowed in Conservative
AdjustResources → MinimumMode::Conservative
OptimizePattern → MinimumMode::Conservative
Cancel → MinimumMode::Conservative
Compensate → MinimumMode::Conservative

// Unknown custom actions default to Normal (fail-safe)
Custom { unknown } → MinimumMode::Normal
```

### Policy Filtering

Actions are filtered before execution:

1. **Pre-filtering**: Planner generates actions based on analysis
2. **Mode check**: Current mode is checked against each action's minimum requirement
3. **Rejection**: Actions that don't meet mode requirements are rejected
4. **Telemetry**: Rejected actions are logged with rationale
5. **Execution**: Only approved actions are executed

### Rejection Rationales

```rust
RejectedAction {
    action: ScaleInstances { delta: 2 },
    current_mode: Conservative,
    required_mode: Normal,
    reason: "Scaling changes system topology and requires full confidence"
}
```

## Mode Changes and Observability

### Automatic Degradation

Mode changes happen automatically based on health signals:

```rust
// Monitor fails → Conservative
update_health(HealthSignal { component: Monitor, score: 0.5 })
// → Mode: Normal → Conservative

// Analyzer fails → Frozen
update_health(HealthSignal { component: Analyzer, score: 0.2 })
// → Mode: Conservative → Frozen
```

### Manual Override

Operators can force modes for maintenance or testing:

```rust
// Force frozen mode (ignore health signals)
mode_manager.set_manual_override(AutonomicMode::Frozen).await;

// Clear override (resume automatic management)
mode_manager.clear_manual_override().await;
```

### Mode Change Events

All mode changes emit structured events:

```rust
ModeChangeEvent {
    from: Normal,
    to: Conservative,
    reason: "Health-driven (monitor: 0.55, analyzer: 0.60)",
    metrics: HealthMetrics { overall_score: 0.58, ... },
    timestamp_ms: 1699564234567,
    manual_override: false,
}
```

### Telemetry

Mode changes produce observable telemetry:

```
WARN mode.from=Normal mode.to=Conservative mode.reason="Health-driven" \
     mode.manual=false "Autonomic mode degraded"
```

## Integration with MAPE-K Loop

### Cycle Execution

```rust
async fn execute_cycle(...) -> CycleResult {
    // 1. Check current mode
    let mode = mode_manager.current_mode().await;

    // 2. In Frozen mode, only observe
    if mode == Frozen {
        return Ok(CycleResult::observation_only());
    }

    // 3. MONITOR: Collect metrics, emit health signal
    let monitor_health = calculate_monitor_health();
    mode_manager.update_health(monitor_health).await;

    // 4. ANALYZE: Detect anomalies, emit confidence signal
    let analysis = analyzer.analyze().await;
    let analyzer_confidence = analysis.health.to_score();
    mode_manager.update_health(analyzer_confidence).await;

    // 5. PLAN: Generate adaptation plan, emit viability signal
    let plan = planner.plan(&analysis).await;
    let planner_viability = plan.is_some() ? 1.0 : 0.5;
    mode_manager.update_health(planner_viability).await;

    // 6. Filter actions by current mode
    let execution_mode = mode_manager.current_mode().await;
    let (allowed, rejected) = policy_filter
        .filter_with_rejected(&plan.actions, execution_mode);

    // 7. EXECUTE: Only allowed actions
    let results = executor.execute(&allowed).await;
    let executor_reliability = results.success_rate();
    mode_manager.update_health(executor_reliability).await;

    // 8. Return cycle result with mode context
    Ok(CycleResult {
        mode: execution_mode,
        allowed: allowed.len(),
        rejected: rejected.len(),
        ...
    })
}
```

## Design Patterns

### Fail-Safe Defaults

- Unknown action types → require Normal mode
- Missing health signals → assume degraded
- Stale health data → filtered out (age > 60s)
- Parse errors in conditions → reject action
- Executor failures → degrade health score

### Safe Ceiling Principle

The system self-limits when uncertain:

```
Healthy System (Mode: Normal)
├─ Monitoring: ✓ Complete
├─ Analysis: ✓ Confident
├─ Planning: ✓ Viable
└─ Actions: All allowed (scaling, migration, optimization)

Degraded System (Mode: Conservative)
├─ Monitoring: ⚠ Incomplete (60% coverage)
├─ Analysis: ⚠ Low confidence
├─ Planning: ✓ Can generate plans
└─ Actions: Only safe runtime tuning (no scaling, no migration)

Critical System (Mode: Frozen)
├─ Monitoring: ✗ Failed (20% coverage)
├─ Analysis: ✗ Cannot analyze reliably
├─ Planning: ⚠ Uncertain plans
└─ Actions: NONE (observation only)
```

### Observable Degradation

All mode-related events are observable:

1. **Health signals** → logged at debug level
2. **Mode changes** → logged at warn/info with full context
3. **Rejected actions** → logged at warn with rationale
4. **Mode history** → queryable via mode_manager.get_history()
5. **Health metrics** → queryable via mode_manager.get_health_metrics()

## Testing Strategy

### Unit Tests

- Mode hierarchy and severity ordering
- Health signal calculation
- Action filtering by mode
- Mode transition logic
- Manual override behavior

### Integration Tests

- Automatic degradation on component failure
- Mode recovery on health improvement
- Action rejection in degraded modes
- Frozen mode prevents all adaptations
- Mode change history tracking

### Property Tests

- Mode changes are monotonic (don't skip levels without cause)
- Health improvements always allow more actions
- Rejected actions always have valid rationale
- Mode changes always emit telemetry

## API Usage Examples

### Basic Mode Management

```rust
// Create controller with mode management
let controller = MapeKController::new(config, monitor);

// Check current mode
let mode = controller.autonomic_mode().await;
println!("Current mode: {}", mode);

// Get mode manager for advanced operations
let mode_manager = controller.mode_manager();
```

### Health Signal Reporting

```rust
// Report component health
mode_manager.update_health(
    HealthSignal::new(ComponentType::Monitor, 0.85)
        .with_details("95% of expected metrics received".to_string())
).await?;
```

### Manual Mode Control

```rust
// Force frozen mode for maintenance
mode_manager.set_manual_override(AutonomicMode::Frozen).await?;

// Perform maintenance...

// Resume automatic mode management
mode_manager.clear_manual_override().await?;
```

### Querying Mode History

```rust
// Get mode change history
let history = mode_manager.get_history().await;
for event in history {
    println!(
        "{} -> {}: {} (manual: {})",
        event.from, event.to, event.reason, event.manual_override
    );
}
```

### Custom Action Annotations

```rust
// Create filter with custom annotations
let mut filter = ModePolicyFilter::new();

// Add annotation for safe custom action
filter.add_annotation(ActionAnnotation {
    action_pattern: ActionPattern::Custom {
        name: "logging".to_string(),
    },
    minimum_mode: MinimumMode::Conservative,
    rationale: "Logging is safe even in degraded mode".to_string(),
});
```

## Performance Considerations

### Lock-Free Where Possible

- Health signal timestamps use `SystemTime` (no locks)
- Mode checks are read-heavy (RwLock optimizes for this)
- Signal aggregation happens infrequently (per MAPE-K cycle)

### Minimal Overhead

- Mode check: O(1) read lock
- Health update: O(1) write lock + O(components) aggregation
- Action filtering: O(actions) with constant-time pattern matching

### Scalability

- Health signals are per-component (bounded by MAPE-K components: 5)
- Mode history is bounded (can be trimmed in production)
- Policy filter uses HashMap for O(1) annotation lookup

## Future Enhancements

### Adaptive Thresholds

Currently, mode thresholds are static (0.3 for Frozen, 0.6 for Conservative). Future versions could:

- Learn optimal thresholds from historical data
- Adjust thresholds based on workload criticality
- Use different thresholds for different system phases

### Gradual Mode Transitions

Instead of instant mode changes, implement gradual transitions:

- "Entering Conservative mode" (grace period before full restriction)
- Hysteresis to prevent mode flapping
- Staged action filtering (progressively restrict actions)

### Mode-Specific Telemetry

Different telemetry schemas for each mode:

- Normal: Full detailed tracing
- Conservative: Reduced sampling, key metrics only
- Frozen: Minimal overhead, emergency alerts only

### Context-Aware Policies

Different action policies based on:

- Tenant priority (gold tier gets Normal mode longer)
- Workflow criticality (critical workflows exempt from some restrictions)
- Time of day (more conservative during business hours)

## Related Systems

- **Dark Matter Tracker**: Provides coverage metrics that inform Monitor health
- **Session Adapter**: Per-session adaptation respects global mode
- **Trace Index**: Mode changes are indexed for counterfactual analysis
- **OTEL Integration**: Health signals and mode changes emitted as spans

## References

1. **MAPE-K Framework**: IBM Autonomic Computing reference model
2. **Safe Degradation**: Netflix Chaos Engineering principles
3. **Circuit Breaker Pattern**: Martin Fowler's resilience patterns
4. **Fail-Safe Defaults**: Security engineering best practices
