//! DFLSS metrics structures

use crate::internal::capability::ProcessCapability;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export quality metrics
pub use crate::internal::quality::{QualityCategories, QualityMetrics};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub operation_ticks: HashMap<String, Vec<f64>>,
    pub operations_under_8_ticks: u32,
    pub total_operations: u32,
    pub median_ticks: f64,
    pub p95_ticks: f64,
    pub p99_ticks: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaverMetrics {
    pub static_pass: bool,
    pub live_pass: Option<bool>,
    pub validations: u32,
    pub failures: u32,
    pub pass_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DflssMetrics {
    pub timestamp: String,
    pub quality: QualityMetrics,
    pub performance: PerformanceMetrics,
    pub weaver: WeaverMetrics,
    pub capability: Option<ProcessCapability>,
    pub dod_compliance: f64,
    pub sigma_level: f64,
    pub dpmo: f64,
}
