# Failure Actions

Failure actions for runtime classes.

## Overview

Failure actions define how to handle errors:
- **R1**: Drop, Park, Escalate
- **W1**: Retry, Degrade, Escalate
- **C1**: Schedule, Queue, Escalate

## Implementation

```rust
match runtime_class {
    RuntimeClass::R1 => {
        if error.is_critical() {
            fiber.escalate(work)?;
        } else {
            fiber.park(work)?;
        }
    }
    // ... other classes
}
```

## Related Documentation

- [Runtime Classes](../runtime-classes.md) - Overview
- [Park and Escalate](../../../architecture/fiber-execution/park-escalate.md) - Mechanisms
