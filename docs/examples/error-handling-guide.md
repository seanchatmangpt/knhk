# KNHK Error Handling Guide

Quick reference for implementing the new error hierarchy.

## Table of Contents
1. [Quick Start](#quick-start)
2. [Common Patterns](#common-patterns)
3. [FFI Integration](#ffi-integration)
4. [OTEL Integration](#otel-integration)
5. [Migration Examples](#migration-examples)

## Quick Start

### Adding Error Types to Your Crate

```toml
# Cargo.toml
[dependencies]
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"] }
knhk-core = { path = "../knhk-core" }  # For OtelContext
```

### Basic Error Definition

```rust
use thiserror::Error;
use serde::{Deserialize, Serialize};

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum MyError {
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    #[error("Processing failed: {reason}")]
    ProcessingFailed { reason: String },
}

pub type MyResult<T> = std::result::Result<T, MyError>;
```

## Common Patterns

### 1. Replace unwrap() with ?

**Before:**
```rust
fn process_data(data: &[u8]) {
    let value = parse(data).unwrap();  // ❌ Panic on error
    transform(value).unwrap();          // ❌ No context
}
```

**After:**
```rust
use knhk_etl::error::{EtlResult, IngestError};

fn process_data(data: &[u8]) -> EtlResult<()> {
    let value = parse(data)
        .map_err(|e| IngestError::InvalidFormat {
            message: format!("Parse failed: {}", e)
        })?;

    transform(value)
        .map_err(|e| TransformError::RuntimeClass {
            message: e.to_string()
        })?;

    Ok(())
}
```

### 2. Convert from std::io::Error

**Before:**
```rust
fn read_file(path: &str) -> Vec<u8> {
    std::fs::read(path).unwrap()  // ❌
}
```

**After:**
```rust
use knhk_config::error::{ConfigError, ConfigResult};

fn read_file(path: &str) -> ConfigResult<Vec<u8>> {
    std::fs::read(path)
        .map_err(|e| ConfigError::IoError {
            message: format!("Failed to read {}: {}", path, e)
        })
}

// Or with automatic conversion:
impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::IoError {
            message: err.to_string()
        }
    }
}

// Then just use ?:
fn read_file(path: &str) -> ConfigResult<Vec<u8>> {
    Ok(std::fs::read(path)?)  // ✅ Automatic conversion
}
```

### 3. Nested Error Conversion

```rust
use knhk_core::error::{KnhkError, OtelContext};
use knhk_etl::error::{EtlError, IngestError};

// Error automatically converts through hierarchy:
// IngestError -> EtlError -> KnhkError

fn top_level() -> Result<(), KnhkError> {
    process_data()?;  // IngestError auto-converts to KnhkError
    Ok(())
}

fn process_data() -> Result<(), EtlError> {
    ingest_data()?;  // IngestError auto-converts to EtlError
    Ok(())
}

fn ingest_data() -> Result<(), IngestError> {
    Err(IngestError::InvalidFormat {
        message: "Bad data".to_string()
    })
}
```

### 4. Adding OTEL Context

```rust
use knhk_core::error::{KnhkError, OtelContext};

fn process_with_context(
    data: &[u8],
    trace_id: [u8; 16],
    span_id: [u8; 8]
) -> Result<(), KnhkError> {
    // Process data...
    ingest_data(data).map_err(|e| {
        let mut error: KnhkError = e.into();
        *error.context_mut() = OtelContext::new(trace_id, span_id)
            .with_location(format!("{}:{}", file!(), line!()));
        error
    })?;

    Ok(())
}
```

### 5. Using Error Macros (Future)

```rust
use knhk_core::{knhk_error, knhk_bail, knhk_context};

fn process(data: &[u8], trace_id: [u8; 16], span_id: [u8; 8]) -> Result<(), KnhkError> {
    if data.is_empty() {
        knhk_bail!(IngestError::InvalidFormat {
            message: "Empty data".to_string()
        });
    }

    // Auto-add OTEL context to result
    knhk_context!(ingest_data(data), trace_id, span_id)?;

    Ok(())
}
```

## FFI Integration

### Rust Side

```rust
use knhk_core::ffi::error::{KnhkErrorFFI, KnhkErrorCode};
use std::ptr;

#[no_mangle]
pub extern "C" fn knhk_ingest_data(
    data: *const u8,
    len: usize,
    out_error: *mut *mut KnhkErrorFFI
) -> i32 {
    // Safety check
    if data.is_null() || out_error.is_null() {
        return KnhkErrorCode::InvalidArgument as i32;
    }

    // Convert to Rust slice
    let data_slice = unsafe { std::slice::from_raw_parts(data, len) };

    // Call Rust function
    match ingest_data(data_slice) {
        Ok(_) => KnhkErrorCode::Success as i32,
        Err(e) => {
            // Convert to FFI error
            let ffi_error = Box::new(KnhkErrorFFI::from(&e));
            unsafe {
                *out_error = Box::into_raw(ffi_error);
            }
            e.to_ffi().code
        }
    }
}
```

### C Side

```c
#include "knhk_error.h"
#include <stdio.h>
#include <stdlib.h>

int process_data(const uint8_t* data, size_t len) {
    knhk_error_t* error = NULL;

    int32_t result = knhk_ingest_data(data, len, &error);

    if (result != KNHK_SUCCESS) {
        if (error != NULL) {
            fprintf(stderr, "Error %d: %s\n",
                    knhk_error_code(error),
                    knhk_error_message(error));

            // Print OTEL trace for correlation
            fprintf(stderr, "Trace ID: ");
            for (int i = 0; i < 16; i++) {
                fprintf(stderr, "%02x", error->trace_id[i]);
            }
            fprintf(stderr, "\n");

            // Optional: print source location
            const char* location = error->source_location;
            if (location != NULL) {
                fprintf(stderr, "Location: %s\n", location);
            }

            knhk_error_free(error);
        }
        return -1;
    }

    return 0;
}
```

## OTEL Integration

### Recording Errors to Spans

```rust
use knhk_otel::{Tracer, SpanContext, SpanStatus, SpanEvent};
use knhk_core::error::KnhkError;
use std::collections::BTreeMap;

fn process_with_otel(
    data: &[u8],
    tracer: &mut Tracer,
    span_ctx: SpanContext
) -> Result<(), KnhkError> {
    match ingest_data(data) {
        Ok(result) => Ok(result),
        Err(e) => {
            // Add error attributes to span
            tracer.add_attribute(
                span_ctx.clone(),
                "error.code".to_string(),
                e.to_ffi().code.to_string()
            );
            tracer.add_attribute(
                span_ctx.clone(),
                "error.message".to_string(),
                e.to_string()
            );

            // Add error event
            let mut attrs = BTreeMap::new();
            attrs.insert("error.type".to_string(), format!("{:?}", e));

            if let Some(location) = e.context().source_location {
                attrs.insert("error.location".to_string(), location);
            }

            tracer.add_event(span_ctx.clone(), SpanEvent {
                name: "error".to_string(),
                timestamp_ms: knhk_otel::get_timestamp_ms(),
                attributes: attrs,
            });

            // End span with error status
            tracer.end_span(span_ctx, SpanStatus::Error);

            Err(e)
        }
    }
}
```

### Auto-propagate Trace Context

```rust
use knhk_core::error::{KnhkError, OtelContext};

fn process_pipeline(data: &[u8]) -> Result<(), KnhkError> {
    // Get current trace context from OTEL
    let trace_id = knhk_otel::current_trace_id();
    let span_id = knhk_otel::current_span_id();

    ingest_data(data).map_err(|e| {
        let mut error: KnhkError = e.into();
        *error.context_mut() = OtelContext::new(trace_id, span_id);
        error
    })?;

    Ok(())
}
```

## Migration Examples

### Example 1: unwrap() in Parser

**Before:**
```rust
fn parse_rdf(data: &[u8]) -> Graph {
    let parser = Parser::new();
    parser.parse(data).unwrap()  // ❌
}
```

**After:**
```rust
use knhk_etl::error::{EtlResult, IngestError};

fn parse_rdf(data: &[u8]) -> EtlResult<Graph> {
    let parser = Parser::new();
    parser.parse(data)
        .map_err(|e| IngestError::RdfParse {
            message: e.to_string()
        }.into())
}

// With automatic conversion from oxigraph:
impl From<oxigraph::io::RdfParseError> for IngestError {
    fn from(err: oxigraph::io::RdfParseError) -> Self {
        IngestError::RdfParse {
            message: err.to_string()
        }
    }
}

// Simplified:
fn parse_rdf(data: &[u8]) -> EtlResult<Graph> {
    let parser = Parser::new();
    Ok(parser.parse(data)?)  // ✅ Auto-converts
}
```

### Example 2: expect() with Context

**Before:**
```rust
fn get_config(key: &str) -> String {
    config_map.get(key)
        .expect("Config key not found")  // ❌
        .clone()
}
```

**After:**
```rust
use knhk_config::error::{ConfigError, ConfigResult};

fn get_config(key: &str) -> ConfigResult<String> {
    config_map.get(key)
        .ok_or_else(|| ConfigError::MissingField {
            field: key.to_string()
        })
        .map(|s| s.clone())
}
```

### Example 3: Lock Poisoning

**Before:**
```rust
fn update_state(state: &Arc<Mutex<State>>) {
    let mut s = state.lock().unwrap();  // ❌ Panic on poison
    s.value += 1;
}
```

**After:**
```rust
use knhk_core::error::{KnhkError, Result};

fn update_state(state: &Arc<Mutex<State>>) -> Result<()> {
    let mut s = state.lock()
        .map_err(|e| KnhkError::Internal {
            message: format!("Lock poisoned: {}", e),
            context: OtelContext::empty(),
        })?;

    s.value += 1;
    Ok(())
}

// Or use automatic conversion:
impl<T> From<std::sync::PoisonError<T>> for KnhkError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        KnhkError::Internal {
            message: format!("Lock poisoned: {}", err),
            context: OtelContext::empty(),
        }
    }
}

// Simplified:
fn update_state(state: &Arc<Mutex<State>>) -> Result<()> {
    let mut s = state.lock()?;  // ✅ Auto-converts
    s.value += 1;
    Ok(())
}
```

### Example 4: Hot Path (Performance Critical)

**Before:**
```rust
fn hot_operation(input: u64) -> u64 {
    assert!(input < MAX_VALUE);  // ❌ Panic in hot path
    input * 2
}
```

**After:**
```rust
use knhk_hot::error::{hot_error, HOT_ERROR_GUARD};

fn hot_operation(input: u64) -> Result<u64, KnhkError> {
    if input >= MAX_VALUE {
        return Err(hot_error(HOT_ERROR_GUARD));  // ✅ No allocations, ≤8 ticks
    }
    Ok(input * 2)
}

// Add OTEL context after hot path:
fn wrapped_hot_operation(
    input: u64,
    trace_id: [u8; 16],
    span_id: [u8; 8]
) -> Result<u64, KnhkError> {
    hot_operation(input).map_err(|e| {
        add_otel_context(e, trace_id, span_id)
    })
}
```

### Example 5: Serialization for Lockchain

```rust
use knhk_lockchain::error::{LockchainError, LockchainResult};
use serde_json;

fn store_error_in_receipt(error: &KnhkError) -> LockchainResult<Vec<u8>> {
    // Serialize error for lockchain receipt
    serde_json::to_vec(error)
        .map_err(|e| LockchainError::Receipt {
            message: format!("Failed to serialize error: {}", e)
        })
}

fn load_error_from_receipt(data: &[u8]) -> LockchainResult<KnhkError> {
    // Deserialize error from lockchain receipt
    serde_json::from_slice(data)
        .map_err(|e| LockchainError::Receipt {
            message: format!("Failed to deserialize error: {}", e)
        })
}
```

## Best Practices

### 1. Always Provide Context

```rust
// ❌ Bad: Generic error message
return Err(IngestError::InvalidFormat {
    message: "Invalid".to_string()
});

// ✅ Good: Specific context
return Err(IngestError::InvalidFormat {
    message: format!("Invalid JSON at line {}: {}", line_num, reason)
});
```

### 2. Use Specific Error Variants

```rust
// ❌ Bad: Generic error
return Err(PipelineError::StageFailed {
    stage: "transform".to_string(),
    reason: "failed".to_string()
});

// ✅ Good: Specific error type
return Err(TransformError::GuardViolation {
    guard_name: "data_size_limit".to_string(),
    reason: format!("Size {} exceeds limit {}", size, limit)
}.into());
```

### 3. Preserve Original Error Information

```rust
// ❌ Bad: Lose original error
.map_err(|_| MyError::Failed { message: "Failed".to_string() })

// ✅ Good: Preserve original error
.map_err(|e| MyError::Failed {
    message: format!("Operation failed: {}", e)
})
```

### 4. Use Result Type Aliases

```rust
// ✅ Good: Consistent result types
pub type EtlResult<T> = std::result::Result<T, EtlError>;
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;

fn my_function() -> EtlResult<Data> {
    // ...
}
```

### 5. Don't Panic in Libraries

```rust
// ❌ Bad: Library code panics
pub fn process(data: &[u8]) {
    assert!(!data.is_empty());  // Panic!
}

// ✅ Good: Return error
pub fn process(data: &[u8]) -> EtlResult<()> {
    if data.is_empty() {
        return Err(IngestError::InvalidFormat {
            message: "Empty data".to_string()
        }.into());
    }
    Ok(())
}
```

## Performance Considerations

### Hot Path (≤8 ticks)
- Use `hot_error(code)` with const error codes
- Add OTEL context after hot path completes
- No string allocations in critical path

### Warm Path (~50 ticks)
- One String allocation for message is acceptable
- Use structured error variants with fields
- OTEL context can be added immediately

### Cold Path (>50 ticks)
- Full error context, multiple allocations OK
- Rich error messages with debugging info
- Complete OTEL integration

## Testing Error Handling

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let ingest_err = IngestError::InvalidFormat {
            message: "test".to_string()
        };

        let etl_err: EtlError = ingest_err.into();
        let knhk_err: KnhkError = etl_err.into();

        assert!(matches!(knhk_err, KnhkError::Etl { .. }));
    }

    #[test]
    fn test_otel_context() {
        let trace_id = [1u8; 16];
        let span_id = [2u8; 8];

        let error = KnhkError::Internal {
            message: "test".to_string(),
            context: OtelContext::new(trace_id, span_id),
        };

        assert_eq!(error.context().trace_id, trace_id);
        assert_eq!(error.context().span_id, span_id);
    }

    #[test]
    fn test_ffi_conversion() {
        let error = KnhkError::Etl {
            source: EtlError::Ingest {
                source: IngestError::InvalidFormat {
                    message: "test".to_string()
                }
            },
            context: OtelContext::new([1u8; 16], [2u8; 8]),
        };

        let ffi_error = error.to_ffi();
        assert_eq!(ffi_error.code, KnhkErrorCode::EtlIngest as i32);
    }
}
```

## Reference

- **Full Architecture**: `/Users/sac/knhk/docs/ERROR_HIERARCHY_ARCHITECTURE.md`
- **Visual Diagrams**: `/Users/sac/knhk/docs/diagrams/error-hierarchy.md`
- **Error Codes**: See architecture doc for complete range mapping
- **FFI Header**: `c/include/knhk_error.h` (to be created)

## Need Help?

Common issues and solutions:

1. **"Error doesn't implement From"**: Add conversion trait or use `.into()`
2. **"Cannot convert between error types"**: Check hierarchy, may need intermediate conversion
3. **"FFI error memory leak"**: Always call `knhk_error_free()` in C code
4. **"Hot path too slow"**: Use `hot_error()` with const codes, add context later
5. **"OTEL context not propagated"**: Use `error.context_mut()` to add context

---

**Status**: Design Complete - Ready for Implementation
**Last Updated**: 2025-11-06
**Version**: 1.0
