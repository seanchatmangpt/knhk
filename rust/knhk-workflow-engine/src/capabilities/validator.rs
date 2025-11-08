//! Capability validator implementation
//!
//! Provides runtime validation of capabilities.

use crate::capabilities::{CapabilityRegistry, CapabilityStatus, CapabilityValidationReport};
use crate::error::{WorkflowError, WorkflowResult};
use crate::patterns::PatternRegistry;

/// Runtime capability validator
pub struct CapabilityValidator;

impl CapabilityValidator {
    /// Validate pattern registry
    pub fn validate_pattern_registry(registry: &PatternRegistry) -> WorkflowResult<()> {
        let patterns = registry.list_patterns();

        // Check all 43 patterns are registered
        if patterns.len() < 43 {
            return Err(WorkflowError::Validation(format!(
                "Expected 43 patterns, found {}",
                patterns.len()
            )));
        }

        // Check pattern IDs are in range 1-43
        for pattern_id in &patterns {
            if pattern_id.0 < 1 || pattern_id.0 > 43 {
                return Err(WorkflowError::Validation(format!(
                    "Invalid pattern ID: {}",
                    pattern_id.0
                )));
            }
        }

        Ok(())
    }

    /// Validate all capabilities
    pub fn validate_all() -> WorkflowResult<CapabilityValidationReport> {
        let capability_registry = CapabilityRegistry::new();

        // Validate required capabilities
        capability_registry.validate_required()?;

        // Generate report
        let implemented = capability_registry.by_status(CapabilityStatus::Implemented);
        let partial = capability_registry.by_status(CapabilityStatus::Partial);
        let untested = capability_registry.by_status(CapabilityStatus::ImplementedUntested);
        let not_implemented = capability_registry.by_status(CapabilityStatus::NotImplemented);

        Ok(CapabilityValidationReport {
            total: capability_registry.list().len(),
            implemented: implemented.len(),
            partial: partial.len(),
            untested: untested.len(),
            not_implemented: not_implemented.len(),
            required_available: capability_registry
                .list()
                .iter()
                .filter(|c| c.required && c.status.is_production_ready())
                .count(),
            required_total: capability_registry
                .list()
                .iter()
                .filter(|c| c.required)
                .count(),
        })
    }

    /// Check if system is production-ready
    pub fn is_production_ready() -> WorkflowResult<bool> {
        let report = Self::validate_all()?;
        Ok(report.all_required_available() && report.production_readiness() == 100.0)
    }
}
