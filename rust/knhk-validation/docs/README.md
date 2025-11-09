# knhk-validation Documentation

Validation framework for KNHK release verification and guard constraint checking.

## Overview

The `knhk-validation` crate provides comprehensive validation utilities for:
- Release validation (v0.4.0+)
- Definition of Done (DoD) checking
- Guard constraint validation (max_run_len ≤ 8)
- CLI binary and command validation
- Network component validation
- Property validation (receipt merging, IRI hashing)
- Performance validation (hot path ≤8 ticks)

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

## Core Components

### ValidationReport

Main container for validation results:

```rust
pub struct ValidationReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub warnings: usize,
    pub results: Vec<ValidationResult>,
}
```

**Methods:**
- `new()` - Create empty report
- `add_result(result)` - Add validation result
- `add_warning(message)` - Add warning (counted separately)
- `is_success()` - Returns true if no failures

### Validation Modules

#### CLI Validation (`cli_validation`)

Validates CLI binary existence and command execution:

```rust
use knhk_validation::cli_validation;

// Check if CLI binary exists
let result = cli_validation::validate_cli_binary_exists();

// Validate CLI command execution
let result = cli_validation::validate_cli_command("hook", &["--help"]);
```

**Functions:**
- `validate_cli_binary_exists()` - Checks for CLI binary in target/release or target/debug
- `validate_cli_command(command, args)` - Validates CLI command execution

#### Network Validation (`network_validation`)

Validates network components:

```rust
use knhk_validation::network_validation;

// Check HTTP client implementation
let result = network_validation::validate_http_client_exists();

// Check OTEL exporter implementation
let result = network_validation::validate_otel_exporter_exists();
```

**Functions:**
- `validate_http_client_exists()` - Validates HTTP client in EmitStage
- `validate_otel_exporter_exists()` - Validates OTEL exporter implementation

#### Configuration Validation (`configuration_validation`)

Validates configuration file parsing:

```rust
use knhk_validation::configuration_validation;

let result = configuration_validation::validate_config_file_parsing();
```

**Functions:**
- `validate_config_file_parsing()` - Validates config file parsing availability

#### Property Validation (`property_validation`)

Validates mathematical properties and constraints:

```rust
use knhk_validation::property_validation;

// Validate receipt merging properties (associative, commutative)
let result = property_validation::validate_receipt_merging_properties();

// Validate IRI hashing determinism
let result = property_validation::validate_iri_hashing_properties();

// Validate guard constraints (max_run_len ≤ 8)
let result = property_validation::validate_guard_constraints();
```

**Functions:**
- `validate_receipt_merging_properties()` - Validates receipt merging properties
- `validate_iri_hashing_properties()` - Validates IRI hashing determinism
- `validate_guard_constraints()` - Validates guard constraint enforcement

#### Performance Validation (`performance_validation`)

Validates performance characteristics:

```rust
use knhk_validation::performance_validation;

// Validate hot path performance (≤8 ticks)
let result = performance_validation::validate_hot_path_performance();

// Validate CLI latency (<100ms)
let result = performance_validation::validate_cli_latency();
```

**Functions:**
- `validate_hot_path_performance()` - Validates hot path operations complete in ≤8 ticks
- `validate_cli_latency()` - Validates CLI commands complete in <100ms

## Usage Examples

### Complete Validation Workflow

```rust
use knhk_validation::*;

#[cfg(feature = "std")]
fn main() {
    let mut report = ValidationReport::new();

    // CLI validation
    report.add_result(cli_validation::validate_cli_binary_exists());
    report.add_result(cli_validation::validate_cli_command("hook", &["--help"]));

    // Network validation
    report.add_result(network_validation::validate_http_client_exists());
    report.add_result(network_validation::validate_otel_exporter_exists());

    // Property validation
    report.add_result(property_validation::validate_guard_constraints());
    report.add_result(property_validation::validate_receipt_merging_properties());

    // Performance validation
    report.add_result(performance_validation::validate_hot_path_performance());

    // Print summary
    println!("Total: {}, Passed: {}, Failed: {}", 
             report.total, report.passed, report.failed);

    if report.is_success() {
        println!("All validation checks passed!");
    } else {
        println!("Some validation checks failed!");
    }
}
```

### Custom Validation

```rust
use knhk_validation::{ValidationReport, ValidationResult};

fn custom_validation() -> ValidationResult {
    // Perform custom check
    let check_passed = true; // Your validation logic
    
    ValidationResult {
        passed: check_passed,
        message: if check_passed {
            "Custom check passed".to_string()
        } else {
            "Custom check failed".to_string()
        },
    }
}

let mut report = ValidationReport::new();
report.add_result(custom_validation());
```

## Key Features

- **Comprehensive Coverage**: Validates CLI, network, configuration, properties, and performance
- **Modular Design**: Separate modules for different validation types
- **Report Aggregation**: Collects all validation results in a single report
- **Warning Support**: Separate tracking for warnings vs failures
- **Guard Constraint Validation**: Enforces max_run_len ≤ 8 (Chatman Constant)
- **Property Validation**: Validates mathematical properties (associativity, commutativity, determinism)

## Dependencies

- `knhk-etl` - For EmitStage validation
- `knhk-otel` - For OTEL exporter validation
- `knhk-lockchain` - For receipt property validation
- `knhk-hot` - For hot path performance validation

## Feature Flags

- `std` - Enables std library features (required for CLI validation)

## Related Documentation

- [Definition of Done](../../../docs/DEFINITION_OF_DONE.md) - DoD criteria
- [Architecture Guide](../../../docs/ARCHITECTURE.md) - 🆕 Consolidated 80/20 guide (System architecture)
- [Performance Guide](../../../docs/PERFORMANCE.md) - 🆕 Consolidated 80/20 guide (Performance optimization)
- [Architecture Reference](../../../docs/architecture.md) - Detailed architecture reference
- [Performance Reference](../../../docs/performance.md) - Detailed performance reference
- [Testing](../../../docs/testing.md) - Testing documentation
