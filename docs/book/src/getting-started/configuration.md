# Configuration

Configuration guide for KNHK.

## Overview

KNHK uses configuration files for:
- Runtime settings
- Feature flags
- Performance tuning
- Integration settings

## Configuration Files

### Cargo.toml

Workspace and crate configuration:

```toml
[workspace]
members = [
    "knhk-etl",
    "knhk-hot",
    # ... other crates
]
```

### Config Files

- `config/default.toml` - Default configuration
- `config/production.toml` - Production settings
- `config/development.toml` - Development settings

## Configuration Options

### Performance Settings

```toml
[performance]
tick_budget = 8
max_run_len = 8
cache_size = 1024
```

### Integration Settings

```toml
[integration]
weaver_enabled = true
otel_enabled = true
lockchain_enabled = true
```

## Related Documentation

- [Configuration Reference](../../reference/configuration.md) - Complete reference
- [Dependency Configuration](../../reference/dependency-configuration.md) - Dependencies
