# KNHK Automated Self-Validation

**"Eating Our Own Dog Food"** - Using KNHKS to validate itself automatically.

## Overview

KNHKS includes automated self-validation that uses KNHKS to validate KNHKS operations. This ensures that the system is continuously validated using its own validation framework, creating a self-referential validation loop.

## Features

- **Continuous Validation**: Run validation automatically at configurable intervals
- **Weaver Integration**: Validates telemetry schemas using OpenTelemetry Weaver
- **Receipt Tracking**: Generates cryptographic receipts for all validation operations
- **OTEL Spans**: All validations generate OpenTelemetry spans for observability
- **Comprehensive Reports**: JSON reports with full validation results

## Quick Start

### One-Time Validation

```bash
# Run single validation
./scripts/automate-self-validation.sh

# Or use CLI directly
knhk validate self-validate --weaver --receipts --output validation_report.json
```

### Continuous Validation (Daemon Mode)

```bash
# Run continuous validation (every 5 minutes by default)
./scripts/automate-self-validation.sh --daemon

# Or use CLI directly
knhk validate self-validate-daemon --interval 300 --weaver --receipts --output evidence/self_validation/
```

### Environment Variables

```bash
# Configure validation interval (seconds)
export INTERVAL=300  # 5 minutes

# Configure output directory
export OUTPUT_DIR=evidence/self_validation

# Enable/disable features
export WEAVER_ENABLED=true
export RECEIPTS_ENABLED=true
```

## CLI Commands

### `validate self-validate`

Run one-time self-validation.

**Options:**
- `--weaver`: Enable Weaver schema validation
- `--receipts`: Enable receipt tracking
- `--output <path>`: Output report file path

**Example:**
```bash
knhk validate self-validate --weaver --receipts --output validation_report.json
```

### `validate self-validate-daemon`

Run continuous self-validation (daemon mode).

**Options:**
- `--interval <seconds>`: Validation interval (default: 300)
- `--weaver`: Enable Weaver schema validation
- `--receipts`: Enable receipt tracking
- `--output <path>`: Output directory for reports

**Example:**
```bash
knhk validate self-validate-daemon --interval 300 --weaver --receipts --output evidence/self_validation/
```

## What Gets Validated

The self-validation system validates:

1. **CLI Binary**: Checks if CLI binary exists and is executable
2. **CLI Commands**: Validates that CLI commands work correctly
3. **Guard Constraints**: Validates max_run_len ≤ 8 constraint
4. **Performance Constraints**: Validates hot path ≤8 ticks constraint
5. **Weaver Schema**: Validates telemetry schemas (if enabled)
6. **Receipt Generation**: Generates receipts for all operations (if enabled)

## Validation Report Format

```json
{
  "timestamp_ms": 1699292400000,
  "validation_report": {
    "total": 10,
    "passed": 9,
    "failed": 1,
    "warnings": 0,
    "results": [
      {
        "passed": true,
        "message": "CLI binary found at target/release/knhk"
      },
      {
        "passed": false,
        "message": "Guard constraint violated: run_len 9 > 8"
      }
    ]
  },
  "weaver_compliant": true,
  "weaver_violations": 0,
  "receipts_generated": 5,
  "span_id": "0x1234567890abcdef"
}
```

## Integration with CI/CD

### GitHub Actions

```yaml
- name: Run Self-Validation
  run: |
    cd rust
    cargo build --package knhk-cli --release
    ./target/release/knhk validate self-validate --weaver --receipts --output validation_report.json
```

### Cron Job

```bash
# Run validation every hour
0 * * * * /path/to/knhk validate self-validate --weaver --receipts --output /var/log/knhk/validation_$(date +\%Y\%m\%d_\%H\%M\%S).json
```

## Architecture

The self-validation system uses:

- **knhk-validation**: Validation framework for checks
- **knhk-lockchain**: Receipt storage and provenance
- **knhk-otel**: OpenTelemetry integration for spans
- **Weaver**: Schema validation for telemetry

## Receipt Tracking

When receipt tracking is enabled, each validation operation generates a receipt:

```rust
Receipt {
    cycle_id: 12345,      // 8-beat epoch cycle
    shard_id: 0,          // Shard identifier
    hook_id: 0x12345678,  // Operation identifier
    actual_ticks: 1,      // Performance ticks (≤8)
    hash_a: 0xabcdef,    // Content-addressed hash
}
```

Receipts are stored in the lockchain for cryptographic provenance.

## Weaver Validation

Weaver validation ensures that telemetry schemas match runtime behavior:

```bash
# Weaver validation is automatically run if --weaver is enabled
weaver registry check -r registry/
weaver registry live-check --registry registry/
```

## Troubleshooting

### CLI Binary Not Found

```bash
# Build CLI first
cd rust
cargo build --package knhk-cli --release
```

### Weaver Validation Fails

```bash
# Check Weaver is installed
weaver --version

# Check registry files
weaver registry check -r registry/
```

### Receipt Storage Fails

```bash
# Check lockchain storage directory permissions
ls -la evidence/self_validation/
```

## Best Practices

1. **Run Continuous Validation**: Use daemon mode for production systems
2. **Enable Weaver**: Always enable Weaver validation for schema compliance
3. **Track Receipts**: Enable receipt tracking for audit trails
4. **Monitor Reports**: Review validation reports regularly
5. **Integrate with CI/CD**: Run validation in CI/CD pipelines

## Related Documentation

- [Validation Framework](../rust/knhk-validation/README.md)
- [Lockchain Documentation](../rust/knhk-lockchain/README.md)
- [OTEL Integration](../rust/knhk-otel/README.md)
- [Weaver Integration](../docs/WEAVER_INTEGRATION.md)



