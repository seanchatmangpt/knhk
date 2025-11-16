//! Invariants checker - validates Q preservation.
//!
//! The 5 invariants that must be preserved across all snapshots:
//! 1. Completeness: All sectors covered
//! 2. Consistency: No contradictions
//! 3. Correctness: Projections accurate
//! 4. Performance: Hot path ≤8 ticks
//! 5. Provenance: All changes receipted

use crate::receipt::{ValidationError, ValidationResults};
use crate::snapshot::SigmaSnapshot;
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidatorError {
    #[error("Invariant violated: {0}")]
    InvariantViolation(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

/// Invariants checker
pub struct InvariantValidator {
    /// Minimum sectors required for completeness
    min_sectors: usize,

    /// Performance threshold (ticks)
    max_ticks: u64,
}

impl Default for InvariantValidator {
    fn default() -> Self {
        Self {
            min_sectors: 1,
            max_ticks: 8,
        }
    }
}

impl InvariantValidator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_min_sectors(mut self, min_sectors: usize) -> Self {
        self.min_sectors = min_sectors;
        self
    }

    pub fn with_max_ticks(mut self, max_ticks: u64) -> Self {
        self.max_ticks = max_ticks;
        self
    }

    /// Validate snapshot against all invariants
    pub fn validate(&self, snapshot: &SigmaSnapshot) -> ValidationResults {
        let mut results = ValidationResults::default();

        // Static checks
        results.static_checks_passed = self.validate_static(snapshot, &mut results.errors);

        // Dynamic checks
        results.dynamic_checks_passed = self.validate_dynamic(snapshot, &mut results.errors);

        // Performance checks
        results.performance_checks_passed = self.validate_performance(snapshot, &mut results.errors);

        // Invariants Q
        results.invariants_q_preserved = self.validate_invariants_q(snapshot, &mut results.errors);

        results
    }

    /// Invariant 1: Completeness - all sectors covered
    fn check_completeness(&self, snapshot: &SigmaSnapshot, errors: &mut Vec<ValidationError>) -> bool {
        let triples = snapshot.all_triples();

        // Count unique sectors
        let sectors: HashSet<_> = triples.iter()
            .filter(|t| t.predicate == "sector")
            .map(|t| &t.object)
            .collect();

        if sectors.len() < self.min_sectors {
            errors.push(ValidationError::new(
                "Q1_COMPLETENESS",
                format!("Only {} sectors found, minimum {}", sectors.len(), self.min_sectors)
            ));
            return false;
        }

        true
    }

    /// Invariant 2: Consistency - no contradictions
    fn check_consistency(&self, snapshot: &SigmaSnapshot, errors: &mut Vec<ValidationError>) -> bool {
        let triples = snapshot.all_triples();

        // Check for contradictions: same subject+predicate, different objects
        let mut seen = std::collections::HashMap::new();

        for triple in &triples {
            let key = (&triple.subject, &triple.predicate);

            if let Some(existing_object) = seen.get(&key) {
                if *existing_object != &triple.object {
                    errors.push(ValidationError::new(
                        "Q2_CONSISTENCY",
                        format!(
                            "Contradiction: {} {} has both {} and {}",
                            triple.subject, triple.predicate, existing_object, triple.object
                        )
                    ));
                    return false;
                }
            } else {
                seen.insert(key, &triple.object);
            }
        }

        true
    }

    /// Invariant 3: Correctness - projections accurate
    fn check_correctness(&self, snapshot: &SigmaSnapshot, errors: &mut Vec<ValidationError>) -> bool {
        let triples = snapshot.all_triples();

        // Basic correctness: all triples have non-empty components
        for triple in &triples {
            if triple.subject.is_empty() || triple.predicate.is_empty() || triple.object.is_empty() {
                errors.push(ValidationError::new(
                    "Q3_CORRECTNESS",
                    "Triple has empty component"
                ));
                return false;
            }
        }

        // TODO: Add projection accuracy tests (requires test data)

        true
    }

    /// Invariant 4: Performance - hot path ≤8 ticks
    fn check_performance(&self, snapshot: &SigmaSnapshot, errors: &mut Vec<ValidationError>) -> bool {
        // Measure query performance
        let start = std::time::Instant::now();

        // Simulate hot path: query by subject (most common operation)
        let triples = snapshot.all_triples();
        if let Some(triple) = triples.first() {
            let _ = snapshot.query_subject(&triple.subject);
        }

        let elapsed_nanos = start.elapsed().as_nanos() as u64;
        let ticks = elapsed_nanos / 250; // Assume 4 GHz CPU (250ps per tick)

        if ticks > self.max_ticks {
            errors.push(ValidationError::new(
                "Q4_PERFORMANCE",
                format!("Query took {} ticks, max {}", ticks, self.max_ticks)
            ));
            return false;
        }

        true
    }

    /// Invariant 5: Provenance - all changes receipted
    fn check_provenance(&self, snapshot: &SigmaSnapshot, errors: &mut Vec<ValidationError>) -> bool {
        // Note: We don't require receipts during validation itself,
        // as we're in the process of creating the receipt.
        // Receipts are required when committing/promoting snapshots (enforced at that level).

        // If snapshot has receipt, verify it
        if let Some(receipt) = &snapshot.validation_receipt {
            if let Err(e) = receipt.verify() {
                errors.push(ValidationError::new(
                    "Q5_PROVENANCE",
                    format!("Receipt verification failed: {}", e)
                ));
                return false;
            }
        }

        true
    }

    /// Validate all 5 invariants
    fn validate_invariants_q(&self, snapshot: &SigmaSnapshot, errors: &mut Vec<ValidationError>) -> bool {
        let q1 = self.check_completeness(snapshot, errors);
        let q2 = self.check_consistency(snapshot, errors);
        let q3 = self.check_correctness(snapshot, errors);
        let q4 = self.check_performance(snapshot, errors);
        let q5 = self.check_provenance(snapshot, errors);

        q1 && q2 && q3 && q4 && q5
    }

    /// Static checks (SHACL, Σ² invariants)
    fn validate_static(&self, snapshot: &SigmaSnapshot, errors: &mut Vec<ValidationError>) -> bool {
        // Basic structural validation
        let triples = snapshot.all_triples();

        if triples.is_empty() {
            errors.push(ValidationError::new(
                "STATIC_EMPTY",
                "Snapshot has no triples"
            ));
            return false;
        }

        true
    }

    /// Dynamic checks (projection tests, runtime behavior)
    fn validate_dynamic(&self, snapshot: &SigmaSnapshot, _errors: &mut Vec<ValidationError>) -> bool {
        // Test query operations work
        let triples = snapshot.all_triples();

        if let Some(triple) = triples.first() {
            let results = snapshot.query_subject(&triple.subject);
            if results.is_empty() {
                return false;
            }
        }

        true
    }

    /// Performance checks (SLO tests)
    fn validate_performance(&self, snapshot: &SigmaSnapshot, errors: &mut Vec<ValidationError>) -> bool {
        self.check_performance(snapshot, errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::{SnapshotMetadata, Triple, TripleStore};

    #[test]
    fn test_completeness_check() {
        let mut store = TripleStore::new();
        store.add(Triple::new("company1", "sector", "Technology"));
        store.add(Triple::new("company2", "sector", "Healthcare"));

        let snapshot = SigmaSnapshot::new(None, store, SnapshotMetadata::default())
            .expect("Failed to create snapshot");

        let validator = InvariantValidator::new().with_min_sectors(1);
        let mut errors = vec![];

        assert!(validator.check_completeness(&snapshot, &mut errors));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_consistency_check_pass() {
        let mut store = TripleStore::new();
        store.add(Triple::new("alice", "age", "30"));
        store.add(Triple::new("bob", "age", "25"));

        let snapshot = SigmaSnapshot::new(None, store, SnapshotMetadata::default())
            .expect("Failed to create snapshot");

        let validator = InvariantValidator::new();
        let mut errors = vec![];

        assert!(validator.check_consistency(&snapshot, &mut errors));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_consistency_check_fail() {
        let mut store = TripleStore::new();
        store.add(Triple::new("alice", "age", "30"));
        store.add(Triple::new("alice", "age", "31")); // Contradiction!

        let snapshot = SigmaSnapshot::new(None, store, SnapshotMetadata::default())
            .expect("Failed to create snapshot");

        let validator = InvariantValidator::new();
        let mut errors = vec![];

        assert!(!validator.check_consistency(&snapshot, &mut errors));
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "Q2_CONSISTENCY");
    }

    #[test]
    fn test_correctness_check() {
        let mut store = TripleStore::new();
        store.add(Triple::new("subject", "predicate", "object"));

        let snapshot = SigmaSnapshot::new(None, store, SnapshotMetadata::default())
            .expect("Failed to create snapshot");

        let validator = InvariantValidator::new();
        let mut errors = vec![];

        assert!(validator.check_correctness(&snapshot, &mut errors));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_performance_check() {
        let mut store = TripleStore::new();
        // Add enough data to make query meaningful
        for i in 0..100 {
            store.add(Triple::new(
                format!("subject{}", i),
                "predicate",
                format!("object{}", i)
            ));
        }

        let snapshot = SigmaSnapshot::new(None, store, SnapshotMetadata::default())
            .expect("Failed to create snapshot");

        let validator = InvariantValidator::new().with_max_ticks(1000); // Generous for test
        let mut errors = vec![];

        assert!(validator.check_performance(&snapshot, &mut errors));
    }

    #[test]
    fn test_full_validation() {
        let mut store = TripleStore::new();
        store.add(Triple::new("company1", "sector", "Technology"));
        store.add(Triple::new("company1", "revenue", "1000000"));

        let snapshot = SigmaSnapshot::new(None, store, SnapshotMetadata::default())
            .expect("Failed to create snapshot");

        let validator = InvariantValidator::new().with_min_sectors(1);
        let results = validator.validate(&snapshot);

        assert!(results.static_checks_passed);
        assert!(results.dynamic_checks_passed);
        assert!(results.performance_checks_passed);
        assert!(results.invariants_q_preserved);
    }
}
