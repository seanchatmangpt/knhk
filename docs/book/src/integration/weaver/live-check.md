# Weaver Live-Check Setup

Weaver live-check validation for OpenTelemetry telemetry.

## Overview

Weaver provides live-check validation for OTEL spans and metrics:
- Schema validation
- OTEL correlation
- Real-time validation
- Error diagnostics

## Setup

### 1. Install Weaver

```bash
# Install Weaver CLI
npm install -g @weaver-ai/cli
```

### 2. Configure Weaver

```toml
[weaver]
enabled = true
endpoint = "https://api.weaver.ai"
api_key = "your-api-key"
```

### 3. Enable Live-Check

```rust
use knhk_otel::weaver::Weaver;

let weaver = Weaver::new(config)?;
weaver.enable_live_check()?;
```

## Related Documentation

- [Weaver Integration](../weaver.md) - Overview
- [Schema Validation](schema-validation.md) - Schema validation
- [OTEL Correlation](otel-correlation.md) - OTEL correlation
