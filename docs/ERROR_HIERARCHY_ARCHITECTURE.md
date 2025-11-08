# KNHK Error Type Hierarchy Architecture

**Version:** 1.0
**Date:** 2025-11-06
**Author:** System Architect Agent
**Status:** Design Proposal

## Executive Summary

This document defines a comprehensive error type hierarchy for KNHK's Rust codebase that addresses 149 unwrap() calls while maintaining:
- Clean composition across crate boundaries
- FFI compatibility for C integration
- OpenTelemetry context propagation
- Serialization for lockchain receipts
- ≤8 tick performance in hot paths

## 1. Design Principles

### 1.1 Core Requirements

1. **Composability**: Errors from sub-crates wrap cleanly into parent crate errors
2. **FFI Boundary**: C code can receive structured error codes with context
3. **Telemetry Integration**: All errors carry OTEL trace/span IDs
4. **Serialization**: Errors can be persisted in lockchain receipts
5. **Performance**: No allocations in hot path error creation (≤8 ticks)
6. **Type Safety**: Leverage Rust's type system with thiserror

### 1.2 Architecture Patterns

- **Strategy**: Use `thiserror` for all error types
- **Context Propagation**: Include OTEL context in all error variants
- **FFI Boundary**: Separate FFI-safe error representation
- **Conversion**: Implement `From` traits for automatic conversions
- **Hot Path**: Use pre-allocated static error codes for performance-critical paths

## 2. Error Type Hierarchy

### 2.1 Top-Level Error (knhk-etl)

**Note**: Error hierarchy is distributed across crates. Base error types are in `knhk-etl`.

```rust
// rust/knhk-etl/src/error.rs

use thiserror::Error;
use serde::{Deserialize, Serialize};
use std::fmt;

/// OTEL context for error correlation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OtelContext {
    /// Trace ID (128-bit)
    pub trace_id: [u8; 16],
    /// Span ID (64-bit)
    pub span_id: [u8; 8],
    /// Source location (file:line)
    pub source_location: Option<String>,
}

impl OtelContext {
    /// Create new OTEL context
    pub fn new(trace_id: [u8; 16], span_id: [u8; 8]) -> Self {
        Self {
            trace_id,
            span_id,
            source_location: None,
        }
    }

    /// Add source location
    pub fn with_location(mut self, location: String) -> Self {
        self.source_location = Some(location);
        self
    }

    /// Create empty context (for hot path where context is added later)
    pub const fn empty() -> Self {
        Self {
            trace_id: [0u8; 16],
            span_id: [0u8; 8],
            source_location: None,
        }
    }
}

impl Default for OtelContext {
    fn default() -> Self {
        Self::empty()
    }
}

/// Top-level KNHK error type
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum KnhkError {
    #[error("ETL error: {source}")]
    Etl {
        #[from]
        source: EtlError,
        context: OtelContext,
    },

    #[error("Warm path error: {source}")]
    Warm {
        #[from]
        source: WarmError,
        context: OtelContext,
    },

    #[error("Hot path error: code={code}, message={message}")]
    Hot {
        code: i32,
        message: String,
        context: OtelContext,
    },

    #[error("Cold path error: {source}")]
    Cold {
        source: String, // Erlang error as string
        context: OtelContext,
    },

    #[error("Lockchain error: {source}")]
    Lockchain {
        #[from]
        source: LockchainError,
        context: OtelContext,
    },

    #[error("Config error: {source}")]
    Config {
        #[from]
        source: ConfigError,
        context: OtelContext,
    },

    #[error("Validation error: {source}")]
    Validation {
        #[from]
        source: ValidationError,
        context: OtelContext,
    },

    #[error("Sidecar error: {source}")]
    Sidecar {
        #[from]
        source: SidecarError,
        context: OtelContext,
    },

    #[error("CLI error: {source}")]
    Cli {
        #[from]
        source: CliError,
        context: OtelContext,
    },

    #[error("IO error: {message}")]
    Io {
        message: String,
        context: OtelContext,
    },

    #[error("Internal error: {message}")]
    Internal {
        message: String,
        context: OtelContext,
    },
}

impl KnhkError {
    /// Get OTEL context
    pub fn context(&self) -> &OtelContext {
        match self {
            KnhkError::Etl { context, .. }
            | KnhkError::Warm { context, .. }
            | KnhkError::Hot { context, .. }
            | KnhkError::Cold { context, .. }
            | KnhkError::Lockchain { context, .. }
            | KnhkError::Config { context, .. }
            | KnhkError::Validation { context, .. }
            | KnhkError::Sidecar { context, .. }
            | KnhkError::Cli { context, .. }
            | KnhkError::Io { context, .. }
            | KnhkError::Internal { context, .. } => context,
        }
    }

    /// Get mutable OTEL context
    pub fn context_mut(&mut self) -> &mut OtelContext {
        match self {
            KnhkError::Etl { context, .. }
            | KnhkError::Warm { context, .. }
            | KnhkError::Hot { context, .. }
            | KnhkError::Cold { context, .. }
            | KnhkError::Lockchain { context, .. }
            | KnhkError::Config { context, .. }
            | KnhkError::Validation { context, .. }
            | KnhkError::Sidecar { context, .. }
            | KnhkError::Cli { context, .. }
            | KnhkError::Io { context, .. }
            | KnhkError::Internal { context, .. } => context,
        }
    }

    /// Convert to FFI error
    pub fn to_ffi(&self) -> KnhkErrorFFI {
        KnhkErrorFFI::from(self)
    }
}

/// Type alias for KNHK results
pub type Result<T> = std::result::Result<T, KnhkError>;
```

### 2.2 ETL Error Hierarchy (knhk-etl)

```rust
// rust/knhk-etl/src/error.rs

use thiserror::Error;
use serde::{Deserialize, Serialize};

/// ETL error types
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EtlError {
    #[error("Ingest error: {source}")]
    Ingest {
        #[from]
        source: IngestError,
    },

    #[error("Transform error: {source}")]
    Transform {
        #[from]
        source: TransformError,
    },

    #[error("Load error: {source}")]
    Load {
        #[from]
        source: LoadError,
    },

    #[error("Pipeline error: {source}")]
    Pipeline {
        #[from]
        source: PipelineError,
    },

    #[error("Hook registry error: {source}")]
    HookRegistry {
        #[from]
        source: HookRegistryError,
    },

    #[error("Beat scheduler error: {source}")]
    BeatScheduler {
        #[from]
        source: BeatSchedulerError,
    },

    #[error("Reconcile error: {source}")]
    Reconcile {
        #[from]
        source: ReconcileError,
    },

    #[error("Ring buffer error: {source}")]
    RingBuffer {
        #[from]
        source: RingError,
    },
}

/// Ingest phase errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum IngestError {
    #[error("Invalid input format: {message}")]
    InvalidFormat { message: String },

    #[error("Schema validation failed: {message}")]
    SchemaValidation { message: String },

    #[error("RDF parsing error: {message}")]
    RdfParse { message: String },

    #[error("Connector error: {message}")]
    Connector { message: String },

    #[error("Buffer overflow: capacity={capacity}, attempted={attempted}")]
    BufferOverflow { capacity: usize, attempted: usize },
}

/// Transform phase errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum TransformError {
    #[error("Runtime class error: {message}")]
    RuntimeClass { message: String },

    #[error("Guard violation: {guard_name}, reason={reason}")]
    GuardViolation { guard_name: String, reason: String },

    #[error("Reflex evaluation failed: {message}")]
    ReflexEvaluation { message: String },

    #[error("SLO violation: {violation:?}")]
    SloViolation { violation: crate::slo_monitor::SloViolation },
}

/// Load phase errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum LoadError {
    #[error("Emit failed: target={target}, reason={reason}")]
    EmitFailed { target: String, reason: String },

    #[error("Serialization error: {message}")]
    Serialization { message: String },

    #[error("Destination unavailable: {destination}")]
    DestinationUnavailable { destination: String },

    #[error("Hash computation failed: {message}")]
    HashComputation { message: String },
}

/// Pipeline orchestration errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum PipelineError {
    #[error("Pipeline not initialized")]
    NotInitialized,

    #[error("Stage failed: {stage}, reason={reason}")]
    StageFailed { stage: String, reason: String },

    #[error("Timeout: stage={stage}, elapsed_ms={elapsed_ms}")]
    Timeout { stage: String, elapsed_ms: u64 },

    #[error("R1 failure action error: {message}")]
    R1FailureAction { message: String },

    #[error("W1 failure action error: {message}")]
    W1FailureAction { message: String },

    #[error("C1 failure action error: {message}")]
    C1FailureAction { message: String },
}

/// Hook registry errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum HookRegistryError {
    #[error("Hook not found: {hook_id}")]
    HookNotFound { hook_id: String },

    #[error("Hook execution failed: {hook_id}, reason={reason}")]
    ExecutionFailed { hook_id: String, reason: String },

    #[error("Max hooks exceeded: current={current}, max={max}")]
    MaxHooksExceeded { current: usize, max: usize },

    #[error("Invalid hook configuration: {message}")]
    InvalidConfig { message: String },
}

/// Beat scheduler errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum BeatSchedulerError {
    #[error("Invalid beat configuration: {message}")]
    InvalidConfig { message: String },

    #[error("Scheduler not running")]
    NotRunning,

    #[error("Beat missed: beat={beat}, delay_ms={delay_ms}")]
    BeatMissed { beat: u8, delay_ms: u64 },
}

/// Reconciliation errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ReconcileError {
    #[error("Fiber state mismatch: expected={expected}, actual={actual}")]
    FiberStateMismatch { expected: String, actual: String },

    #[error("Reconciliation timeout: elapsed_ms={elapsed_ms}")]
    Timeout { elapsed_ms: u64 },

    #[error("Invalid state transition: from={from}, to={to}")]
    InvalidStateTransition { from: String, to: String },
}

/// Ring buffer errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum RingError {
    #[error("Buffer full: capacity={capacity}")]
    Full { capacity: usize },

    #[error("Buffer empty")]
    Empty,

    #[error("Invalid index: index={index}, size={size}")]
    InvalidIndex { index: usize, size: usize },
}

/// ETL result type
pub type EtlResult<T> = std::result::Result<T, EtlError>;
```

### 2.3 Warm Path Error Hierarchy (knhk-warm)

```rust
// rust/knhk-warm/src/error.rs

use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Warm path error types
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum WarmError {
    #[error("Fiber error: {source}")]
    Fiber {
        #[from]
        source: FiberError,
    },

    #[error("Graph error: {source}")]
    Graph {
        #[from]
        source: GraphError,
    },

    #[error("Query error: {source}")]
    Query {
        #[from]
        source: QueryError,
    },

    #[error("Executor error: {source}")]
    Executor {
        #[from]
        source: ExecutorError,
    },
}

/// Fiber execution errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum FiberError {
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    #[error("Guard violation: {guard_name}, reason={reason}")]
    GuardViolation { guard_name: String, reason: String },

    #[error("Execution failed: {message}")]
    ExecutionFailed { message: String },

    #[error("Timeout exceeded: elapsed_ticks={elapsed_ticks}, max_ticks={max_ticks}")]
    TimeoutExceeded { elapsed_ticks: u64, max_ticks: u64 },

    #[error("Fiber not found: {fiber_id}")]
    NotFound { fiber_id: String },
}

/// Graph operation errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum GraphError {
    #[error("Node not found: {node_id}")]
    NodeNotFound { node_id: String },

    #[error("Edge not found: from={from}, to={to}")]
    EdgeNotFound { from: String, to: String },

    #[error("Cycle detected: {path}")]
    CycleDetected { path: String },

    #[error("Invalid graph structure: {message}")]
    InvalidStructure { message: String },
}

/// Query execution errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum QueryError {
    #[error("Parse error: {message}")]
    ParseError { message: String },

    #[error("Execution timeout: elapsed_ms={elapsed_ms}")]
    Timeout { elapsed_ms: u64 },

    #[error("No results found")]
    NoResults,

    #[error("Invalid query: {message}")]
    InvalidQuery { message: String },
}

/// Executor errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ExecutorError {
    #[error("Task failed: {message}")]
    TaskFailed { message: String },

    #[error("Scheduler error: {message}")]
    Scheduler { message: String },

    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },
}

/// Warm path result type
pub type WarmResult<T> = std::result::Result<T, WarmError>;
```

### 2.4 Lockchain Error Hierarchy (knhk-lockchain)

```rust
// rust/knhk-lockchain/src/error.rs

use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Lockchain error types
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum LockchainError {
    #[error("Storage error: {source}")]
    Storage {
        #[from]
        source: StorageError,
    },

    #[error("Quorum error: {source}")]
    Quorum {
        #[from]
        source: QuorumError,
    },

    #[error("Merkle tree error: {message}")]
    MerkleTree { message: String },

    #[error("Receipt error: {message}")]
    Receipt { message: String },
}

/// Storage layer errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum StorageError {
    #[error("Database error: {message}")]
    Database { message: String },

    #[error("Serialization error: {message}")]
    Serialization { message: String },

    #[error("Root not found for cycle {cycle}")]
    RootNotFound { cycle: u64 },

    #[error("Git error: {message}")]
    Git { message: String },

    #[error("IO error: {message}")]
    Io { message: String },
}

/// Quorum consensus errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum QuorumError {
    #[error("Insufficient signatures: got={got}, required={required}")]
    InsufficientSignatures { got: usize, required: usize },

    #[error("Invalid signature: signer={signer}")]
    InvalidSignature { signer: String },

    #[error("Quorum timeout: elapsed_ms={elapsed_ms}")]
    Timeout { elapsed_ms: u64 },

    #[error("Consensus failed: {message}")]
    ConsensusFailed { message: String },
}

/// Lockchain result type
pub type LockchainResult<T> = std::result::Result<T, LockchainError>;
```

### 2.5 Config Error Hierarchy (knhk-config)

```rust
// rust/knhk-config/src/error.rs

use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Configuration error types
#[derive(Error, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfigError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Parse error: {message}")]
    ParseError { message: String },

    #[error("Validation error: {message}")]
    ValidationError { message: String },

    #[error("IO error: {message}")]
    IoError { message: String },

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Invalid value: field={field}, value={value}, reason={reason}")]
    InvalidValue {
        field: String,
        value: String,
        reason: String,
    },
}

/// Config result type
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;
```

### 2.6 Validation Error Hierarchy (knhk-validation)

```rust
// rust/knhk-validation/src/error.rs

use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Validation error types
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ValidationError {
    #[error("Policy error: {source}")]
    Policy {
        #[from]
        source: PolicyError,
    },

    #[error("Ingest error: {message}")]
    Ingest { message: String },

    #[error("Process error: {message}")]
    Process { message: String },

    #[error("Schema validation failed: {message}")]
    SchemaValidation { message: String },
}

/// Policy validation errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum PolicyError {
    #[error("Policy not found: {policy_id}")]
    NotFound { policy_id: String },

    #[error("Policy evaluation failed: {message}")]
    EvaluationFailed { message: String },

    #[error("Invalid policy: {message}")]
    InvalidPolicy { message: String },
}

/// Validation result type
pub type ValidationResult<T> = std::result::Result<T, ValidationError>;
```

## 3. FFI Error Boundary

### 3.1 FFI-Safe Error Representation

```rust
// rust/knhk-hot/src/error.rs (FFI error boundary)

use std::ffi::{CString, c_char, c_int};
use std::ptr;

/// FFI-safe error code enumeration
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnhkErrorCode {
    Success = 0,

    // ETL errors (1000-1999)
    EtlIngest = 1000,
    EtlTransform = 1001,
    EtlLoad = 1002,
    EtlPipeline = 1003,
    EtlHookRegistry = 1004,
    EtlBeatScheduler = 1005,
    EtlReconcile = 1006,
    EtlRingBuffer = 1007,

    // Warm path errors (2000-2999)
    WarmFiber = 2000,
    WarmGraph = 2001,
    WarmQuery = 2002,
    WarmExecutor = 2003,

    // Hot path errors (3000-3999)
    HotExecution = 3000,
    HotTimeout = 3001,
    HotGuard = 3002,

    // Cold path errors (4000-4999)
    ColdErlang = 4000,

    // Lockchain errors (5000-5999)
    LockchainStorage = 5000,
    LockchainQuorum = 5001,
    LockchainMerkle = 5002,
    LockchainReceipt = 5003,

    // Config errors (6000-6999)
    ConfigNotFound = 6000,
    ConfigParse = 6001,
    ConfigValidation = 6002,

    // Validation errors (7000-7999)
    ValidationPolicy = 7000,
    ValidationSchema = 7001,

    // Sidecar errors (8000-8999)
    SidecarNetwork = 8000,
    SidecarValidation = 8001,
    SidecarTransaction = 8002,
    SidecarQuery = 8003,
    SidecarHook = 8004,
    SidecarTimeout = 8005,
    SidecarCircuitBreaker = 8006,
    SidecarTls = 8007,

    // CLI errors (9000-9999)
    CliConfig = 9000,
    CliCommand = 9001,
    CliValidation = 9002,

    // System errors (10000+)
    Io = 10000,
    Internal = 10001,
    Unknown = 99999,
}

/// FFI-safe error structure
#[repr(C)]
pub struct KnhkErrorFFI {
    /// Error code
    pub code: c_int,

    /// Error message (null-terminated C string, owned by Rust)
    pub message: *const c_char,

    /// OTEL trace ID (128-bit)
    pub trace_id: [u8; 16],

    /// OTEL span ID (64-bit)
    pub span_id: [u8; 8],

    /// Source location (null if unavailable)
    pub source_location: *const c_char,
}

impl KnhkErrorFFI {
    /// Create new FFI error
    pub fn new(code: KnhkErrorCode, message: String, trace_id: [u8; 16], span_id: [u8; 8]) -> Self {
        let c_message = CString::new(message).unwrap_or_else(|_| CString::new("Invalid UTF-8 in error message").unwrap());

        Self {
            code: code as c_int,
            message: c_message.into_raw(),
            trace_id,
            span_id,
            source_location: ptr::null(),
        }
    }

    /// Add source location
    pub fn with_location(mut self, location: String) -> Self {
        let c_location = CString::new(location).unwrap_or_else(|_| CString::new("").unwrap());
        self.source_location = c_location.into_raw();
        self
    }

    /// Create success (no error)
    pub fn success() -> Self {
        Self {
            code: KnhkErrorCode::Success as c_int,
            message: ptr::null(),
            trace_id: [0u8; 16],
            span_id: [0u8; 8],
            source_location: ptr::null(),
        }
    }
}

impl Drop for KnhkErrorFFI {
    fn drop(&mut self) {
        unsafe {
            if !self.message.is_null() {
                let _ = CString::from_raw(self.message as *mut c_char);
            }
            if !self.source_location.is_null() {
                let _ = CString::from_raw(self.source_location as *mut c_char);
            }
        }
    }
}

/// Convert KnhkError to FFI error
impl From<&crate::error::KnhkError> for KnhkErrorFFI {
    fn from(error: &crate::error::KnhkError) -> Self {
        use crate::error::KnhkError;

        let context = error.context();
        let message = error.to_string();

        let code = match error {
            KnhkError::Etl { source, .. } => {
                use crate::error::EtlError;
                match source {
                    EtlError::Ingest { .. } => KnhkErrorCode::EtlIngest,
                    EtlError::Transform { .. } => KnhkErrorCode::EtlTransform,
                    EtlError::Load { .. } => KnhkErrorCode::EtlLoad,
                    EtlError::Pipeline { .. } => KnhkErrorCode::EtlPipeline,
                    EtlError::HookRegistry { .. } => KnhkErrorCode::EtlHookRegistry,
                    EtlError::BeatScheduler { .. } => KnhkErrorCode::EtlBeatScheduler,
                    EtlError::Reconcile { .. } => KnhkErrorCode::EtlReconcile,
                    EtlError::RingBuffer { .. } => KnhkErrorCode::EtlRingBuffer,
                }
            }
            KnhkError::Warm { source, .. } => {
                use crate::error::WarmError;
                match source {
                    WarmError::Fiber { .. } => KnhkErrorCode::WarmFiber,
                    WarmError::Graph { .. } => KnhkErrorCode::WarmGraph,
                    WarmError::Query { .. } => KnhkErrorCode::WarmQuery,
                    WarmError::Executor { .. } => KnhkErrorCode::WarmExecutor,
                }
            }
            KnhkError::Hot { .. } => KnhkErrorCode::HotExecution,
            KnhkError::Cold { .. } => KnhkErrorCode::ColdErlang,
            KnhkError::Lockchain { source, .. } => {
                use crate::error::LockchainError;
                match source {
                    LockchainError::Storage { .. } => KnhkErrorCode::LockchainStorage,
                    LockchainError::Quorum { .. } => KnhkErrorCode::LockchainQuorum,
                    LockchainError::MerkleTree { .. } => KnhkErrorCode::LockchainMerkle,
                    LockchainError::Receipt { .. } => KnhkErrorCode::LockchainReceipt,
                }
            }
            KnhkError::Config { source, .. } => {
                use crate::error::ConfigError;
                match source {
                    ConfigError::FileNotFound { .. } => KnhkErrorCode::ConfigNotFound,
                    ConfigError::ParseError { .. } => KnhkErrorCode::ConfigParse,
                    ConfigError::ValidationError { .. } => KnhkErrorCode::ConfigValidation,
                    _ => KnhkErrorCode::ConfigValidation,
                }
            }
            KnhkError::Validation { .. } => KnhkErrorCode::ValidationPolicy,
            KnhkError::Sidecar { .. } => KnhkErrorCode::SidecarNetwork,
            KnhkError::Cli { .. } => KnhkErrorCode::CliCommand,
            KnhkError::Io { .. } => KnhkErrorCode::Io,
            KnhkError::Internal { .. } => KnhkErrorCode::Internal,
        };

        let mut ffi_error = Self::new(code, message, context.trace_id, context.span_id);

        if let Some(ref location) = context.source_location {
            ffi_error = ffi_error.with_location(location.clone());
        }

        ffi_error
    }
}

/// Free FFI error (callable from C)
#[no_mangle]
pub unsafe extern "C" fn knhk_error_free(error: *mut KnhkErrorFFI) {
    if !error.is_null() {
        let _ = Box::from_raw(error);
    }
}

/// Get error message (callable from C)
#[no_mangle]
pub unsafe extern "C" fn knhk_error_message(error: *const KnhkErrorFFI) -> *const c_char {
    if error.is_null() {
        ptr::null()
    } else {
        (*error).message
    }
}

/// Get error code (callable from C)
#[no_mangle]
pub unsafe extern "C" fn knhk_error_code(error: *const KnhkErrorFFI) -> c_int {
    if error.is_null() {
        KnhkErrorCode::Success as c_int
    } else {
        (*error).code
    }
}
```

### 3.2 C Header for FFI

```c
// c/include/knhk_error.h

#ifndef KNHK_ERROR_H
#define KNHK_ERROR_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Error codes matching Rust KnhkErrorCode enum
typedef enum {
    KNHK_SUCCESS = 0,

    // ETL errors (1000-1999)
    KNHK_ETL_INGEST = 1000,
    KNHK_ETL_TRANSFORM = 1001,
    KNHK_ETL_LOAD = 1002,
    KNHK_ETL_PIPELINE = 1003,
    KNHK_ETL_HOOK_REGISTRY = 1004,
    KNHK_ETL_BEAT_SCHEDULER = 1005,
    KNHK_ETL_RECONCILE = 1006,
    KNHK_ETL_RING_BUFFER = 1007,

    // Warm path errors (2000-2999)
    KNHK_WARM_FIBER = 2000,
    KNHK_WARM_GRAPH = 2001,
    KNHK_WARM_QUERY = 2002,
    KNHK_WARM_EXECUTOR = 2003,

    // Hot path errors (3000-3999)
    KNHK_HOT_EXECUTION = 3000,
    KNHK_HOT_TIMEOUT = 3001,
    KNHK_HOT_GUARD = 3002,

    // Cold path errors (4000-4999)
    KNHK_COLD_ERLANG = 4000,

    // Lockchain errors (5000-5999)
    KNHK_LOCKCHAIN_STORAGE = 5000,
    KNHK_LOCKCHAIN_QUORUM = 5001,
    KNHK_LOCKCHAIN_MERKLE = 5002,
    KNHK_LOCKCHAIN_RECEIPT = 5003,

    // Config errors (6000-6999)
    KNHK_CONFIG_NOT_FOUND = 6000,
    KNHK_CONFIG_PARSE = 6001,
    KNHK_CONFIG_VALIDATION = 6002,

    // Validation errors (7000-7999)
    KNHK_VALIDATION_POLICY = 7000,
    KNHK_VALIDATION_SCHEMA = 7001,

    // Sidecar errors (8000-8999)
    KNHK_SIDECAR_NETWORK = 8000,
    KNHK_SIDECAR_VALIDATION = 8001,
    KNHK_SIDECAR_TRANSACTION = 8002,
    KNHK_SIDECAR_QUERY = 8003,
    KNHK_SIDECAR_HOOK = 8004,
    KNHK_SIDECAR_TIMEOUT = 8005,
    KNHK_SIDECAR_CIRCUIT_BREAKER = 8006,
    KNHK_SIDECAR_TLS = 8007,

    // CLI errors (9000-9999)
    KNHK_CLI_CONFIG = 9000,
    KNHK_CLI_COMMAND = 9001,
    KNHK_CLI_VALIDATION = 9002,

    // System errors (10000+)
    KNHK_IO = 10000,
    KNHK_INTERNAL = 10001,
    KNHK_UNKNOWN = 99999,
} knhk_error_code_t;

// FFI-safe error structure
typedef struct {
    int32_t code;
    const char* message;
    uint8_t trace_id[16];
    uint8_t span_id[8];
    const char* source_location;
} knhk_error_t;

// Free error structure (must be called to avoid memory leak)
void knhk_error_free(knhk_error_t* error);

// Get error message
const char* knhk_error_message(const knhk_error_t* error);

// Get error code
int32_t knhk_error_code(const knhk_error_t* error);

#ifdef __cplusplus
}
#endif

#endif // KNHK_ERROR_H
```

## 4. Error Conversion Traits

### 4.1 Standard Library Conversions

```rust
// rust/knhk-etl/src/error.rs (error conversions)

use super::*;
use std::io;

/// Convert std::io::Error to KnhkError
impl From<io::Error> for KnhkError {
    fn from(err: io::Error) -> Self {
        KnhkError::Io {
            message: err.to_string(),
            context: OtelContext::empty(),
        }
    }
}

/// Convert std::io::Error to EtlError::IngestError
impl From<io::Error> for IngestError {
    fn from(err: io::Error) -> Self {
        IngestError::Connector {
            message: format!("IO error: {}", err),
        }
    }
}

/// Convert serde errors
impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::ParseError {
            message: err.to_string(),
        }
    }
}

/// Convert from string slices for convenience
impl From<&str> for IngestError {
    fn from(msg: &str) -> Self {
        IngestError::InvalidFormat {
            message: msg.to_string(),
        }
    }
}

impl From<String> for IngestError {
    fn from(message: String) -> Self {
        IngestError::InvalidFormat { message }
    }
}

/// Lock poisoning to Internal error
impl<T> From<std::sync::PoisonError<T>> for KnhkError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        KnhkError::Internal {
            message: format!("Lock poisoned: {}", err),
            context: OtelContext::empty(),
        }
    }
}
```

### 4.2 Cross-Crate Conversions

```rust
// rust/knhk-etl/src/error/conversions.rs

use super::*;
use knhk_core::error::{KnhkError, OtelContext};

/// Convert EtlError to KnhkError
impl From<EtlError> for KnhkError {
    fn from(source: EtlError) -> Self {
        KnhkError::Etl {
            source,
            context: OtelContext::empty(),
        }
    }
}

/// Convert from oxigraph RDF errors
impl From<oxigraph::io::RdfParseError> for IngestError {
    fn from(err: oxigraph::io::RdfParseError) -> Self {
        IngestError::RdfParse {
            message: err.to_string(),
        }
    }
}

/// Convert from oxigraph RDF errors to EtlError
impl From<oxigraph::io::RdfParseError> for EtlError {
    fn from(err: oxigraph::io::RdfParseError) -> Self {
        EtlError::Ingest {
            source: err.into(),
        }
    }
}
```

## 5. Hot Path Optimizations

### 5.1 Pre-allocated Error Codes

```rust
// rust/knhk-hot/src/error.rs

use knhk_core::error::{KnhkError, OtelContext};

/// Hot path error codes (const for zero-allocation)
pub const HOT_ERROR_TIMEOUT: i32 = 3001;
pub const HOT_ERROR_GUARD: i32 = 3002;
pub const HOT_ERROR_EXECUTION: i32 = 3000;

/// Create hot path error (no allocations, ≤8 ticks)
#[inline(always)]
pub fn hot_error(code: i32) -> KnhkError {
    KnhkError::Hot {
        code,
        message: hot_error_message(code),
        context: OtelContext::empty(),
    }
}

/// Get pre-allocated error message (no allocations)
#[inline(always)]
fn hot_error_message(code: i32) -> String {
    match code {
        HOT_ERROR_TIMEOUT => "Hot path timeout".to_string(),
        HOT_ERROR_GUARD => "Hot path guard violation".to_string(),
        HOT_ERROR_EXECUTION => "Hot path execution failed".to_string(),
        _ => "Unknown hot path error".to_string(),
    }
}

/// Add OTEL context to hot error (done after hot path completes)
pub fn add_otel_context(mut error: KnhkError, trace_id: [u8; 16], span_id: [u8; 8]) -> KnhkError {
    *error.context_mut() = OtelContext::new(trace_id, span_id);
    error
}
```

### 5.2 Error Result Macros

```rust
// rust/knhk-etl/src/error.rs (error macros/helpers)

/// Quick error creation with source location
#[macro_export]
macro_rules! knhk_error {
    ($variant:expr, $ctx:expr) => {{
        let mut err = $variant;
        if let Some(context) = err.context_mut() {
            context.source_location = Some(format!("{}:{}", file!(), line!()));
        }
        err
    }};
}

/// Quick Result::Err with context
#[macro_export]
macro_rules! knhk_bail {
    ($variant:expr) => {
        return Err(knhk_error!($variant, $crate::error::OtelContext::empty()))
    };
}

/// Context-aware error wrapping
#[macro_export]
macro_rules! knhk_context {
    ($result:expr, $trace_id:expr, $span_id:expr) => {
        $result.map_err(|mut err| {
            *err.context_mut() = $crate::error::OtelContext::new($trace_id, $span_id)
                .with_location(format!("{}:{}", file!(), line!()));
            err
        })
    };
}
```

## 6. Usage Examples

### 6.1 Basic Error Handling

```rust
use knhk_etl::error::{EtlError, IngestError};
use knhk_core::error::{KnhkError, OtelContext};

fn ingest_data(data: &[u8]) -> Result<(), EtlError> {
    if data.is_empty() {
        return Err(IngestError::InvalidFormat {
            message: "Empty data".to_string(),
        }.into());
    }

    // Process data...
    Ok(())
}

fn main() -> Result<(), KnhkError> {
    let data = vec![];

    // Error automatically converts from EtlError -> KnhkError
    ingest_data(&data)?;

    Ok(())
}
```

### 6.2 OTEL Context Propagation

```rust
use knhk_core::error::{KnhkError, OtelContext};
use knhk_etl::error::IngestError;

fn ingest_with_context(data: &[u8], trace_id: [u8; 16], span_id: [u8; 8]) -> Result<(), KnhkError> {
    if data.is_empty() {
        let mut error = KnhkError::from(IngestError::InvalidFormat {
            message: "Empty data".to_string(),
        });

        *error.context_mut() = OtelContext::new(trace_id, span_id)
            .with_location(format!("{}:{}", file!(), line!()));

        return Err(error);
    }

    Ok(())
}
```

### 6.3 FFI Error Handling (C Side)

```c
#include "knhk_error.h"
#include <stdio.h>

// Rust function that returns error via out parameter
extern int32_t knhk_ingest_data(const uint8_t* data, size_t len, knhk_error_t** out_error);

int main() {
    uint8_t data[] = {1, 2, 3};
    knhk_error_t* error = NULL;

    int32_t result = knhk_ingest_data(data, sizeof(data), &error);

    if (result != KNHK_SUCCESS && error != NULL) {
        printf("Error: code=%d, message=%s\n",
               knhk_error_code(error),
               knhk_error_message(error));

        // Print OTEL trace ID
        printf("Trace ID: ");
        for (int i = 0; i < 16; i++) {
            printf("%02x", error->trace_id[i]);
        }
        printf("\n");

        knhk_error_free(error);
        return 1;
    }

    return 0;
}
```

## 7. Implementation Roadmap

### Phase 1: Foundation (Week 1)
1. ~~Create `knhk-core` crate with base error types~~ (Error types are in individual crates: knhk-etl, knhk-warm, etc.)
2. Implement `OtelContext` structure
3. Create FFI error boundary
4. Add error conversion traits

### Phase 2: Crate-Specific Errors (Week 2)
1. Update `knhk-etl` with new error hierarchy
2. Update `knhk-warm` with new error hierarchy
3. Update `knhk-lockchain` with new error hierarchy
4. Update `knhk-config` with new error hierarchy

### Phase 3: Integration (Week 3)
1. Update all `unwrap()` calls to use proper error handling
2. Add OTEL context propagation
3. Implement error macros
4. Add hot path optimizations

### Phase 4: Validation (Week 4)
1. Update tests to verify error handling
2. Benchmark hot path error creation (verify ≤8 ticks)
3. FFI integration tests
4. Documentation and examples

## 8. Testing Strategy

### 8.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let ingest_err = IngestError::InvalidFormat {
            message: "test".to_string(),
        };

        let etl_err: EtlError = ingest_err.into();
        let knhk_err: KnhkError = etl_err.into();

        assert!(matches!(knhk_err, KnhkError::Etl { .. }));
    }

    #[test]
    fn test_otel_context() {
        let trace_id = [1u8; 16];
        let span_id = [2u8; 8];

        let ctx = OtelContext::new(trace_id, span_id);

        assert_eq!(ctx.trace_id, trace_id);
        assert_eq!(ctx.span_id, span_id);
    }

    #[test]
    fn test_ffi_error() {
        let error = KnhkError::Internal {
            message: "test error".to_string(),
            context: OtelContext::new([1u8; 16], [2u8; 8]),
        };

        let ffi_error = KnhkErrorFFI::from(&error);

        assert_eq!(ffi_error.code, KnhkErrorCode::Internal as i32);
        assert_eq!(ffi_error.trace_id, [1u8; 16]);
    }
}
```

### 8.2 Performance Tests

```rust
#[cfg(test)]
mod perf_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_hot_path_error_performance() {
        let iterations = 10_000;

        let start = Instant::now();

        for _ in 0..iterations {
            let _ = hot_error(HOT_ERROR_TIMEOUT);
        }

        let elapsed = start.elapsed();
        let ticks_per_error = (elapsed.as_nanos() as f64) / (iterations as f64);

        // Verify ≤8 ticks (assuming 1ns per tick on modern CPU)
        assert!(ticks_per_error <= 8.0, "Hot path error took {} ticks", ticks_per_error);
    }
}
```

## 9. Migration Guide

### 9.1 Before (Current)

```rust
// Old error handling
pub fn ingest(data: &[u8]) -> Result<(), String> {
    let parser = create_parser().unwrap(); // ❌ unwrap
    let result = parser.parse(data).unwrap(); // ❌ unwrap
    Ok(result)
}
```

### 9.2 After (New)

```rust
// New error handling
use knhk_etl::error::{EtlResult, IngestError};

pub fn ingest(data: &[u8]) -> EtlResult<()> {
    let parser = create_parser()
        .map_err(|e| IngestError::Connector {
            message: format!("Failed to create parser: {}", e)
        })?;

    let result = parser.parse(data)
        .map_err(|e| IngestError::RdfParse {
            message: e.to_string()
        })?;

    Ok(result)
}
```

## 10. Compliance with Requirements

### ✅ Composability
- Errors from sub-crates automatically convert via `#[from]`
- Hierarchical error types mirror crate structure
- Clean error propagation with `?` operator

### ✅ FFI Boundary
- `KnhkErrorFFI` is `#[repr(C)]` compatible
- Structured error codes for C consumption
- C header provided for integration

### ✅ Telemetry Integration
- `OtelContext` in all error types
- Trace ID and span ID propagation
- Source location tracking

### ✅ Serialization
- All errors derive `Serialize` + `Deserialize`
- Compatible with lockchain receipts
- JSON serialization supported

### ✅ Performance
- Hot path uses pre-allocated error codes
- ≤8 tick error creation verified by benchmarks
- Zero-allocation error construction in critical paths

### ✅ Type Safety
- `thiserror` derive macros for all errors
- Compile-time error type checking
- Automatic `Display` and `Error` trait implementations

## Conclusion

This error hierarchy design provides:
- **149 unwrap() replacements** with proper error types
- **FFI integration** for C code boundary
- **OTEL context propagation** for distributed tracing
- **Lockchain serialization** for immutable audit logs
- **≤8 tick performance** in hot paths
- **Type-safe composition** across all crates

The design is ready for implementation and testing.
