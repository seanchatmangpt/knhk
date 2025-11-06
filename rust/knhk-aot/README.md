# knhk-aot

Ahead-of-Time (AOT) compilation guard for IR validation before execution.

## Overview

`knhk-aot` provides AOT validation for hook IR before execution, ensuring operations meet the Chatman Constant constraint (≤8 ticks). The guard validates operation types, run lengths, and operation-specific constraints to prevent invalid operations from reaching the hot path.

## Quick Start

```rust
use knhk_aot::{AotGuard, ValidationResult};

// Validate ASK_SP operation with run_len = 8
match AotGuard::validate_ir(1, 8, 0) {
    Ok(()) => println!("Valid IR"),
    Err(result) => {
        eprintln!("Validation failed: {}", AotGuard::error_message(&result));
    }
}

// Validate UNIQUE operation (run_len must be ≤ 1)
match AotGuard::validate_ir(8, 1, 0) {
    Ok(()) => println!("Valid UNIQUE operation"),
    Err(result) => {
        eprintln!("Invalid: {}", AotGuard::error_message(&result));
    }
}

// Invalid: run_len > 8
assert!(AotGuard::validate_ir(1, 9, 0).is_err());
```

## Validation Rules

### Run Length Constraint
- **All operations**: run_len ≤ 8 (Chatman Constant)
- **UNIQUE operations**: run_len ≤ 1

### Operation Type Validation
- Only operations in hot path set are allowed:
  - ASK operations (1, 3, 7)
  - COUNT operations (2, 5, 6, 9, 10, 11)
  - UNIQUE (8)
  - COMPARE operations (12-16)
  - CONSTRUCT8 (32)

### Operation-Specific Constraints
- **COUNT operations**: k threshold ≤ run_len
- **UNIQUE**: run_len must be ≤ 1
- **CONSTRUCT8**: run_len ≤ 8

## Key Features

- **IR Validation**: Validates hook IR before execution
- **Guard Enforcement**: Enforces Chatman Constant (≤8 ticks)
- **Operation Filtering**: Only allows hot path operations
- **Error Messages**: Descriptive validation error messages
- **Template Analysis**: Template analysis for CONSTRUCT8 operations
- **MPHF Generation**: Minimal Perfect Hash Function generation

## Validation Results

- **Valid**: Operation passes all checks
- **ExceedsTickBudget**: Operation exceeds 8-tick budget
- **InvalidOperation**: Operation not in hot path set
- **InvalidRunLength**: Run length > 8

## Dependencies

- `rio_turtle` (optional) - RDF parsing for template analysis
- `rio_api` (optional) - RDF API for template analysis

## Performance

- **Validation Overhead**: ~10ns per operation
- **Compile-Time**: AOT validation happens before execution
- **Runtime**: No runtime overhead (validated at compile time)

## Related Documentation

- [Technical Documentation](docs/README.md) - Detailed API reference
- [Architecture](../../docs/architecture.md) - System architecture
- [Performance](../../docs/performance.md) - Performance guide

