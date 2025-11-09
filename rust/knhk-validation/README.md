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
- **Advisor Pattern**: Pluggable validation advisors (Guard, Performance, Receipt)
  - Inspired by OpenTelemetry Weaver's advisor architecture
  - See `advisor` module for details
- **Diagnostic System**: Structured diagnostics with rich context and multiple output formats
  - ANSI, JSON, and GitHub Workflow formats
  - See `diagnostics` module for details
- **CLI Validation**: Binary and command validation
- **Property Validation**: Mathematical property checks
- **Policy Engine**: Rego-based policy engine for custom validation rules (optional)

## Documentation

For detailed documentation, see [docs/README.md](docs/README.md).

## Related Documentation

- [Definition of Done](../../docs/DEFINITION_OF_DONE.md) - DoD criteria
- [Architecture Guide](../../docs/ARCHITECTURE.md) - 🆕 Consolidated 80/20 guide (System architecture)
- [Architecture Reference](../../docs/architecture.md) - Detailed architecture reference
- [Testing](../../docs/testing.md) - Testing documentation
- [Weaver Integration](../../docs/WEAVER_INTEGRATION.md) - Weaver patterns integration (Advisor & Diagnostic patterns)
