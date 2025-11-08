# SLO Monitoring

Service Level Objective (SLO) monitoring for runtime classes.

## Overview

SLO monitoring tracks:
- Latency per runtime class
- Error rates
- Performance budgets
- SLO violations

## Implementation

```rust
use knhk_etl::slo::SloMonitor;

let slo = SloMonitor::new();
slo.record_latency(operation, ticks)?;
if slo.violated() {
    fiber.park(work)?;
}
```

## Related Documentation

- [Runtime Classes](../runtime-classes.md) - Overview
- [R1 Hot Path](r1.md) - Hot path
- [W1 Warm Path](w1.md) - Warm path
