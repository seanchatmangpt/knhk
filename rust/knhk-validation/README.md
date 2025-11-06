# knhk-validation

Validation framework for release validation and property checking.

## Overview

`knhk-validation` provides a validation framework for verifying KNHK releases, including CLI validation, network validation, configuration validation, property validation, and performance validation. The crate generates validation reports with pass/fail status for each check.

## Quick Start

```rust
use knhk_validation::{ValidationReport, ValidationResult};

// Create validation report
let mut report = ValidationReport::new();

// Add validation results
report.add_result(ValidationResult {
    passed: true,
    message: "CLI binary found".to_string(),
});

report.add_result(ValidationResult {
    passed: false,
    message: "Configuration file missing".to_string(),
});

// Check overall status
if report.is_success() {
    println!("All validations passed!");
} else {
    println!("{} validations failed", report.failed);
}
```

## Validation Categories

### CLI Validation (`cli_validation` module)
- Binary existence checks
- Command execution tests
- Parameter validation

### Network Validation (`network_validation` module)
- HTTP client availability
- OTEL exporter existence
- Network connectivity

### Configuration Validation (`configuration_validation` module)
- Config file parsing
- Schema validation
- Environment variable checks

### Property Validation (`property_validation` module)
- Receipt merging properties (associativity, commutativity)
- IRI hashing properties (determinism)
- Guard constraint properties (max_run_len ≤ 8)

### Performance Validation (`performance_validation` module)
- Hot path performance (≤8 ticks)
- CLI latency (<100ms)
- Throughput benchmarks

## Key Features

- **Validation Reports**: Structured pass/fail reporting
- **Property Checking**: Mathematical property validation
- **Performance Validation**: Latency and throughput checks
- **CLI Testing**: Command-line interface validation
- **Network Testing**: Connectivity and service availability

## Usage Example

```rust
use knhk_validation::cli_validation;

// Validate CLI binary exists
let result = cli_validation::validate_cli_binary_exists();
if !result.passed {
    eprintln!("Validation failed: {}", result.message);
}

// Validate CLI command works
let result = cli_validation::validate_cli_command("boot", &["init"]);
if result.passed {
    println!("CLI command validated");
}
```

## Dependencies

- `knhk-etl` (optional) - For ETL validation
- `knhk-otel` (optional) - For OTEL validation
- `knhk-lockchain` (optional) - For receipt validation
- `knhk-hot` (optional) - For hot path validation

## Related Documentation

- [Technical Documentation](docs/README.md) - Detailed API reference
- [Architecture](../../docs/architecture.md) - System architecture
- [Definition of Done](../../docs/DEFINITION_OF_DONE.md) - DoD criteria

