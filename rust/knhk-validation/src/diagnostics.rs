// rust/knhk-validation/src/diagnostics.rs
// Structured Diagnostics System (inspired by Weaver)
// Provides rich error context, OTEL integration, and JSON output for CI/CD

#![cfg(feature = "diagnostics")]

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

#[cfg(feature = "diagnostics")]
use miette::{Diagnostic, SourceSpan};

/// Diagnostic message with rich context
#[derive(Debug, Clone)]
pub struct DiagnosticMessage {
    /// Error code (e.g., "E001", "GUARD_VIOLATION")
    pub code: String,
    /// Human-readable message
    pub message: String,
    /// Severity level
    pub severity: Severity,
    /// Additional context (key-value pairs)
    pub context: BTreeMap<String, String>,
    /// OTEL span ID for tracing
    pub span_id: Option<String>,
    /// Source location (file:line:column)
    pub source_location: Option<SourceLocation>,
    /// Related diagnostics
    pub related: Vec<DiagnosticMessage>,
}

/// Severity level for diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Information - useful context without action needed
    Info,
    /// Warning - something that should be addressed but doesn't break functionality
    Warning,
    /// Error - something that breaks functionality
    Error,
    /// Critical - system-level failure
    Critical,
}

/// Source location information
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

impl DiagnosticMessage {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            severity: Severity::Error,
            context: BTreeMap::new(),
            span_id: None,
            source_location: None,
            related: Vec::new(),
        }
    }
    
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }
    
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
    
    pub fn with_span_id(mut self, span_id: impl Into<String>) -> Self {
        self.span_id = Some(span_id.into());
        self
    }
    
    pub fn with_source_location(mut self, file: impl Into<String>, line: u32, column: u32) -> Self {
        self.source_location = Some(SourceLocation {
            file: file.into(),
            line,
            column,
        });
        self
    }
    
    pub fn with_related(mut self, related: DiagnosticMessage) -> Self {
        self.related.push(related);
        self
    }
}

/// Collection of diagnostic messages
#[derive(Debug, Clone)]
pub struct Diagnostics {
    messages: Vec<DiagnosticMessage>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }
    
    pub fn add(&mut self, message: DiagnosticMessage) {
        self.messages.push(message);
    }
    
    pub fn messages(&self) -> &[DiagnosticMessage] {
        &self.messages
    }
    
    pub fn has_errors(&self) -> bool {
        self.messages.iter().any(|m| {
            matches!(m.severity, Severity::Error | Severity::Critical)
        })
    }
    
    pub fn has_warnings(&self) -> bool {
        self.messages.iter().any(|m| matches!(m.severity, Severity::Warning))
    }
    
    /// Convert to JSON for CI/CD integration
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self::new()
    }
}

// Serialization for JSON output
#[cfg(feature = "std")]
impl serde::Serialize for DiagnosticMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("DiagnosticMessage", 7)?;
        state.serialize_field("code", &self.code)?;
        state.serialize_field("message", &self.message)?;
        state.serialize_field("severity", &format!("{:?}", self.severity))?;
        state.serialize_field("context", &self.context)?;
        state.serialize_field("span_id", &self.span_id)?;
        state.serialize_field("source_location", &self.source_location)?;
        state.serialize_field("related", &self.related)?;
        state.end()
    }
}

#[cfg(feature = "std")]
impl serde::Serialize for SourceLocation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("SourceLocation", 3)?;
        state.serialize_field("file", &self.file)?;
        state.serialize_field("line", &self.line)?;
        state.serialize_field("column", &self.column)?;
        state.end()
    }
}

#[cfg(feature = "std")]
impl serde::Serialize for Diagnostics {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Diagnostics", 1)?;
        state.serialize_field("messages", &self.messages)?;
        state.end()
    }
}

/// Helper functions for common diagnostic patterns

/// Create a guard constraint violation diagnostic
pub fn guard_constraint_violation(actual: u64, max: u64) -> DiagnosticMessage {
    DiagnosticMessage::new(
        "GUARD_CONSTRAINT_VIOLATION",
        format!("Run length {} exceeds maximum allowed {}", actual, max),
    )
    .with_severity(Severity::Error)
    .with_context("actual_run_len", actual.to_string())
    .with_context("max_run_len", max.to_string())
    .with_context("constraint", "Chatman Constant (max_run_len ≤ 8)")
}

/// Create a performance budget violation diagnostic
pub fn performance_budget_violation(actual_ticks: u32, max_ticks: u32) -> DiagnosticMessage {
    DiagnosticMessage::new(
        "PERFORMANCE_BUDGET_VIOLATION",
        format!("Tick count {} exceeds budget {}", actual_ticks, max_ticks),
    )
    .with_severity(Severity::Error)
    .with_context("actual_ticks", actual_ticks.to_string())
    .with_context("max_ticks", max_ticks.to_string())
    .with_context("budget", "8 ticks (Chatman Constant)")
}

/// Create an SLO violation diagnostic
pub fn slo_violation(runtime_class: &str, actual_latency_ns: u64, slo_ns: u64) -> DiagnosticMessage {
    DiagnosticMessage::new(
        "SLO_VIOLATION",
        format!("{} latency {} ns exceeds SLO {} ns", runtime_class, actual_latency_ns, slo_ns),
    )
    .with_severity(Severity::Warning)
    .with_context("runtime_class", runtime_class.to_string())
    .with_context("actual_latency_ns", actual_latency_ns.to_string())
    .with_context("slo_ns", slo_ns.to_string())
}

/// Create a receipt validation error diagnostic
pub fn receipt_validation_error(receipt_id: &str, reason: &str) -> DiagnosticMessage {
    DiagnosticMessage::new(
        "RECEIPT_VALIDATION_ERROR",
        format!("Receipt {} validation failed: {}", receipt_id, reason),
    )
    .with_severity(Severity::Error)
    .with_context("receipt_id", receipt_id.to_string())
    .with_context("reason", reason.to_string())
}

/// Create a policy violation diagnostic
pub fn policy_violation(policy_name: &str, violation: &str) -> DiagnosticMessage {
    DiagnosticMessage::new(
        "POLICY_VIOLATION",
        format!("Policy {} violation: {}", policy_name, violation),
    )
    .with_severity(Severity::Error)
    .with_context("policy_name", policy_name.to_string())
    .with_context("violation", violation.to_string())
}

/// Format diagnostics for human-readable output (using miette)
#[cfg(feature = "std")]
pub fn format_diagnostics(diagnostics: &Diagnostics) -> String {
    use alloc::format;
    
    let mut output = String::new();
    
    for (i, msg) in diagnostics.messages().iter().enumerate() {
        if i > 0 {
            output.push_str("\n");
        }
        
        output.push_str(&format!("[{}] {}: {}\n", 
            match msg.severity {
                Severity::Info => "INFO",
                Severity::Warning => "WARN",
                Severity::Error => "ERROR",
                Severity::Critical => "CRITICAL",
            },
            msg.code,
            msg.message
        ));
        
        if !msg.context.is_empty() {
            output.push_str("  Context:\n");
            for (key, value) in &msg.context {
                output.push_str(&format!("    {}: {}\n", key, value));
            }
        }
        
        if let Some(ref span_id) = msg.span_id {
            output.push_str(&format!("  Span ID: {}\n", span_id));
        }
        
        if let Some(ref loc) = msg.source_location {
            output.push_str(&format!("  Location: {}:{}:{}\n", loc.file, loc.line, loc.column));
        }
        
        if !msg.related.is_empty() {
            output.push_str("  Related:\n");
            for related in &msg.related {
                output.push_str(&format!("    - {}: {}\n", related.code, related.message));
            }
        }
    }
    
    output
}

/// Format diagnostics as JSON for CI/CD
#[cfg(feature = "std")]
pub fn format_diagnostics_json(diagnostics: &Diagnostics) -> Result<String, serde_json::Error> {
    diagnostics.to_json()
}
