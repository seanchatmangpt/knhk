# KNHK Error Hierarchy - Visual Architecture

## Error Type Hierarchy

```
KnhkError (Top-Level)
├── EtlError (knhk-etl)
│   ├── IngestError
│   │   ├── InvalidFormat
│   │   ├── SchemaValidation
│   │   ├── RdfParse
│   │   ├── Connector
│   │   └── BufferOverflow
│   ├── TransformError
│   │   ├── RuntimeClass
│   │   ├── GuardViolation
│   │   ├── ReflexEvaluation
│   │   └── SloViolation
│   ├── LoadError
│   │   ├── EmitFailed
│   │   ├── Serialization
│   │   ├── DestinationUnavailable
│   │   └── HashComputation
│   ├── PipelineError
│   │   ├── NotInitialized
│   │   ├── StageFailed
│   │   ├── Timeout
│   │   ├── R1FailureAction
│   │   ├── W1FailureAction
│   │   └── C1FailureAction
│   ├── HookRegistryError
│   │   ├── HookNotFound
│   │   ├── ExecutionFailed
│   │   ├── MaxHooksExceeded
│   │   └── InvalidConfig
│   ├── BeatSchedulerError
│   │   ├── InvalidConfig
│   │   ├── NotRunning
│   │   └── BeatMissed
│   ├── ReconcileError
│   │   ├── FiberStateMismatch
│   │   ├── Timeout
│   │   └── InvalidStateTransition
│   └── RingError
│       ├── Full
│       ├── Empty
│       └── InvalidIndex
├── WarmError (knhk-warm)
│   ├── FiberError
│   │   ├── InvalidInput
│   │   ├── GuardViolation
│   │   ├── ExecutionFailed
│   │   ├── TimeoutExceeded
│   │   └── NotFound
│   ├── GraphError
│   │   ├── NodeNotFound
│   │   ├── EdgeNotFound
│   │   ├── CycleDetected
│   │   └── InvalidStructure
│   ├── QueryError
│   │   ├── ParseError
│   │   ├── Timeout
│   │   ├── NoResults
│   │   └── InvalidQuery
│   └── ExecutorError
│       ├── TaskFailed
│       ├── Scheduler
│       └── ResourceExhausted
├── HotError (via FFI from C)
│   ├── Execution (3000)
│   ├── Timeout (3001)
│   └── Guard (3002)
├── ColdError (Erlang integration)
│   └── ErlangError (4000)
├── LockchainError (knhk-lockchain)
│   ├── StorageError
│   │   ├── Database
│   │   ├── Serialization
│   │   ├── RootNotFound
│   │   ├── Git
│   │   └── Io
│   ├── QuorumError
│   │   ├── InsufficientSignatures
│   │   ├── InvalidSignature
│   │   ├── Timeout
│   │   └── ConsensusFailed
│   ├── MerkleTreeError
│   └── ReceiptError
├── ConfigError (knhk-config)
│   ├── FileNotFound
│   ├── ParseError
│   ├── ValidationError
│   ├── IoError
│   ├── MissingField
│   └── InvalidValue
├── ValidationError (knhk-validation)
│   ├── PolicyError
│   │   ├── NotFound
│   │   ├── EvaluationFailed
│   │   └── InvalidPolicy
│   ├── IngestError
│   ├── ProcessError
│   └── SchemaValidation
├── SidecarError (knhk-sidecar)
│   ├── NetworkError
│   ├── ValidationError
│   ├── ValidationFailed
│   ├── TransactionFailed
│   ├── QueryFailed
│   ├── HookEvaluationFailed
│   ├── TimeoutError
│   ├── CircuitBreakerOpen
│   ├── TlsError
│   ├── BatchError
│   ├── RetryExhausted
│   ├── ConfigError
│   ├── GrpcError
│   ├── InternalError
│   └── PipelineError
├── CliError (knhk-cli)
│   ├── Config
│   ├── Io
│   ├── Command
│   ├── Validation
│   ├── InvalidArgument
│   └── NotFound
├── IoError (std::io::Error wrapper)
└── InternalError (catch-all)
```

## Error Code Ranges

```
Range       | Category          | Crate
------------|-------------------|------------------
0           | Success           | -
1000-1999   | ETL Errors        | knhk-etl
2000-2999   | Warm Path Errors  | knhk-warm
3000-3999   | Hot Path Errors   | knhk-hot (C FFI)
4000-4999   | Cold Path Errors  | knhk-cold (Erlang)
5000-5999   | Lockchain Errors  | knhk-lockchain
6000-6999   | Config Errors     | knhk-config
7000-7999   | Validation Errors | knhk-validation
8000-8999   | Sidecar Errors    | knhk-sidecar
9000-9999   | CLI Errors        | knhk-cli
10000+      | System Errors     | std lib
99999       | Unknown           | -
```

## Error Context Flow

```
┌─────────────────────────────────────────────────────────┐
│                    Rust Error                            │
│  ┌─────────────────────────────────────────────────┐   │
│  │ KnhkError                                        │   │
│  │  - Variant (EtlError, WarmError, etc.)         │   │
│  │  - OtelContext:                                 │   │
│  │    - trace_id: [u8; 16]                        │   │
│  │    - span_id: [u8; 8]                          │   │
│  │    - source_location: Option<String>           │   │
│  └─────────────────────────────────────────────────┘   │
│                        ↓                                 │
│                   Conversion                             │
│                        ↓                                 │
│  ┌─────────────────────────────────────────────────┐   │
│  │ KnhkErrorFFI (#[repr(C)])                       │   │
│  │  - code: i32                                    │   │
│  │  - message: *const c_char                      │   │
│  │  - trace_id: [u8; 16]                          │   │
│  │  - span_id: [u8; 8]                            │   │
│  │  - source_location: *const c_char             │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
                         ↓
                    FFI Boundary
                         ↓
┌─────────────────────────────────────────────────────────┐
│                     C Code                               │
│  knhk_error_t* error;                                    │
│  int32_t code = knhk_error_code(error);                 │
│  const char* msg = knhk_error_message(error);           │
│  // Use error...                                         │
│  knhk_error_free(error);                                 │
└─────────────────────────────────────────────────────────┘
```

## Error Conversion Chain Example

```
oxigraph::RdfParseError
        ↓ (From trait)
    IngestError::RdfParse
        ↓ (From trait)
    EtlError::Ingest
        ↓ (From trait)
    KnhkError::Etl { source, context }
        ↓ (to_ffi())
    KnhkErrorFFI { code: 1000, message, trace_id, span_id }
```

## Hot Path Optimization

```
Hot Path (≤8 ticks):
  const HOT_ERROR_TIMEOUT: i32 = 3001
        ↓
  hot_error(HOT_ERROR_TIMEOUT)  // No allocations
        ↓
  KnhkError::Hot {
    code: 3001,
    message: "Hot path timeout",  // Pre-allocated
    context: OtelContext::empty()  // Zero-init
  }
        ↓ (After hot path)
  add_otel_context(error, trace_id, span_id)
```

## Serialization for Lockchain

```
┌────────────────────────────────────────┐
│ KnhkError (with all context)           │
│  - Derives Serialize + Deserialize     │
└────────────────────────────────────────┘
              ↓
      serde_json::to_vec()
              ↓
┌────────────────────────────────────────┐
│ JSON bytes                              │
└────────────────────────────────────────┘
              ↓
    Lockchain Receipt Storage
              ↓
┌────────────────────────────────────────┐
│ Immutable Audit Log                     │
│  - Git commit                           │
│  - Merkle tree                          │
│  - Quorum proof                         │
└────────────────────────────────────────┘
```

## OTEL Integration

```
Error Creation:
  KnhkError::Etl {
    source: IngestError::InvalidFormat { ... },
    context: OtelContext {
      trace_id: [active trace ID],
      span_id: [active span ID],
      source_location: Some("file.rs:123")
    }
  }
        ↓
Error Propagation:
  - Trace ID follows error through stack
  - Span ID associates error with specific operation
  - Source location pinpoints error origin
        ↓
OTEL Span Recording:
  span.record_error(error.to_string())
  span.set_attribute("error.code", error.to_ffi().code)
  span.set_attribute("error.trace_id", hex(error.context().trace_id))
  span.end(Status::Error)
```

## Usage Pattern Comparison

### Before (unwrap)
```rust
fn process(data: &[u8]) -> () {
    let parser = create_parser().unwrap();  // ❌ Panic on error
    parser.parse(data).unwrap();             // ❌ No context
}
```

### After (typed errors)
```rust
fn process(data: &[u8]) -> EtlResult<()> {
    let parser = create_parser()
        .map_err(|e| IngestError::Connector {
            message: format!("Parser init: {}", e)
        })?;

    parser.parse(data)
        .map_err(|e| IngestError::RdfParse {
            message: e.to_string()
        })?;

    Ok(())
}
```

### FFI Usage (C)
```c
knhk_error_t* error = NULL;
int32_t result = knhk_process_data(data, len, &error);

if (result != KNHK_SUCCESS) {
    printf("Error %d: %s\n",
           knhk_error_code(error),
           knhk_error_message(error));

    // Access OTEL trace for correlation
    printf("Trace: ");
    for (int i = 0; i < 16; i++) {
        printf("%02x", error->trace_id[i]);
    }
    printf("\n");

    knhk_error_free(error);
}
```

## Performance Characteristics

| Operation                    | Ticks | Allocations |
|------------------------------|-------|-------------|
| Hot path error creation      | ≤8    | 0           |
| Warm path error creation     | ~50   | 1 (String)  |
| Error conversion (From)      | ~20   | 0           |
| FFI conversion               | ~100  | 2 (CString) |
| OTEL context addition        | ~30   | 0           |
| Serialization (JSON)         | ~500  | Multiple    |

## Implementation Status

- [ ] Phase 1: Foundation (knhk-core) - Week 1
- [ ] Phase 2: Crate-specific errors - Week 2
- [ ] Phase 3: Integration (replace unwraps) - Week 3
- [ ] Phase 4: Validation & testing - Week 4

## Design Decisions

1. **thiserror over manual impl**: Reduces boilerplate, ensures consistency
2. **OtelContext in all errors**: Enables distributed tracing, minimal overhead
3. **Separate FFI layer**: Clean boundary, type safety maintained
4. **Error code ranges**: Easy categorization, debugging, monitoring
5. **Hot path optimization**: Const codes + empty context = ≤8 ticks
6. **Serialization support**: Lockchain receipts, audit logs, debugging
7. **Hierarchical structure**: Mirrors crate architecture, intuitive composition

## References

- Design Document: `/Users/sac/knhk/docs/ERROR_HIERARCHY_ARCHITECTURE.md`
- Unwrap Audit: 149 unwrap() calls to replace
- OTEL Integration: OpenTelemetry context propagation
- FFI Boundary: C header at `c/include/knhk_error.h`
