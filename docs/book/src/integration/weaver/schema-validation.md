# Weaver Schema Validation

Schema validation for OTEL spans and metrics.

## Overview

Weaver validates OTEL spans and metrics against schemas:
- Span schema validation
- Metric schema validation
- Attribute validation
- Error reporting

## Usage

```rust
use knhk_otel::weaver::Weaver;

let weaver = Weaver::new(config)?;
let validation_result = weaver.validate_span(span)?;
```

## Related Documentation

- [Weaver Integration](../weaver.md) - Overview
- [Live-Check Setup](live-check.md) - Setup guide
