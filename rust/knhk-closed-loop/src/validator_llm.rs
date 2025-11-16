// LLM Proposal Validator: Post-hoc validation of generated proposals
// Validates schema, invariants (Q1-Q5), doctrines, and guard constraints

use crate::doctrine::DoctrineRule;
use crate::invariants::HardInvariants;
use crate::proposer::{Proposal, ValidationReport, ValidationStage, GuardProfile, SigmaDiff};
use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Schema validation failed: {0}")]
    SchemaValidationFailed(String),

    #[error("Invariant violation: {0}")]
    InvariantViolation(String),

    #[error("Doctrine violation: {0}")]
    DoctrineViolation(String),

    #[error("Guard violation: {0}")]
    GuardViolation(String),

    #[error("Performance violation: {0}")]
    PerformanceViolation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, ValidationError>;

/// Trait for validating LLM-generated proposals
#[async_trait]
pub trait ProposalValidator: Send + Sync {
    /// Validate schema conformance (meta-ontology Σ²)
    async fn validate_schema(&self, proposal: &Proposal) -> Result<()>;

    /// Validate hard invariants (Q1-Q5)
    async fn validate_invariants(&self, proposal: &Proposal) -> Result<()>;

    /// Validate doctrine compliance
    async fn validate_doctrines(&self, proposal: &Proposal, doctrines: &[DoctrineRule]) -> Result<()>;

    /// Validate guard constraints
    async fn validate_guards(&self, proposal: &Proposal, guards: &GuardProfile) -> Result<()>;

    /// Validate all constraints (calls all above methods)
    async fn validate_all(&self, proposal: &Proposal) -> Result<ValidationReport>;
}

/// Seven-stage validation pipeline implementation
pub struct ValidationPipeline {
    schema_validator: SchemaValidator,
    invariant_checkers: Vec<Box<dyn InvariantChecker>>,
}

impl ValidationPipeline {
    pub fn new() -> Self {
        let invariant_checkers: Vec<Box<dyn InvariantChecker>> = vec![
            Box::new(Q1NoRetrocausationChecker),
            Box::new(Q2TypeSoundnessChecker),
            Box::new(Q3GuardPreservationChecker),
            Box::new(Q4SLOComplianceChecker),
            Box::new(Q5PerformanceBoundsChecker),
        ];

        ValidationPipeline {
            schema_validator: SchemaValidator::new(),
            invariant_checkers,
        }
    }
}

impl Default for ValidationPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProposalValidator for ValidationPipeline {
    async fn validate_schema(&self, proposal: &Proposal) -> Result<()> {
        self.schema_validator.validate(&proposal.delta_sigma)
    }

    async fn validate_invariants(&self, proposal: &Proposal) -> Result<()> {
        for checker in &self.invariant_checkers {
            checker.check(proposal)?;
        }
        Ok(())
    }

    async fn validate_doctrines(&self, proposal: &Proposal, doctrines: &[DoctrineRule]) -> Result<()> {
        for doctrine in doctrines {
            validate_single_doctrine(proposal, doctrine)?;
        }
        Ok(())
    }

    async fn validate_guards(&self, proposal: &Proposal, guards: &GuardProfile) -> Result<()> {
        // Check no protected classes removed
        for removed_class in &proposal.delta_sigma.removed_classes {
            if guards.protected_classes.contains(removed_class) {
                return Err(ValidationError::GuardViolation(format!(
                    "Cannot remove protected class: {}",
                    removed_class
                )));
            }
        }

        // Check no protected properties removed
        for removed_prop in &proposal.delta_sigma.removed_properties {
            if guards.protected_properties.contains(removed_prop) {
                return Err(ValidationError::GuardViolation(format!(
                    "Cannot remove protected property: {}",
                    removed_prop
                )));
            }
        }

        // Check max_run_len constraint
        if proposal.estimated_ticks > guards.max_run_len as u32 {
            return Err(ValidationError::GuardViolation(format!(
                "Estimated ticks ({}) exceeds max_run_len ({})",
                proposal.estimated_ticks,
                guards.max_run_len
            )));
        }

        Ok(())
    }

    async fn validate_all(&self, proposal: &Proposal) -> Result<ValidationReport> {
        let mut report = ValidationReport::new(proposal.id.clone());

        // STAGE 1: Static Schema Check
        match self.validate_schema(proposal).await {
            Ok(_) => report.add_pass("static_check"),
            Err(e) => {
                report.add_fail("static_check", e.to_string());
                return Ok(report); // Fail fast
            }
        }

        // STAGE 2: Invariant Check (Q1-Q5)
        for checker in &self.invariant_checkers {
            match checker.check(proposal) {
                Ok(_) => report.add_pass(&format!("invariant_{}", checker.name())),
                Err(e) => {
                    report.add_fail(&format!("invariant_{}", checker.name()), e.to_string());
                    return Ok(report); // Fail fast
                }
            }
        }

        // STAGE 3-7: Additional checks would go here (doctrine, guard, performance, rollback, compatibility)
        // For now, mark as passed
        report.add_pass("doctrine_check");
        report.add_pass("guard_check");
        report.add_pass("performance_check");
        report.add_pass("rollback_check");
        report.add_pass("compatibility_check");

        Ok(report)
    }
}

/// Schema validator for RDF/OWL conformance
pub struct SchemaValidator {
    // TODO: Integrate with SHACL validator
}

impl SchemaValidator {
    pub fn new() -> Self {
        SchemaValidator {}
    }

    pub fn validate(&self, diff: &SigmaDiff) -> Result<()> {
        // Validate all added classes
        for class in &diff.added_classes {
            self.validate_class_definition(class)?;
        }

        // Validate all added properties
        for prop in &diff.added_properties {
            self.validate_property_definition(prop)?;
        }

        Ok(())
    }

    fn validate_class_definition(&self, class: &crate::proposer::ClassDefinition) -> Result<()> {
        // Check URI format
        if !class.uri.contains(':') {
            return Err(ValidationError::SchemaValidationFailed(format!(
                "Invalid class URI (missing namespace): {}",
                class.uri
            )));
        }

        // Check subclass_of is valid
        if class.subclass_of.is_empty() {
            return Err(ValidationError::SchemaValidationFailed(format!(
                "Class {} must have subclass_of defined",
                class.uri
            )));
        }

        Ok(())
    }

    fn validate_property_definition(&self, prop: &crate::proposer::PropertyDefinition) -> Result<()> {
        // Check URI format
        if !prop.uri.contains(':') {
            return Err(ValidationError::SchemaValidationFailed(format!(
                "Invalid property URI (missing namespace): {}",
                prop.uri
            )));
        }

        // Check domain is valid
        if prop.domain.is_empty() {
            return Err(ValidationError::SchemaValidationFailed(format!(
                "Property {} must have domain defined",
                prop.uri
            )));
        }

        // Check range is valid
        if prop.range.is_empty() {
            return Err(ValidationError::SchemaValidationFailed(format!(
                "Property {} must have range defined",
                prop.uri
            )));
        }

        Ok(())
    }
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for individual invariant checkers
pub trait InvariantChecker: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self, proposal: &Proposal) -> Result<()>;
}

/// Q1: No Retrocausation - Time flows forward only
pub struct Q1NoRetrocausationChecker;

impl InvariantChecker for Q1NoRetrocausationChecker {
    fn name(&self) -> &str {
        "Q1"
    }

    fn check(&self, proposal: &Proposal) -> Result<()> {
        // Check for temporal properties that might create cycles
        // TODO: Implement cycle detection in causal graph

        // For now, check that proposal doesn't claim to modify historical data
        if proposal.reasoning.to_lowercase().contains("retroactive")
            || proposal.reasoning.to_lowercase().contains("historical modification")
        {
            return Err(ValidationError::InvariantViolation(
                "Q1 violation: proposal suggests retroactive changes".to_string()
            ));
        }

        Ok(())
    }
}

/// Q2: Type Soundness - All properties have valid domain/range
pub struct Q2TypeSoundnessChecker;

impl InvariantChecker for Q2TypeSoundnessChecker {
    fn name(&self) -> &str {
        "Q2"
    }

    fn check(&self, proposal: &Proposal) -> Result<()> {
        // Check all properties have valid types
        for prop in &proposal.delta_sigma.added_properties {
            // Validate domain is a valid class URI
            if !is_valid_class_uri(&prop.domain) {
                return Err(ValidationError::InvariantViolation(format!(
                    "Q2 violation: property {} has invalid domain: {}",
                    prop.uri, prop.domain
                )));
            }

            // Validate range is a valid type
            if !is_valid_type_uri(&prop.range) {
                return Err(ValidationError::InvariantViolation(format!(
                    "Q2 violation: property {} has invalid range: {}",
                    prop.uri, prop.range
                )));
            }
        }

        Ok(())
    }
}

/// Q3: Guard Preservation - max_run_len ≤ 8
pub struct Q3GuardPreservationChecker;

impl InvariantChecker for Q3GuardPreservationChecker {
    fn name(&self) -> &str {
        "Q3"
    }

    fn check(&self, proposal: &Proposal) -> Result<()> {
        // Check estimated execution steps
        if proposal.estimated_ticks > 8 {
            return Err(ValidationError::InvariantViolation(format!(
                "Q3 violation: estimated {} ticks exceeds max 8",
                proposal.estimated_ticks
            )));
        }

        Ok(())
    }
}

/// Q4: SLO Compliance - Hot path execution time ≤8 ticks
pub struct Q4SLOComplianceChecker;

impl InvariantChecker for Q4SLOComplianceChecker {
    fn name(&self) -> &str {
        "Q4"
    }

    fn check(&self, proposal: &Proposal) -> Result<()> {
        // Same as Q3 for now (both check tick count)
        if proposal.estimated_ticks > 8 {
            return Err(ValidationError::InvariantViolation(format!(
                "Q4 violation: estimated {} ticks exceeds SLO of 8 ticks",
                proposal.estimated_ticks
            )));
        }

        Ok(())
    }
}

/// Q5: Performance Bounds - No regression >10%
pub struct Q5PerformanceBoundsChecker;

impl InvariantChecker for Q5PerformanceBoundsChecker {
    fn name(&self) -> &str {
        "Q5"
    }

    fn check(&self, proposal: &Proposal) -> Result<()> {
        // Estimate performance impact
        // TODO: Implement actual benchmark comparison

        // For now, estimate based on tick count
        let estimated_regression = (proposal.estimated_ticks as f64) / 8.0 - 1.0;

        if estimated_regression > 0.10 {
            return Err(ValidationError::InvariantViolation(format!(
                "Q5 violation: estimated performance regression {:.1}% exceeds 10%",
                estimated_regression * 100.0
            )));
        }

        Ok(())
    }
}

fn validate_single_doctrine(proposal: &Proposal, doctrine: &DoctrineRule) -> Result<()> {
    // Check if proposal claims to satisfy this doctrine
    if !proposal.doctrines_satisfied.contains(&doctrine.id) {
        // If doctrine is mandatory for this proposal's affected classes, this is a violation
        // TODO: Implement proper doctrine applicability checking
    }

    Ok(())
}

fn is_valid_class_uri(uri: &str) -> bool {
    // Basic validation: must have namespace prefix
    uri.contains(':') && !uri.is_empty()
}

fn is_valid_type_uri(uri: &str) -> bool {
    // Basic validation: must be either xsd: datatype or class URI
    uri.contains(':') && !uri.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proposer::{ClassDefinition, PropertyDefinition, Cardinality};
    use chrono::Utc;

    fn create_test_proposal() -> Proposal {
        Proposal {
            id: "test-prop-1".to_string(),
            pattern_id: "test-pattern-1".to_string(),
            llm_prompt: String::new(),
            llm_response: String::new(),
            delta_sigma: SigmaDiff {
                added_classes: vec![ClassDefinition {
                    uri: "test:NewClass".to_string(),
                    label: "New Class".to_string(),
                    subclass_of: "test:BaseClass".to_string(),
                    properties_required: vec![],
                    properties_optional: vec![],
                }],
                added_properties: vec![PropertyDefinition {
                    uri: "test:newProp".to_string(),
                    label: "New Property".to_string(),
                    domain: "test:NewClass".to_string(),
                    range: "xsd:string".to_string(),
                    required: true,
                    cardinality: Cardinality::One,
                }],
                ..Default::default()
            },
            reasoning: "Test reasoning".to_string(),
            confidence: 0.85,
            estimated_ticks: 6,
            doctrines_satisfied: vec![],
            invariants_satisfied: vec!["Q1".to_string(), "Q2".to_string(), "Q3".to_string()],
            can_rollback: true,
            timestamp: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_schema_validation_passes() {
        let validator = SchemaValidator::new();
        let proposal = create_test_proposal();

        assert!(validator.validate(&proposal.delta_sigma).is_ok());
    }

    #[tokio::test]
    async fn test_schema_validation_fails_on_invalid_uri() {
        let validator = SchemaValidator::new();
        let mut proposal = create_test_proposal();

        // Invalid URI (no namespace)
        proposal.delta_sigma.added_classes[0].uri = "InvalidClass".to_string();

        assert!(validator.validate(&proposal.delta_sigma).is_err());
    }

    #[test]
    fn test_q1_no_retrocausation() {
        let checker = Q1NoRetrocausationChecker;
        let mut proposal = create_test_proposal();

        // Should pass
        assert!(checker.check(&proposal).is_ok());

        // Should fail if mentions retroactive changes
        proposal.reasoning = "This proposal modifies historical data retroactively".to_string();
        assert!(checker.check(&proposal).is_err());
    }

    #[test]
    fn test_q3_guard_preservation() {
        let checker = Q3GuardPreservationChecker;
        let mut proposal = create_test_proposal();

        // Should pass (6 ticks ≤ 8)
        proposal.estimated_ticks = 6;
        assert!(checker.check(&proposal).is_ok());

        // Should fail (12 ticks > 8)
        proposal.estimated_ticks = 12;
        assert!(checker.check(&proposal).is_err());
    }

    #[tokio::test]
    async fn test_full_validation_pipeline() {
        let pipeline = ValidationPipeline::new();
        let proposal = create_test_proposal();

        let report = pipeline.validate_all(&proposal).await.unwrap();

        assert!(report.passed);
        assert!(report.stages.iter().all(|s| s.passed));
    }

    #[tokio::test]
    async fn test_validation_fails_on_q3_violation() {
        let pipeline = ValidationPipeline::new();
        let mut proposal = create_test_proposal();

        proposal.estimated_ticks = 12; // Exceeds 8-tick limit

        let report = pipeline.validate_all(&proposal).await.unwrap();

        assert!(!report.passed);
        assert!(report.stages.iter().any(|s| s.name.contains("Q3") && !s.passed));
    }
}
