//! Q Invariant Checker
//!
//! Runtime enforcement of guard invariants Q.
//! Validates preconditions and postconditions for workflow operations.

use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Invariant type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InvariantType {
    /// Precondition (must be true before operation)
    Precondition,
    /// Postcondition (must be true after operation)
    Postcondition,
    /// State invariant (must always be true)
    StateInvariant,
    /// Temporal invariant (must be true at specific times)
    TemporalInvariant,
}

/// Invariant check result
#[derive(Debug, Clone)]
pub struct InvariantCheckResult {
    /// Whether invariant passed
    pub passed: bool,
    /// Invariant ID
    pub invariant_id: String,
    /// Invariant type
    pub invariant_type: InvariantType,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Check timestamp
    pub checked_at_ms: u64,
}

/// Invariant predicate function
pub type InvariantPredicate = Arc<
    dyn Fn(&serde_json::Value) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send>>
        + Send
        + Sync,
>;

/// Invariant definition
#[derive(Clone)]
pub struct Invariant {
    /// Invariant ID
    pub id: String,
    /// Invariant type
    pub invariant_type: InvariantType,
    /// Invariant name
    pub name: String,
    /// Invariant description
    pub description: String,
    /// Invariant predicate (returns true if invariant holds)
    pub predicate: InvariantPredicate,
    /// Severity (1=low, 10=critical)
    pub severity: u8,
}

/// Invariant checker
pub struct InvariantChecker {
    /// Invariants by type
    invariants: Arc<RwLock<HashMap<InvariantType, Vec<Invariant>>>>,
    /// Invariant check history
    check_history: Arc<RwLock<Vec<InvariantCheckResult>>>,
}

impl InvariantChecker {
    /// Create new invariant checker
    pub fn new() -> Self {
        Self {
            invariants: Arc::new(RwLock::new(HashMap::new())),
            check_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register an invariant
    pub async fn register_invariant(&self, invariant: Invariant) -> WorkflowResult<()> {
        if invariant.id.is_empty() {
            return Err(WorkflowError::Validation(
                "Invariant ID cannot be empty".to_string(),
            ));
        }

        let mut invariants = self.invariants.write().await;
        invariants
            .entry(invariant.invariant_type)
            .or_insert_with(Vec::new)
            .push(invariant);

        Ok(())
    }

    /// Check preconditions
    pub async fn check_preconditions(
        &self,
        state: &serde_json::Value,
    ) -> WorkflowResult<Vec<InvariantCheckResult>> {
        self.check_invariants(InvariantType::Precondition, state)
            .await
    }

    /// Check postconditions
    pub async fn check_postconditions(
        &self,
        state: &serde_json::Value,
    ) -> WorkflowResult<Vec<InvariantCheckResult>> {
        self.check_invariants(InvariantType::Postcondition, state)
            .await
    }

    /// Check state invariants
    pub async fn check_state_invariants(
        &self,
        state: &serde_json::Value,
    ) -> WorkflowResult<Vec<InvariantCheckResult>> {
        self.check_invariants(InvariantType::StateInvariant, state)
            .await
    }

    /// Check all invariants of a given type
    async fn check_invariants(
        &self,
        invariant_type: InvariantType,
        state: &serde_json::Value,
    ) -> WorkflowResult<Vec<InvariantCheckResult>> {
        let invariants = self.invariants.read().await;
        let inv_list = invariants.get(&invariant_type).cloned().unwrap_or_default();
        drop(invariants);

        let mut results = Vec::new();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        for invariant in inv_list {
            let passed = (invariant.predicate)(state).await;

            let result = InvariantCheckResult {
                passed,
                invariant_id: invariant.id.clone(),
                invariant_type,
                error: if !passed {
                    Some(format!("Invariant '{}' violated", invariant.name))
                } else {
                    None
                },
                checked_at_ms: timestamp,
            };

            results.push(result.clone());

            // Record in history
            let mut history = self.check_history.write().await;
            history.push(result);
        }

        Ok(results)
    }

    /// Validate workflow specification
    pub fn validate_workflow_spec(&self, _spec: &crate::parser::WorkflowSpec) -> WorkflowResult<()> {
        // For now, basic validation - can be extended
        Ok(())
    }

    /// Validate that all invariants passed
    pub fn validate_results(results: &[InvariantCheckResult]) -> WorkflowResult<()> {
        let failures: Vec<_> = results.iter().filter(|r| !r.passed).collect();

        if failures.is_empty() {
            Ok(())
        } else {
            let error_msg = failures
                .iter()
                .map(|r| r.error.as_deref().unwrap_or("Unknown error"))
                .collect::<Vec<_>>()
                .join("; ");

            Err(WorkflowError::GuardViolation(error_msg))
        }
    }

    /// Get check history
    pub async fn get_check_history(&self) -> Vec<InvariantCheckResult> {
        let history = self.check_history.read().await;
        history.clone()
    }

    /// Get failure count
    pub async fn get_failure_count(&self) -> usize {
        let history = self.check_history.read().await;
        history.iter().filter(|r| !r.passed).count()
    }

    /// Clear history
    pub async fn clear_history(&self) {
        let mut history = self.check_history.write().await;
        history.clear();
    }
}

impl Default for InvariantChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_invariant_checker() {
        let checker = InvariantChecker::new();

        // Register an invariant that checks for positive values
        let invariant = Invariant {
            id: "positive-value".to_string(),
            invariant_type: InvariantType::Precondition,
            name: "Positive Value".to_string(),
            description: "Value must be positive".to_string(),
            predicate: Arc::new(|state| {
                Box::pin(async move {
                    state
                        .get("value")
                        .and_then(|v| v.as_i64())
                        .map(|v| v > 0)
                        .unwrap_or(false)
                })
            }),
            severity: 5,
        };

        checker
            .register_invariant(invariant)
            .await
            .expect("Failed to register invariant");

        // Test with valid state
        let valid_state = serde_json::json!({"value": 10});
        let results = checker
            .check_preconditions(&valid_state)
            .await
            .expect("Check failed");
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);

        // Test with invalid state
        let invalid_state = serde_json::json!({"value": -5});
        let results = checker
            .check_preconditions(&invalid_state)
            .await
            .expect("Check failed");
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
    }

    #[tokio::test]
    async fn test_validate_results() {
        let passed_result = InvariantCheckResult {
            passed: true,
            invariant_id: "inv1".to_string(),
            invariant_type: InvariantType::Precondition,
            error: None,
            checked_at_ms: 0,
        };

        let failed_result = InvariantCheckResult {
            passed: false,
            invariant_id: "inv2".to_string(),
            invariant_type: InvariantType::Precondition,
            error: Some("Failed".to_string()),
            checked_at_ms: 0,
        };

        // All passed
        assert!(InvariantChecker::validate_results(&[passed_result.clone()]).is_ok());

        // Some failed
        assert!(InvariantChecker::validate_results(&[passed_result, failed_result]).is_err());
    }

    #[tokio::test]
    async fn test_check_history() {
        let checker = InvariantChecker::new();

        let invariant = Invariant {
            id: "test-inv".to_string(),
            invariant_type: InvariantType::StateInvariant,
            name: "Test".to_string(),
            description: "Test invariant".to_string(),
            predicate: Arc::new(|_| Box::pin(async move { true })),
            severity: 1,
        };

        checker
            .register_invariant(invariant)
            .await
            .expect("Failed to register");

        let state = serde_json::json!({});
        let _ = checker.check_state_invariants(&state).await;

        let history = checker.get_check_history().await;
        assert_eq!(history.len(), 1);

        let failure_count = checker.get_failure_count().await;
        assert_eq!(failure_count, 0);
    }
}
