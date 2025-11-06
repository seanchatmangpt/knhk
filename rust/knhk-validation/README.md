# knhk-validation

Validation framework for KNHK release verification and guard constraint checking.

## Overview

`knhk-validation` provides comprehensive validation utilities for release verification, Definition of Done (DoD) checking, guard constraint validation, CLI validation, network component validation, and performance validation.

## Quick Start

```rust
use knhk_validation::{ValidationReport, ValidationResult};

// Create validation report
let mut report = ValidationReport::new();

// Add validation results
report.add_result(ValidationResult {
    passed: true,
    message: "Check passed".to_string(),
});

// Check if all validations passed
if report.is_success() {
    println!("All checks passed!");
}
```

## Key Features

- **Release Validation**: Comprehensive v0.4.0+ validation
- **DoD Checking**: Definition of Done validation
- **Guard Constraints**: Enforces max_run_len ≤ 8
- **CLI Validation**: Binary and command validation
- **Property Validation**: Mathematical property checks

## Documentation

For detailed documentation, see [docs/README.md](docs/README.md).

## Related Documentation

- [Definition of Done](../../docs/DEFINITION_OF_DONE.md) - DoD criteria
- [Architecture](../../docs/architecture.md) - System architecture
- [Testing](../../docs/testing.md) - Testing documentation
- [Weaver Analysis and Learnings](../../docs/WEAVER_ANALYSIS_AND_LEARNINGS.md) - Policy engine patterns and validation improvements
