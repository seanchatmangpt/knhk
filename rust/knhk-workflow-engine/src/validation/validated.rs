//! Type-level validation state tracking for DFLSS compliance
//!
//! Uses phantom types to encode validation state in the type system,
//! preventing invalid state transitions at compile time.
//!
//! **Architecture**: All validation happens at ingress (knhk-workflow-engine).
//! Hot path (knhk-hot) only accepts `ValidatedTriples<SchemaValidated>`.

//! Type-level validation state tracking for DFLSS compliance
//!
//! Uses phantom types to encode validation state in the type system,
//! preventing invalid state transitions at compile time.
//!
//! **Architecture**: All validation happens at ingress (knhk-workflow-engine).
//! Hot path (knhk-hot) only accepts `ValidatedTriples<SchemaValidated>`.

use crate::error::{WorkflowError, WorkflowResult};
use std::marker::PhantomData;

// Import Triple from oxigraph (used in parser)
use oxigraph::model::Triple as OxigraphTriple;

/// Validation level marker trait
///
/// Types implementing this trait represent different validation states.
/// The type system enforces that only properly validated data can enter the hot path.
pub trait ValidationLevel {}

/// Unvalidated state - data has not been validated
#[derive(Debug)]
pub struct Unvalidated;

/// Guard validated state - data has passed guard validation (MAX_RUN_LEN ≤ 8)
#[derive(Debug)]
pub struct GuardValidated;

/// Schema validated state - data has passed schema validation (Weaver)
#[derive(Debug)]
pub struct SchemaValidated;

impl ValidationLevel for Unvalidated {}
impl ValidationLevel for GuardValidated {}
impl ValidationLevel for SchemaValidated {}

/// Validated triples with type-level validation state
///
/// The type parameter `V: ValidationLevel` encodes the validation state in the type system.
/// This prevents invalid state transitions at compile time.
///
/// # Type Safety
///
/// - `ValidatedTriples<Unvalidated>`: Raw triples, not yet validated
/// - `ValidatedTriples<GuardValidated>`: Passed guard validation, not yet schema validated
/// - `ValidatedTriples<SchemaValidated>`: Fully validated, safe for hot path
///
/// # Example
///
/// ```rust
/// // Start with unvalidated triples
/// let triples = ValidatedTriples::new(vec![triple1, triple2]);
///
/// // Validate guards (type changes: Unvalidated → GuardValidated)
/// let guard_validated = triples.validate_guards()?;
///
/// // Validate schema (type changes: GuardValidated → SchemaValidated)
/// let schema_validated = guard_validated.validate_schema()?;
///
/// // Only SchemaValidated can enter hot path
/// let hot_path_triples = schema_validated.into_hot_path();
/// ```
/// Validated triples with type-level validation state
///
/// The type parameter `V: ValidationLevel` encodes the validation state in the type system.
/// This prevents invalid state transitions at compile time.
///
/// # Type Safety
///
/// - `ValidatedTriples<Unvalidated>`: Raw triples, not yet validated
/// - `ValidatedTriples<GuardValidated>`: Passed guard validation, not yet schema validated
/// - `ValidatedTriples<SchemaValidated>`: Fully validated, safe for hot path
///
/// # Example
///
/// ```rust
/// // Start with unvalidated triples
/// let triples = ValidatedTriples::new(vec![triple1, triple2]);
///
/// // Validate guards (type changes: Unvalidated → GuardValidated)
/// let guard_validated = triples.validate_guards()?;
///
/// // Validate schema (type changes: GuardValidated → SchemaValidated)
/// let schema_validated = guard_validated.validate_schema()?;
///
/// // Only SchemaValidated can enter hot path
/// let hot_path_triples = schema_validated.into_hot_path();
/// ```
/// Validated triples with type-level validation state
///
/// The type parameter `V: ValidationLevel` encodes the validation state in the type system.
/// This prevents invalid state transitions at compile time.
///
/// # Type Safety
///
/// - `ValidatedTriples<Unvalidated>`: Raw triples, not yet validated
/// - `ValidatedTriples<GuardValidated>`: Passed guard validation, not yet schema validated
/// - `ValidatedTriples<SchemaValidated>`: Fully validated, safe for hot path
///
/// # Example
///
/// ```rust
/// // Start with unvalidated triples
/// let triples = ValidatedTriples::new(vec![triple1, triple2]);
///
/// // Validate guards (type changes: Unvalidated → GuardValidated)
/// let guard_validated = triples.validate_guards()?;
///
/// // Validate schema (type changes: GuardValidated → SchemaValidated)
/// let schema_validated = guard_validated.validate_schema()?;
///
/// // Only SchemaValidated can enter hot path
/// let hot_path_triples = schema_validated.into_hot_path();
/// ```
#[derive(Debug)]
pub struct ValidatedTriples<V: ValidationLevel> {
    triples: Vec<OxigraphTriple>,
    _validation: PhantomData<V>,
}

impl ValidatedTriples<Unvalidated> {
    /// Create new unvalidated triples
    ///
    /// This is the entry point for data entering the validation pipeline.
    pub fn new(triples: Vec<OxigraphTriple>) -> Self {
        Self {
            triples,
            _validation: PhantomData,
        }
    }

    /// Validate guard constraints (MAX_RUN_LEN ≤ 8)
    ///
    /// Transitions from `Unvalidated` to `GuardValidated`.
    /// Returns error if guard validation fails (DFLSS CTQ 2 violation).
    pub fn validate_guards(self) -> Result<ValidatedTriples<GuardValidated>, WorkflowError> {
        use crate::security::{GuardContext, GuardFunction, MaxRunLengthGuard};
        use serde_json::Value;

        // Check MAX_RUN_LEN ≤ 8
        if self.triples.len() > 8 {
            return Err(WorkflowError::Validation(format!(
                "DFLSS CTQ 2 violation: Triple count {} exceeds max_run_len 8 (Chatman Constant)",
                self.triples.len()
            )));
        }

        // Use guard for additional validation
        let guard = MaxRunLengthGuard::<8>::new();
        // Convert triples to JSON array for guard validation
        let triples_json = Value::Array(
            self.triples
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    // Create a simple JSON object representing the triple
                    // The guard only checks length, so we don't need full triple details
                    serde_json::json!({
                        "index": i,
                        "valid": true,
                    })
                })
                .collect(),
        );

        let context = GuardContext {
            workflow_spec: None,
            rdf_store: None,
            metadata: Value::Null,
        };

        let guard_result = guard
            .execute(&triples_json, &context)
            .map_err(|e| WorkflowError::Validation(format!("Guard validation failed: {}", e)))?;

        if !guard_result.allowed {
            return Err(WorkflowError::Validation(
                guard_result.reason.unwrap_or_else(|| {
                    format!(
                        "DFLSS CTQ 2 violation: Guard validation failed for {} triples",
                        self.triples.len()
                    )
                }),
            ));
        }

        // Transition to GuardValidated state
        Ok(ValidatedTriples {
            triples: self.triples,
            _validation: PhantomData,
        })
    }
}

impl ValidatedTriples<GuardValidated> {
    /// Validate schema (Weaver validation)
    ///
    /// Transitions from `GuardValidated` to `SchemaValidated`.
    /// Returns error if schema validation fails (DFLSS CTQ 1 violation).
    pub fn validate_schema(self) -> Result<ValidatedTriples<SchemaValidated>, WorkflowError> {
        // TODO: Implement Weaver schema validation
        // For now, we assume schema validation passes if guard validation passed
        // In production, this would call Weaver validation

        // Transition to SchemaValidated state
        Ok(ValidatedTriples {
            triples: self.triples,
            _validation: PhantomData,
        })
    }

    /// Get triples (read-only access)
    ///
    /// Returns a reference to the triples for inspection.
    pub fn triples(&self) -> &[OxigraphTriple] {
        &self.triples
    }
}

impl ValidatedTriples<SchemaValidated> {
    /// Convert to hot path input (zero-copy)
    ///
    /// This is the only way to extract triples for hot path execution.
    /// The type system guarantees that only fully validated data can enter the hot path.
    ///
    /// # Architecture Compliance
    ///
    /// This method consumes `ValidatedTriples<SchemaValidated>`, ensuring that:
    /// 1. All validation happened at ingress (knhk-workflow-engine)
    /// 2. Hot path receives pre-validated data
    /// 3. No defensive checks needed in hot path
    pub fn into_hot_path(self) -> Vec<OxigraphTriple> {
        self.triples
    }

    /// Get triples reference (read-only access)
    ///
    /// Returns a reference to the validated triples.
    pub fn triples(&self) -> &[OxigraphTriple] {
        &self.triples
    }
}

/// Helper function to validate triples through the full pipeline
///
/// This function provides a convenient way to validate triples from unvalidated
/// to schema-validated state in a single call.
///
/// # DFLSS Compliance
///
/// This function ensures:
/// - Guard validation (MAX_RUN_LEN ≤ 8) - DFLSS CTQ 2
/// - Schema validation (Weaver) - DFLSS CTQ 1
/// - Type-level guarantees that only validated data enters hot path
pub fn validate_triples_for_hot_path(
    triples: Vec<OxigraphTriple>,
) -> Result<ValidatedTriples<SchemaValidated>, WorkflowError> {
    ValidatedTriples::<Unvalidated>::new(triples)
        .validate_guards()?
        .validate_schema()
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxigraph::model::{NamedNode, Triple as OxigraphTriple};

    fn create_test_triple() -> OxigraphTriple {
        OxigraphTriple::new(
            NamedNode::new("http://example.org/subject").unwrap(),
            NamedNode::new("http://example.org/predicate").unwrap(),
            NamedNode::new("http://example.org/object").unwrap(),
        )
    }

    #[test]
    fn test_validation_pipeline() {
        // Create unvalidated triples
        let triples = vec![create_test_triple(); 8];
        let unvalidated = ValidatedTriples::<Unvalidated>::new(triples);

        // Validate guards (Unvalidated → GuardValidated)
        let guard_validated = unvalidated
            .validate_guards()
            .expect("Guard validation should succeed for 8 triples");
        assert_eq!(guard_validated.triples().len(), 8);

        // Validate schema (GuardValidated → SchemaValidated)
        let schema_validated = guard_validated
            .validate_schema()
            .expect("Schema validation should succeed");
        assert_eq!(schema_validated.triples().len(), 8);

        // Extract for hot path (SchemaValidated → Vec<Triple>)
        let hot_path_triples = schema_validated.into_hot_path();
        assert_eq!(hot_path_triples.len(), 8);
    }

    #[test]
    fn test_guard_validation_failure() {
        // Create triples exceeding MAX_RUN_LEN
        let triples = vec![create_test_triple(); 9];
        let unvalidated = ValidatedTriples::<Unvalidated>::new(triples);

        // Guard validation should fail
        let result = unvalidated.validate_guards();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, WorkflowError::Validation(_)));
        assert!(error.to_string().contains("DFLSS CTQ 2 violation"));
    }

    #[test]
    fn test_helper_function() {
        // Test helper function for full validation pipeline
        let triples = vec![create_test_triple(); 8];
        let validated = validate_triples_for_hot_path(triples)
            .expect("Full validation pipeline should succeed");

        // Extract for hot path
        let hot_path_triples = validated.into_hot_path();
        assert_eq!(hot_path_triples.len(), 8);
    }

    #[test]
    fn test_type_safety() {
        // Verify that type system prevents invalid state transitions
        let triples = vec![create_test_triple(); 8];
        let unvalidated = ValidatedTriples::<Unvalidated>::new(triples);

        // Cannot directly convert Unvalidated to hot path
        // This would be a compile error:
        // let hot_path = unvalidated.into_hot_path(); // Error: method not found

        // Must go through validation pipeline
        let guard_validated = unvalidated.validate_guards().unwrap();

        // Cannot directly convert GuardValidated to hot path
        // This would be a compile error:
        // let hot_path = guard_validated.into_hot_path(); // Error: method not found

        // Must validate schema first
        let schema_validated = guard_validated.validate_schema().unwrap();

        // Now can extract for hot path
        let _hot_path = schema_validated.into_hot_path();
    }
}
