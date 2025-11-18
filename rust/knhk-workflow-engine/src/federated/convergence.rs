//! KL divergence-based convergence validation

use super::traits::ConvergenceValidator;
use super::types::{ConvergenceStatus, FederatedError};
use tracing::instrument;

/// KL divergence-based convergence validator
///
/// # Convergence Criteria
///
/// 1. KL divergence < threshold (default: 0.01)
/// 2. Minimum rounds completed (default: 10)
///
/// # KL Divergence Approximation
///
/// For parameter vectors p and q, we approximate:
/// ```text
/// KL(P || Q) ≈ (1/2) × ||p - q||² / σ²
/// ```
///
/// Where σ² is the variance (assumed to be 1.0 for simplicity).
///
/// # Example
///
/// ```rust
/// use knhk_workflow_engine::federated::*;
///
/// let validator = KLConvergenceValidator::new();
///
/// let old_params = vec![0.0; 1000];
/// let new_params = vec![0.001; 1000]; // Small change
///
/// let status = validator.check_convergence(&old_params, &new_params, 15).unwrap();
///
/// match status {
///     ConvergenceStatus::Converged { kl_divergence, rounds_completed } => {
///         println!("Converged! KL: {}, Rounds: {}", kl_divergence, rounds_completed);
///     }
///     ConvergenceStatus::Training { .. } => {
///         println!("Still training...");
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct KLConvergenceValidator {
    /// KL divergence threshold for convergence
    kl_threshold: f64,
    /// Minimum rounds before declaring convergence
    min_rounds: u64,
}

impl KLConvergenceValidator {
    /// Create a new validator with default settings
    ///
    /// # Defaults
    /// - `kl_threshold`: 0.01 (1% divergence)
    /// - `min_rounds`: 10
    pub fn new() -> Self {
        Self {
            kl_threshold: 0.01,
            min_rounds: 10,
        }
    }

    /// Create a new validator with custom settings
    pub fn with_config(kl_threshold: f64, min_rounds: u64) -> Self {
        Self {
            kl_threshold,
            min_rounds,
        }
    }

    /// Compute KL divergence between parameter distributions
    ///
    /// # Formula
    ///
    /// ```text
    /// KL(P || Q) = Σ P(x) log(P(x) / Q(x))
    /// ```
    ///
    /// For parameter vectors, we approximate as:
    /// ```text
    /// KL ≈ (1/2) × ||p - q||² / σ²
    /// ```
    ///
    /// # Arguments
    ///
    /// - `old_params`: Previous model parameters
    /// - `new_params`: Current model parameters
    ///
    /// # Returns
    ///
    /// KL divergence value (lower is more similar)
    pub fn compute_kl_divergence(&self, old_params: &[f32], new_params: &[f32]) -> f64 {
        assert_eq!(
            old_params.len(),
            new_params.len(),
            "Parameter vectors must have same length"
        );

        if old_params.is_empty() {
            return 0.0;
        }

        // Compute L2 squared distance
        let squared_distance: f32 = old_params
            .iter()
            .zip(new_params.iter())
            .map(|(old, new)| (old - new).powi(2))
            .sum();

        // Variance approximation (σ² = 1.0)
        let variance = 1.0;

        // KL approximation: (1/2) × ||p - q||² / σ²
        (0.5 * squared_distance / variance) as f64
    }
}

impl Default for KLConvergenceValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ConvergenceValidator for KLConvergenceValidator {
    #[instrument(
        name = "federated.learning.convergence_check",
        skip(self, old_params, new_params),
        fields(
            round = round,
            param_dim = old_params.len(),
        )
    )]
    fn check_convergence(
        &self,
        old_params: &[f32],
        new_params: &[f32],
        round: u64,
    ) -> Result<ConvergenceStatus, FederatedError> {
        // Compute KL divergence
        let kl = self.compute_kl_divergence(old_params, new_params);

        tracing::debug!(
            round = round,
            kl_divergence = kl,
            threshold = self.kl_threshold,
            min_rounds = self.min_rounds,
            "Convergence check"
        );

        // Check convergence criteria
        if round >= self.min_rounds && kl < self.kl_threshold {
            tracing::info!(
                round = round,
                kl_divergence = kl,
                "Model converged!"
            );

            Ok(ConvergenceStatus::Converged {
                kl_divergence: kl,
                rounds_completed: round,
            })
        } else {
            // Estimate remaining rounds (linear extrapolation)
            let estimated_remaining = if kl > 0.0 && kl > self.kl_threshold {
                let progress_rate = kl / round as f64;
                ((self.kl_threshold / progress_rate) as u64).saturating_sub(round)
            } else {
                0
            };

            tracing::debug!(
                round = round,
                kl_divergence = kl,
                estimated_remaining = estimated_remaining,
                "Still training"
            );

            Ok(ConvergenceStatus::Training {
                kl_divergence: kl,
                rounds_completed: round,
                estimated_rounds_remaining: estimated_remaining,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kl_divergence_identical_params() {
        let validator = KLConvergenceValidator::new();

        let params = vec![0.5; 1000];
        let kl = validator.compute_kl_divergence(&params, &params);

        assert_eq!(kl, 0.0, "Identical params should have KL = 0");
    }

    #[test]
    fn test_kl_divergence_small_change() {
        let validator = KLConvergenceValidator::new();

        let old_params = vec![0.5; 1000];
        let new_params = vec![0.501; 1000]; // 0.2% change

        let kl = validator.compute_kl_divergence(&old_params, &new_params);

        assert!(kl > 0.0 && kl < 0.1, "Small change should have small KL");
    }

    #[test]
    fn test_convergence_with_sufficient_rounds() {
        let validator = KLConvergenceValidator::new();

        let old_params = vec![0.5; 1000];
        let new_params = vec![0.5001; 1000]; // Very small change

        let status = validator
            .check_convergence(&old_params, &new_params, 15)
            .unwrap();

        match status {
            ConvergenceStatus::Converged {
                kl_divergence,
                rounds_completed,
            } => {
                assert!(kl_divergence < 0.01);
                assert_eq!(rounds_completed, 15);
            }
            _ => panic!("Expected convergence"),
        }
    }

    #[test]
    fn test_no_convergence_insufficient_rounds() {
        let validator = KLConvergenceValidator::new();

        let old_params = vec![0.5; 1000];
        let new_params = vec![0.5001; 1000]; // Very small change

        // Only 5 rounds, need 10 minimum
        let status = validator
            .check_convergence(&old_params, &new_params, 5)
            .unwrap();

        match status {
            ConvergenceStatus::Training { .. } => {
                // Expected
            }
            _ => panic!("Should not converge with <10 rounds"),
        }
    }

    #[test]
    fn test_no_convergence_large_kl() {
        let validator = KLConvergenceValidator::new();

        let old_params = vec![0.0; 1000];
        let new_params = vec![1.0; 1000]; // Large change

        let status = validator
            .check_convergence(&old_params, &new_params, 15)
            .unwrap();

        match status {
            ConvergenceStatus::Training { kl_divergence, .. } => {
                assert!(kl_divergence > 0.01, "Should have large KL divergence");
            }
            _ => panic!("Should not converge with large KL"),
        }
    }
}
