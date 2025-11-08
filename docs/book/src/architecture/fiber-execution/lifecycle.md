# Fiber Lifecycle

Fiber lifecycle management for per-shard execution units.

## Overview

Fibers go through distinct lifecycle phases:
1. **Creation**: Initialize fiber for shard
2. **Execution**: Execute work at tick slot
3. **Park**: Park over-budget work
4. **Escalate**: Escalate critical failures
5. **Cleanup**: Resource cleanup

## Lifecycle Phases

### Creation

```rust
let fiber = Fiber::new(shard_id);
fiber.initialize()?;
```

### Execution

```rust
let result = fiber.execute_tick(tick)?;
```

### Park

```rust
if result.over_budget {
    fiber.park(work)?;
}
```

### Escalate

```rust
if result.critical_failure {
    fiber.escalate(work)?;
}
```

### Cleanup

```rust
fiber.cleanup()?;
```

## State Management

### Fiber State

```rust
pub enum FiberState {
    Idle,
    Executing,
    Parked,
    Escalated,
}
```

## Related Documentation

- [Fiber Execution](../fiber-execution.md) - Overview
- [Tick-Based Rotation](tick-rotation.md) - Work routing
- [Park and Escalate](park-escalate.md) - Failure handling
