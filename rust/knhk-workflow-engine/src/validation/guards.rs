//! Guard constraint validation
//!
//! Validates guard constraints including max_run_len, max_batch_size, etc.

use crate::error::{WorkflowError, WorkflowResult};

/// Maximum run length (Chatman Constant: ≤8)
pub const MAX_RUN_LEN: usize = 8;

/// Maximum batch size
pub const MAX_BATCH_SIZE: usize = 1000;

/// Validate run length
pub fn validate_run_len(len: usize) -> WorkflowResult<()> {
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
    if size > MAX_BATCH_SIZE {
        return Err(WorkflowError::Validation(format!(
            "Batch size {} exceeds maximum {}",
            size, MAX_BATCH_SIZE
        )));
    }
    Ok(())
}

/// Validate pattern ID
pub fn validate_pattern_id(id: u32) -> WorkflowResult<()> {
    if id < 1 || id > 43 {
        return Err(WorkflowError::Validation(format!(
            "Pattern ID {} must be between 1 and 43",
            id
        )));
    }
    Ok(())
}
