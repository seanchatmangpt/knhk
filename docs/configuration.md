# KNHK Configuration Guide

## Overview

KNHK uses TOML configuration files with environment variable overrides. Configuration is loaded from `~/.knhk/config.toml` (Unix) or `%APPDATA%/knhk/config.toml` (Windows).

## Configuration File Location

- **Unix/Linux/macOS**: `~/.knhk/config.toml`
- **Windows**: `%APPDATA%/knhk/config.toml`
- **Override**: Set `KNHK_CONFIG_PATH` environment variable

## Configuration Schema

### Basic Configuration

```toml
[knhk]
version = "0.5.0"
context = "default"
max_run_len = 8  # Must be ≤ 8 (guard constraint)
```

### Connector Configuration

```toml
[knhk.connectors.kafka-prod]
type = "kafka"
bootstrap_servers = ["localhost:9092"]
topic = "triples"
schema = "urn:knhk:schema:enterprise"
max_run_len = 8  # Must be ≤ 8
max_batch_size = 1000
```

### Epoch Configuration

```toml
[knhk.epochs.default]
tau = 8  # Must be ≤ 8 (guard constraint)
ordering = "deterministic"
```

## Environment Variable Overrides

Environment variables override configuration file values. Use `KNHK_` prefix:

- `KNHK_CONTEXT` - Set default context
- `KNHK_VERSION` - Override version
- `KNHK_MAX_RUN_LEN` - Override max_run_len
- `KNHK_CONNECTOR_{NAME}_TYPE` - Override connector type
- `KNHK_CONNECTOR_{NAME}_MAX_RUN_LEN` - Override connector max_run_len
- `KNHK_EPOCH_{NAME}_TAU` - Override epoch tau

### Examples

```bash
# Override context
export KNHK_CONTEXT=production

# Override connector configuration
export KNHK_CONNECTOR_KAFKA_PROD_TYPE=kafka
export KNHK_CONNECTOR_KAFKA_PROD_MAX_RUN_LEN=8

# Override epoch configuration
export KNHK_EPOCH_DEFAULT_TAU=8
```

## Configuration Priority

1. **Environment Variables** (highest priority)
2. **Configuration File** (`~/.knhk/config.toml`)
3. **Default Configuration** (lowest priority)

## Validation

Configuration is validated on load:

- `max_run_len` must be ≤ 8 (guard constraint)
- `tau` must be ≤ 8 (guard constraint)
- Connector `max_run_len` must be ≤ 8

Validation errors are reported with clear error messages.

## Error Handling

If configuration file is missing or invalid:

- Warning message is printed
- Default configuration is used
- Configuration error metric is recorded (if OTEL enabled)

## Example Configuration File

```toml
[knhk]
version = "0.5.0"
context = "production"
max_run_len = 8

[knhk.connectors.kafka-prod]
type = "kafka"
bootstrap_servers = ["kafka1:9092", "kafka2:9092"]
topic = "triples"
schema = "urn:knhk:schema:enterprise"
max_run_len = 8
max_batch_size = 1000

[knhk.connectors.salesforce-prod]
type = "salesforce"
instance_url = "https://instance.salesforce.com"
api_version = "v57.0"
object_type = "Account"
schema = "urn:knhk:schema:salesforce"
max_run_len = 8

[knhk.epochs.default]
tau = 8
ordering = "deterministic"

[knhk.epochs.batch]
tau = 8
ordering = "deterministic"
```

## CLI Integration

Configuration is automatically loaded when CLI starts:

```bash
knhk hook list  # Uses configuration from ~/.knhk/config.toml
```

Environment variables override file configuration:

```bash
KNHK_CONTEXT=test knhk hook list  # Uses "test" context
```

## API Usage

```rust
use knhk_config::{load_config, load_default_config, get_default_config_path};

// Load configuration from default path
let config_path = get_default_config_path();
let config = load_config(&config_path)?;

// Use configuration
println!("Context: {}", config.knhk.context);
println!("Max run len: {}", config.knhk.max_run_len);

// Access connector configuration
if let Some(connector) = config.connectors.get("kafka-prod") {
    println!("Connector type: {}", connector.connector_type);
}
```

## Guard Constraints

All configuration values must respect guard constraints:

- **max_run_len**: Must be ≤ 8
- **tau**: Must be ≤ 8

Violations cause configuration load to fail with validation error.

## See Also

- CLI Documentation: `docs/cli.md`
- Architecture: `docs/architecture.md`
- Performance: `docs/performance.md`

