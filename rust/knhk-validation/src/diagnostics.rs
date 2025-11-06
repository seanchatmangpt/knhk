// rust/knhk-validation/src/diagnostics.rs
// Diagnostic System - Inspired by Weaver's diagnostic architecture
// Structured diagnostics with rich context and multiple output formats

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use serde::{Deserialize, Serialize};

/// Diagnostic severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    /// Information - useful context
    Info,
    /// Warning - potential issue
    Warning,
    /// Error - validation failure
    Error,
    /// Fatal - unrecoverable error
    Fatal,
}

/// Diagnostic message with rich context
/// Inspired by Weaver's DiagnosticMessage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticMessage {
    /// Severity level
    pub severity: DiagnosticSeverity,
    /// Message code (e.g., "E001", "W042")
    pub code: String,
    /// Human-readable message
    pub message: String,
    /// Source location (file, line, column)
    pub location: Option<DiagnosticLocation>,
    /// Additional context
    pub context: BTreeMap<String, String>,
    /// Related diagnostics
    pub related: Vec<DiagnosticMessage>,
}

/// Source location for diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticLocation {
    /// File path
    pub file: String,
    /// Line number (1-based)
    pub line: Option<u32>,
    /// Column number (1-based)
    pub column: Option<u32>,
}

impl DiagnosticMessage {
    /// Create a new diagnostic message
    pub fn new(severity: DiagnosticSeverity, code: String, message: String) -> Self {
        Self {
            severity,
            code,
            message,
            location: None,
            context: BTreeMap::new(),
            related: Vec::new(),
        }
    }
    
    /// Add source location
    pub fn with_location(mut self, location: DiagnosticLocation) -> Self {
        self.location = Some(location);
        self
    }
    
    /// Add context key-value pair
    pub fn with_context(mut self, key: String, value: String) -> Self {
        self.context.insert(key, value);
        self
    }
    
    /// Add related diagnostic
    pub fn with_related(mut self, related: DiagnosticMessage) -> Self {
        self.related.push(related);
        self
    }
    
    /// Format as ANSI string (for terminal output)
    pub fn format_ansi(&self) -> String {
        use alloc::format;
        
        let severity_str = match self.severity {
            DiagnosticSeverity::Info => "ℹ",
            DiagnosticSeverity::Warning => "⚠",
            DiagnosticSeverity::Error => "✗",
            DiagnosticSeverity::Fatal => "✗",
        };
        
        let mut output = format!("{} [{}] {}", severity_str, self.code, self.message);
        
        if let Some(ref location) = self.location {
            output.push_str(&format!("\n  at {}:{}:{}", 
                location.file,
                location.line.map(|l| l.to_string()).unwrap_or_else(|| "?".to_string()),
                location.column.map(|c| c.to_string()).unwrap_or_else(|| "?".to_string())
            ));
        }
        
        if !self.context.is_empty() {
            output.push_str("\n  context:");
            for (key, value) in &self.context {
                output.push_str(&format!("\n    {}: {}", key, value));
            }
        }
        
        if !self.related.is_empty() {
            output.push_str("\n  related:");
            for related in &self.related {
                output.push_str(&format!("\n    {}", related.format_ansi()));
            }
        }
        
        output
    }
    
    /// Format as JSON string
    pub fn format_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Collection of diagnostic messages
/// Inspired by Weaver's DiagnosticMessages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticMessages {
    /// Diagnostic messages
    pub messages: Vec<DiagnosticMessage>,
    /// Total count by severity
    pub counts: DiagnosticCounts,
}

/// Counts of diagnostics by severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticCounts {
    pub info: usize,
    pub warning: usize,
    pub error: usize,
    pub fatal: usize,
}

impl DiagnosticCounts {
    pub fn new() -> Self {
        Self {
            info: 0,
            warning: 0,
            error: 0,
            fatal: 0,
        }
    }
    
    pub fn total(&self) -> usize {
        self.info + self.warning + self.error + self.fatal
    }
}

impl Default for DiagnosticCounts {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticMessages {
    /// Create new diagnostic messages collection
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            counts: DiagnosticCounts::new(),
        }
    }
    
    /// Add a diagnostic message
    pub fn add(&mut self, message: DiagnosticMessage) {
        match message.severity {
            DiagnosticSeverity::Info => self.counts.info += 1,
            DiagnosticSeverity::Warning => self.counts.warning += 1,
            DiagnosticSeverity::Error => self.counts.error += 1,
            DiagnosticSeverity::Fatal => self.counts.fatal += 1,
        }
        self.messages.push(message);
    }
    
    /// Check if there are any errors or fatal diagnostics
    pub fn has_errors(&self) -> bool {
        self.counts.error > 0 || self.counts.fatal > 0
    }
    
    /// Format all messages as ANSI string
    pub fn format_ansi(&self) -> String {
        use alloc::format;
        
        let mut output = String::new();
        
        if !self.messages.is_empty() {
            output.push_str("Diagnostic Report:\n\n");
            
            for message in &self.messages {
                output.push_str(&message.format_ansi());
                output.push_str("\n\n");
            }
            
            output.push_str(&format!(
                "Total: {} ({} info, {} warnings, {} errors, {} fatal)",
                self.counts.total(),
                self.counts.info,
                self.counts.warning,
                self.counts.error,
                self.counts.fatal
            ));
        }
        
        output
    }
    
    /// Format all messages as JSON string
    pub fn format_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl Default for DiagnosticMessages {
    fn default() -> Self {
        Self::new()
    }
}

/// Diagnostic format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticFormat {
    /// ANSI formatted (for terminal)
    Ansi,
    /// JSON formatted (for CI/CD)
    Json,
    /// GitHub workflow command format
    GitHubWorkflow,
}

impl DiagnosticFormat {
    /// Format diagnostics according to format
    pub fn format(&self, diagnostics: &DiagnosticMessages) -> Result<String, serde_json::Error> {
        match self {
            DiagnosticFormat::Ansi => Ok(diagnostics.format_ansi()),
            DiagnosticFormat::Json => diagnostics.format_json(),
            DiagnosticFormat::GitHubWorkflow => {
                // Format as GitHub workflow commands
                let mut output = String::new();
                for message in &diagnostics.messages {
                    if let Some(ref location) = message.location {
                        let level = match message.severity {
                            DiagnosticSeverity::Info | DiagnosticSeverity::Warning => "warning",
                            DiagnosticSeverity::Error | DiagnosticSeverity::Fatal => "error",
                        };
                        output.push_str(&format!(
                            "::{} file={},line={},col={}::[{}] {}\n",
                            level,
                            location.file,
                            location.line.unwrap_or(0),
                            location.column.unwrap_or(0),
                            message.code,
                            message.message
                        ));
                    }
                }
                Ok(output)
            }
        }
    }
}
