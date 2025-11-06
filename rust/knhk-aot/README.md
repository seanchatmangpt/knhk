# knhk-aot

Ahead-of-Time (AOT) compilation guard and optimization framework for KNHK hot path operations.

## Overview

`knhk-aot` provides AOT compilation optimizations and IR validation to ensure hot path operations meet the Chatman Constant constraint (≤8 ticks / ≤2ns). It includes template analysis, prebinding, MPHF generation, and specialization optimizations.

## Quick Start

```rust
use knhk_aot::{AotGuard, ValidationResult};

// Validate hook IR before execution
match AotGuard::validate_ir(op, run_len, k) {
    Ok(()) => println!("IR validated successfully"),
    Err(result) => eprintln!("Validation failed: {}", AotGuard::error_message(&result)),
}
```

## Key Features

- **IR Validation**: Enforces Chatman Constant (≤8 ticks)
- **Template Analysis**: Separates constants from variables
- **Prebinding**: Precomputes constants for branchless execution
- **MPHF**: O(1) predicate lookup with perfect hashing
- **Specialization**: Operation-specific optimizations

## Documentation

For detailed documentation, see [docs/README.md](docs/README.md).

## Related Documentation

- [Architecture](../../docs/architecture.md) - System architecture
- [Performance](../../docs/performance.md) - Performance guide
- [Hot Path](../knhk-hot/docs/README.md) - Hot path operations
