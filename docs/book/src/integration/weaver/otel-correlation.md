# Weaver OTEL Correlation

OTEL correlation for error diagnostics.

## Overview

Weaver correlates OTEL spans with errors:
- Error span correlation
- Trace context propagation
- Error diagnostics
- Root cause analysis

## Usage

```rust
use knhk_otel::weaver::Weaver;

let weaver = Weaver::new(config)?;
let correlation = weaver.correlate_error(error, span)?;
```

## Related Documentation

- [Weaver Integration](../weaver.md) - Overview
- [Live-Check Setup](live-check.md) - Setup guide
