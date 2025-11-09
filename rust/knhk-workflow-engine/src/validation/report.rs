//! Validation Report Generation
//!
//! Generates comprehensive validation reports in multiple formats:
//! - JSON (for programmatic access)
//! - Markdown (for documentation)
//! - HTML (for visualization)

use crate::parser::WorkflowSpecId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Validation result for a single phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub phase: String,
    pub status: ValidationStatus,
    pub passed: usize,
    pub failed: usize,
    pub warnings: usize,
    pub skipped: usize,
    pub details: Vec<ValidationDetail>,
    pub metrics: HashMap<String, f64>,
}

/// Validation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStatus {
    Pass,
    Fail,
    Warning,
    Skipped,
}

/// Validation detail for individual test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationDetail {
    pub name: String,
    pub status: ValidationStatus,
    pub message: String,
    pub duration_ms: u64,
}

/// Comprehensive validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub spec_id: WorkflowSpecId,
    pub timestamp: String,
    pub phases: HashMap<String, ValidationResult>,
    pub summary: ReportSummary,
}

/// Report summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total_phases: usize,
    pub passed_phases: usize,
    pub failed_phases: usize,
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub warnings: usize,
    pub overall_status: ValidationStatus,
}

impl ValidationReport {
    /// Create a new validation report
    pub fn new(spec_id: WorkflowSpecId) -> Self {
        Self {
            spec_id,
            timestamp: chrono::Utc::now().to_rfc3339(),
            phases: HashMap::new(),
            summary: ReportSummary {
                total_phases: 0,
                passed_phases: 0,
                failed_phases: 0,
                total_tests: 0,
                passed_tests: 0,
                failed_tests: 0,
                warnings: 0,
                overall_status: ValidationStatus::Skipped,
            },
        }
    }

    /// Add phase result
    pub fn add_phase_result(&mut self, phase: &str, result: ValidationResult) {
        self.phases.insert(phase.to_string(), result.clone());
        self.update_summary();
    }

    /// Update summary from phase results
    fn update_summary(&mut self) {
        self.summary.total_phases = self.phases.len();
        self.summary.passed_phases = self
            .phases
            .values()
            .filter(|r| matches!(r.status, ValidationStatus::Pass))
            .count();
        self.summary.failed_phases = self
            .phases
            .values()
            .filter(|r| matches!(r.status, ValidationStatus::Fail))
            .count();

        self.summary.total_tests = self
            .phases
            .values()
            .map(|r| r.passed + r.failed + r.skipped)
            .sum();
        self.summary.passed_tests = self.phases.values().map(|r| r.passed).sum();
        self.summary.failed_tests = self.phases.values().map(|r| r.failed).sum();
        self.summary.warnings = self.phases.values().map(|r| r.warnings).sum();

        // Determine overall status
        if self.summary.failed_phases > 0 {
            self.summary.overall_status = ValidationStatus::Fail;
        } else if self.summary.passed_phases == self.summary.total_phases {
            self.summary.overall_status = ValidationStatus::Pass;
        } else if self.summary.warnings > 0 {
            self.summary.overall_status = ValidationStatus::Warning;
        } else {
            self.summary.overall_status = ValidationStatus::Skipped;
        }
    }

    /// Generate Markdown report
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str("# Van der Aalst Validation Report\n\n");
        md.push_str(&format!("**Workflow Spec ID**: {}\n", self.spec_id));
        md.push_str(&format!("**Timestamp**: {}\n\n", self.timestamp));

        // Summary
        md.push_str("## Summary\n\n");
        md.push_str(&format!(
            "- **Overall Status**: {:?}\n",
            self.summary.overall_status
        ));
        md.push_str(&format!(
            "- **Phases**: {} passed / {} failed / {} total\n",
            self.summary.passed_phases, self.summary.failed_phases, self.summary.total_phases
        ));
        md.push_str(&format!(
            "- **Tests**: {} passed / {} failed / {} total\n",
            self.summary.passed_tests, self.summary.failed_tests, self.summary.total_tests
        ));
        md.push_str(&format!("- **Warnings**: {}\n\n", self.summary.warnings));

        // Phase details
        md.push_str("## Phase Results\n\n");
        for (phase_name, phase_result) in &self.phases {
            md.push_str(&format!("### {}\n\n", phase_name));
            md.push_str(&format!("**Status**: {:?}\n", phase_result.status));
            md.push_str(&format!("- Passed: {}\n", phase_result.passed));
            md.push_str(&format!("- Failed: {}\n", phase_result.failed));
            md.push_str(&format!("- Warnings: {}\n", phase_result.warnings));
            md.push_str(&format!("- Skipped: {}\n\n", phase_result.skipped));

            // Metrics
            if !phase_result.metrics.is_empty() {
                md.push_str("**Metrics**:\n");
                for (metric_name, value) in &phase_result.metrics {
                    md.push_str(&format!("- {}: {:.2}\n", metric_name, value));
                }
                md.push_str("\n");
            }

            // Details
            if !phase_result.details.is_empty() {
                md.push_str("**Details**:\n");
                for detail in &phase_result.details {
                    md.push_str(&format!(
                        "- {}: {:?} - {} ({}ms)\n",
                        detail.name, detail.status, detail.message, detail.duration_ms
                    ));
                }
                md.push_str("\n");
            }
        }

        md
    }

    /// Generate JSON report
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Generate HTML report
    pub fn to_html(&self) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<title>Van der Aalst Validation Report</title>\n");
        html.push_str("<style>\n");
        html.push_str("body { font-family: Arial, sans-serif; margin: 20px; }\n");
        html.push_str(".pass { color: green; }\n");
        html.push_str(".fail { color: red; }\n");
        html.push_str(".warning { color: orange; }\n");
        html.push_str(".skipped { color: gray; }\n");
        html.push_str("</style>\n");
        html.push_str("</head>\n<body>\n");
        html.push_str("<h1>Van der Aalst Validation Report</h1>\n");
        html.push_str(&format!(
            "<p><strong>Workflow Spec ID</strong>: {}</p>\n",
            self.spec_id
        ));
        html.push_str(&format!(
            "<p><strong>Timestamp</strong>: {}</p>\n",
            self.timestamp
        ));
        html.push_str("<h2>Summary</h2>\n");
        html.push_str(&format!(
            "<p><strong>Overall Status</strong>: <span class=\"{:?}\">{:?}</span></p>\n",
            self.summary.overall_status, self.summary.overall_status
        ));
        html.push_str("</body>\n</html>\n");
        html
    }
}
