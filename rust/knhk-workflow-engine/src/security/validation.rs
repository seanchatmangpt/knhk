//! Security validation utilities
//!
//! Provides input validation, sanitization, and security checks.

use crate::error::{WorkflowError, WorkflowResult};

/// Validate IRI format
pub fn validate_iri(iri: &str) -> WorkflowResult<()> {
    if iri.is_empty() {
        return Err(WorkflowError::Validation("IRI cannot be empty".to_string()));
    }

    // Basic IRI validation (RFC 3987)
    if !iri.starts_with("http://") && !iri.starts_with("https://") && !iri.starts_with("urn:") {
        return Err(WorkflowError::Validation(format!(
            "Invalid IRI format: {}",
            iri
        )));
    }

    Ok(())
}

/// Sanitize string input
pub fn sanitize_string(input: &str) -> String {
    // Remove control characters and normalize whitespace
    input
        .chars()
        .filter(|c| !c.is_control())
        .map(|c| if c.is_whitespace() { ' ' } else { c })
        .collect()
}

/// Validate pattern ID range
pub fn validate_pattern_id(id: u32) -> WorkflowResult<()> {
    if id < 1 || id > 43 {
        return Err(WorkflowError::Validation(format!(
            "Pattern ID {} must be between 1 and 43",
            id
        )));
    }
    Ok(())
}

/// Validate run length (Chatman Constant: ≤8)
pub fn validate_run_len(len: usize) -> WorkflowResult<()> {
    const MAX_RUN_LEN: usize = 8;
    if len > MAX_RUN_LEN {
        return Err(WorkflowError::Validation(format!(
            "Run length {} exceeds maximum {}",
            len, MAX_RUN_LEN
        )));
    }
    Ok(())
}

/// Validate batch size
pub fn validate_batch_size(size: usize) -> WorkflowResult<()> {
    const MAX_BATCH_SIZE: usize = 1000;
    if size > MAX_BATCH_SIZE {
        return Err(WorkflowError::Validation(format!(
            "Batch size {} exceeds maximum {}",
            size, MAX_BATCH_SIZE
        )));
    }
    Ok(())
}

/// Validate priority (0-255)
pub fn validate_priority(priority: u32) -> WorkflowResult<()> {
    if priority > 255 {
        return Err(WorkflowError::Validation(format!(
            "Priority {} exceeds maximum 255",
            priority
        )));
    }
    Ok(())
}
