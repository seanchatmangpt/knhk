# Epoch Containment

Epoch containment ensures all operations complete within the 8-tick time bound.

## Overview

The epoch containment law states: **μ ⊂ τ** (τ=8)

All hook evaluations must complete within 8 ticks (Chatman Constant).

## Time Bounds

### Hot Path (R1)

- **Budget**: ≤8 ticks per operation
- **Enforcement**: Guard constraints validate budget
- **Violation**: Automatic park to W1

### Warm Path (W1)

- **Budget**: More time available
- **Retry**: Exponential backoff
- **Degrade**: Fallback to cache

### Cold Path (C1)

- **Budget**: Async operations
- **Batch**: Process multiple items
- **Schedule**: Non-blocking execution

## Guard Constraints

### Max Run Length

```rust
if triples.len() > 8 {
    return Err(ProcessingError::GuardViolation(
        format!("Triple count {} exceeds max_run_len 8", triples.len())
    ));
}
```

### Performance Budget

```rust
if ticks > 8 {
    return Err(ProcessingError::PerformanceBudgetExceeded(ticks));
}
```

## Enforcement

### Pre-Admission

Work is validated before admission:

```rust
guard.validate(triples, budget)?;
scheduler.enqueue_delta(triples, tick)?;
```

### Runtime Monitoring

SLO monitoring tracks performance:

```rust
let slo = SloMonitor::new();
slo.record_latency(operation, ticks)?;
if slo.violated() {
    fiber.park(work)?;
}
```

## Related Documentation

- [Beat Scheduler](beat-scheduler.md) - Core scheduler
- [Fiber Execution](../fiber-execution.md) - Execution units
- [Hot Path](../hot-path.md) - Hot path operations

