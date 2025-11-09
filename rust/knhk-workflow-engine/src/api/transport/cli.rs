//! CLI transport adapter
//!
//! Converts between CLI-specific types (command arguments, output) and unified models.

use crate::api::models::errors::ApiError;
use std::fmt;

/// CLI adapter for converting between CLI types and unified models
pub struct CliAdapter;

impl CliAdapter {
    /// Convert ApiError to CLI error message
    pub fn error_to_message(error: &ApiError) -> String {
        format!("Error [{}]: {}", error.code, error.message)
    }

    /// Convert ApiError to CLI exit code
    pub fn error_to_exit_code(error: &ApiError) -> i32 {
        match error.code.as_str() {
            "NOT_FOUND" => 2,
            "BAD_REQUEST" | "VALIDATION_ERROR" => 3,
            "INTERNAL_ERROR" => 1,
            "TIMEOUT" => 124,
            "RESOURCE_UNAVAILABLE" => 5,
            _ => 1,
        }
    }

    /// Format response for CLI output
    pub fn format_response<T: fmt::Display>(response: &T) -> String {
        format!("{}", response)
    }

    /// Format error for CLI output
    pub fn format_error(error: &ApiError) -> String {
        let mut output = format!("Error [{}]: {}", error.code, error.message);
        if let Some(ref details) = error.details {
            output.push_str(&format!("\nDetails: {}", details));
        }
        output
    }
}
