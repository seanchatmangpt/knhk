//! Validation result structures

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationStatus {
    Pass,
    Fail,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub status: ValidationStatus,
    pub message: String,
    pub details: Vec<ValidationDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationDetail {
    pub criterion: String,
    pub status: ValidationStatus,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub warnings: u32,
    pub results: Vec<ValidationResult>,
}
