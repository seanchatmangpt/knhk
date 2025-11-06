// knhk-sidecar: Error types with structured diagnostics

use thiserror::Error;
use std::collections::BTreeMap;

/// Structured error context (similar to Weaver's DiagnosticMessage)
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Error code (e.g., "SIDECAR_TRANSACTION_FAILED")
    pub code: String,
    /// Error message
    pub message: String,
    /// Additional context attributes
    pub attributes: BTreeMap<String, String>,
    /// Source location (file:line)
    pub source_location: Option<String>,
    /// Related span ID for OTEL correlation
    pub span_id: Option<String>,
    /// Related trace ID for OTEL correlation
    pub trace_id: Option<String>,
}

impl ErrorContext {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            attributes: BTreeMap::new(),
            source_location: None,
            span_id: None,
            trace_id: None,
        }
    }

    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    pub fn with_source_location(mut self, location: impl Into<String>) -> Self {
        self.source_location = Some(location.into());
        self
    }

    pub fn with_span_id(mut self, span_id: impl Into<String>) -> Self {
        self.span_id = Some(span_id.into());
        self
    }

    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// Convert to JSON for CI/CD integration
    #[cfg(feature = "serde_json")]
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(feature = "serde_json")]
impl serde::Serialize for ErrorContext {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ErrorContext", 6)?;
        state.serialize_field("code", &self.code)?;
        state.serialize_field("message", &self.message)?;
        state.serialize_field("attributes", &self.attributes)?;
        state.serialize_field("source_location", &self.source_location)?;
        state.serialize_field("span_id", &self.span_id)?;
        state.serialize_field("trace_id", &self.trace_id)?;
        state.end()
    }
}

/// Sidecar result type
pub type SidecarResult<T> = Result<T, SidecarError>;

/// Sidecar error types with structured context
#[derive(Debug, Error, Clone)]
#[non_exhaustive]
pub enum SidecarError {
    #[error("Network error: {context}")]
    NetworkError {
        context: ErrorContext,
    },

    #[error("Validation error: {context}")]
    ValidationError {
        context: ErrorContext,
    },

    #[error("Validation failed: {context}")]
    ValidationFailed {
        context: ErrorContext,
    },

    #[error("Transaction failed: {context}")]
    TransactionFailed {
        context: ErrorContext,
    },

    #[error("Query failed: {context}")]
    QueryFailed {
        context: ErrorContext,
    },

    #[error("Hook evaluation failed: {context}")]
    HookEvaluationFailed {
        context: ErrorContext,
    },

    #[error("Request timeout: {context}")]
    TimeoutError {
        context: ErrorContext,
    },

    #[error("Circuit breaker is open: {context}")]
    CircuitBreakerOpen {
        context: ErrorContext,
    },

    #[error("TLS error: {context}")]
    TlsError {
        context: ErrorContext,
    },

    #[error("Batch error: {context}")]
    BatchError {
        context: ErrorContext,
    },

    #[error("Retry exhausted: {context}")]
    RetryExhausted {
        context: ErrorContext,
    },

    #[error("Configuration error: {context}")]
    ConfigError {
        context: ErrorContext,
    },

    #[error("gRPC error: {context}")]
    GrpcError {
        context: ErrorContext,
    },

    #[error("Internal error: {context}")]
    InternalError {
        context: ErrorContext,
    },

    #[error("Pipeline error: {context}")]
    PipelineError {
        context: ErrorContext,
    },
}

impl SidecarError {
    /// Get error context
    pub fn context(&self) -> &ErrorContext {
        match self {
            SidecarError::NetworkError { context }
            | SidecarError::ValidationError { context }
            | SidecarError::ValidationFailed { context }
            | SidecarError::TransactionFailed { context }
            | SidecarError::QueryFailed { context }
            | SidecarError::HookEvaluationFailed { context }
            | SidecarError::TimeoutError { context }
            | SidecarError::CircuitBreakerOpen { context }
            | SidecarError::TlsError { context }
            | SidecarError::BatchError { context }
            | SidecarError::RetryExhausted { context }
            | SidecarError::ConfigError { context }
            | SidecarError::GrpcError { context }
            | SidecarError::InternalError { context }
            | SidecarError::PipelineError { context } => context,
        }
    }

    /// Get error code
    pub fn code(&self) -> &str {
        &self.context().code
    }

    /// Convert to JSON for CI/CD integration
    #[cfg(feature = "serde_json")]
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        use serde_json::json;
        let json_value = json!({
            "error_type": format!("{:?}", self),
            "code": self.code(),
            "message": self.to_string(),
            "context": self.context(),
        });
        serde_json::to_string_pretty(&json_value)
    }

    /// Record error to OTEL span
    #[cfg(feature = "otel")]
    pub fn record_to_span(&self, tracer: &mut knhk_otel::Tracer, span_ctx: knhk_otel::SpanContext) {
        use knhk_otel::{SpanEvent, SpanStatus};
        
        // Add error attributes to span
        tracer.add_attribute(span_ctx.clone(), "error.code".to_string(), self.code().to_string());
        tracer.add_attribute(span_ctx.clone(), "error.message".to_string(), self.to_string());
        
        // Add context attributes
        for (key, value) in &self.context().attributes {
            tracer.add_attribute(span_ctx.clone(), format!("error.{}", key).to_string(), value.clone());
        }
        
        // Add error event
        let error_event = SpanEvent {
            name: "error".to_string(),
            timestamp_ms: knhk_otel::get_timestamp_ms(),
            attributes: {
                let mut attrs = BTreeMap::new();
                attrs.insert("error.code".to_string(), self.code().to_string());
                attrs.insert("error.type".to_string(), format!("{:?}", self));
                attrs.insert("error.message".to_string(), self.to_string());
                // Add context attributes
                for (key, value) in &self.context().attributes {
                    attrs.insert(format!("error.{}", key).to_string(), value.clone());
                }
                attrs
            },
        };
        tracer.add_event(span_ctx.clone(), error_event);
        
        // End span with error status
        tracer.end_span(span_ctx, SpanStatus::Error);
    }
}

// Convenience constructors for backward compatibility
impl SidecarError {
    pub fn network_error(msg: impl Into<String>) -> Self {
        SidecarError::NetworkError {
            context: ErrorContext::new("SIDECAR_NETWORK_ERROR", msg),
        }
    }

    pub fn validation_error(msg: impl Into<String>) -> Self {
        SidecarError::ValidationError {
            context: ErrorContext::new("SIDECAR_VALIDATION_ERROR", msg),
        }
    }

    pub fn validation_failed(msg: impl Into<String>) -> Self {
        SidecarError::ValidationFailed {
            context: ErrorContext::new("SIDECAR_VALIDATION_FAILED", msg),
        }
    }

    pub fn transaction_failed(msg: impl Into<String>) -> Self {
        SidecarError::TransactionFailed {
            context: ErrorContext::new("SIDECAR_TRANSACTION_FAILED", msg),
        }
    }

    pub fn query_failed(msg: impl Into<String>) -> Self {
        SidecarError::QueryFailed {
            context: ErrorContext::new("SIDECAR_QUERY_FAILED", msg),
        }
    }

    pub fn hook_evaluation_failed(msg: impl Into<String>) -> Self {
        SidecarError::HookEvaluationFailed {
            context: ErrorContext::new("SIDECAR_HOOK_EVALUATION_FAILED", msg),
        }
    }

    pub fn tls_error(msg: impl Into<String>) -> Self {
        SidecarError::TlsError {
            context: ErrorContext::new("SIDECAR_TLS_ERROR", msg),
        }
    }

    pub fn config_error(msg: impl Into<String>) -> Self {
        SidecarError::ConfigError {
            context: ErrorContext::new("SIDECAR_CONFIG_ERROR", msg),
        }
    }

    pub fn internal_error(msg: impl Into<String>) -> Self {
        SidecarError::InternalError {
            context: ErrorContext::new("SIDECAR_INTERNAL_ERROR", msg),
        }
    }

    pub fn retry_exhausted(msg: impl Into<String>) -> Self {
        SidecarError::RetryExhausted {
            context: ErrorContext::new("SIDECAR_RETRY_EXHAUSTED", msg),
        }
    }

    pub fn circuit_breaker_open(msg: impl Into<String>) -> Self {
        SidecarError::CircuitBreakerOpen {
            context: ErrorContext::new("SIDECAR_CIRCUIT_BREAKER_OPEN", msg),
        }
    }

    pub fn timeout_error(msg: impl Into<String>) -> Self {
        SidecarError::TimeoutError {
            context: ErrorContext::new("SIDECAR_TIMEOUT_ERROR", msg),
        }
    }
}

impl From<tonic::Status> for SidecarError {
    fn from(status: tonic::Status) -> Self {
        SidecarError::GrpcError {
            context: ErrorContext::new("SIDECAR_GRPC_ERROR", status.message())
                .with_attribute("grpc_code", format!("{:?}", status.code())),
        }
    }
}

impl From<tonic::transport::Error> for SidecarError {
    fn from(err: tonic::transport::Error) -> Self {
        SidecarError::NetworkError {
            context: ErrorContext::new("SIDECAR_NETWORK_ERROR", err.to_string()),
        }
    }
}

impl From<knhk_etl::PipelineError> for SidecarError {
    fn from(err: knhk_etl::PipelineError) -> Self {
        SidecarError::PipelineError {
            context: ErrorContext::new("SIDECAR_PIPELINE_ERROR", format!("{:?}", err))
                .with_attribute("pipeline_error_type", format!("{:?}", err)),
        }
    }
}

/// Check if error is retryable (transient)
pub fn is_retryable_error(err: &SidecarError) -> bool {
    matches!(
        err,
        SidecarError::NetworkError { .. }
            | SidecarError::TimeoutError { .. }
            | SidecarError::CircuitBreakerOpen { .. }
            | SidecarError::GrpcError { .. }
    )
}

/// Check if error is a guard violation (non-retryable)
pub fn is_guard_violation(err: &SidecarError) -> bool {
    matches!(
        err,
        SidecarError::ValidationError { .. } | SidecarError::BatchError { .. }
    )
}

