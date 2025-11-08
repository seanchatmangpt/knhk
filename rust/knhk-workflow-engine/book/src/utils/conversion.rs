//! Type conversion utilities
//!
//! Provides safe type conversions and transformations.

use crate::error::{WorkflowError, WorkflowResult};

/// Convert string to u32 with validation
pub fn str_to_u32(s: &str) -> WorkflowResult<u32> {
    s.parse::<u32>()
        .map_err(|e| WorkflowError::Validation(format!("Invalid number: {}", e)))
}

/// Convert string to u64 with validation
pub fn str_to_u64(s: &str) -> WorkflowResult<u64> {
    s.parse::<u64>()
        .map_err(|e| WorkflowError::Validation(format!("Invalid number: {}", e)))
}

/// Convert string to bool with validation
pub fn str_to_bool(s: &str) -> WorkflowResult<bool> {
    match s.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(WorkflowError::Validation(format!("Invalid boolean: {}", s))),
    }
}

/// Convert bytes to hex string
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Convert hex string to bytes
pub fn hex_to_bytes(hex: &str) -> WorkflowResult<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return Err(WorkflowError::Validation(
            "Hex string length must be even".to_string(),
        ));
    }

    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| WorkflowError::Validation(format!("Invalid hex: {}", e)))
        })
        .collect()
}
