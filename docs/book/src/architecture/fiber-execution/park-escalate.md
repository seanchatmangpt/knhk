# Park and Escalate

Park and escalate mechanisms for handling over-budget work and failures.

## Overview

When work exceeds the tick budget or encounters failures:
- **Park**: Move work to W1 warm path for retry
- **Escalate**: Move critical failures to supervisor

## Park Mechanism

### When to Park

Work is parked when:
- Tick budget exceeded (ticks > 8)
- Performance SLO violated
- Non-critical failure occurred

### Park Implementation

```rust
if ticks > 8 {
    fiber.park(work)?;
    park_manager.enqueue_w1(work)?;
}
```

### W1 Processing

Parked work is processed in W1:
- More time budget available
- Can break into smaller chunks
- Retry with exponential backoff

## Escalate Mechanism

### When to Escalate

Work is escalated when:
- Critical failure occurred
- Unrecoverable error
- Requires supervisor intervention

### Escalate Implementation

```rust
if error.is_critical() {
    fiber.escalate(work)?;
    supervisor.handle(work)?;
}
```

## Failure Actions

### R1 (Hot Path)

- **Drop**: Drop Δ, emit receipt (acceptable data loss)
- **Park**: Park Δ to W1 (retry with more budget)
- **Escalate**: Escalate to supervisor (requires intervention)

### W1 (Warm Path)

- **Retry**: Retry with exponential backoff
- **Degrade**: Degrade to cached result
- **Escalate**: Escalate to supervisor if max retries exceeded

### C1 (Cold Path)

- **Schedule**: Schedule async task (non-blocking)
- **Queue**: Queue for batch processing
- **Escalate**: Escalate to supervisor if queue full

## Related Documentation

- [Fiber Execution](../fiber-execution.md) - Overview
- [Lifecycle](lifecycle.md) - Fiber lifecycle
- [Runtime Classes](../../implementation/runtime-classes/failure-actions.md) - Failure actions
