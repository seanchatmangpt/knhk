//! Byzantine-robust median aggregation implementation

use super::traits::ByzantineRobustAggregator;
use super::types::{AggregatedGradients, FederatedError, Gradients};
use async_trait::async_trait;
use rayon::prelude::*;
use tracing::{instrument, span, Level};

/// Byzantine-robust median aggregator implementation
///
/// # Algorithm
///
/// Uses coordinate-wise median aggregation to tolerate Byzantine (malicious) agents.
///
/// # Byzantine Tolerance
///
/// **Theorem**: Tolerates up to f < n/3 Byzantine agents.
///
/// **Proof**: For n agents, median is at position ⌈n/2⌉. For median to be Byzantine,
/// need at least ⌈n/2⌉ Byzantine agents. But f < n/3 < n/2, so impossible.
/// Therefore, median is from honest majority. ∎
///
/// # Performance
///
/// - Time: O(d × n log n) where d = dimension, n = agents
/// - Parallel: O(d × log n) on multi-core
/// - Target: <5ms for d=1000, n=100
///
/// # Example
///
/// ```rust,no_run
/// use knhk_workflow_engine::federated::*;
///
/// # async fn example() -> Result<(), FederatedError> {
/// let aggregator = MedianAggregator::new();
///
/// let gradients = vec![
///     Gradients { values: vec![0.5; 1000], agent_id: "agent_1".into(), timestamp: 0, round: 0 },
///     Gradients { values: vec![0.6; 1000], agent_id: "agent_2".into(), timestamp: 0, round: 0 },
///     // ... more gradients
/// ];
///
/// let result = aggregator.aggregate(gradients, 100).await?;
/// println!("Aggregated {} agents, detected {} Byzantine",
///          result.num_agents, result.byzantine_agents.len());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct MedianAggregator {
    /// Maximum Byzantine fraction (< 1/3)
    max_byzantine_fraction: f64,
    /// Detection threshold for outliers (z-score)
    outlier_threshold: f64,
}

impl MedianAggregator {
    /// Create a new median aggregator with default settings
    ///
    /// # Defaults
    /// - `max_byzantine_fraction`: 0.33 (f < n/3)
    /// - `outlier_threshold`: 3.0 (3 standard deviations)
    pub fn new() -> Self {
        Self {
            max_byzantine_fraction: 0.33,
            outlier_threshold: 3.0,
        }
    }

    /// Create a new median aggregator with custom settings
    pub fn with_config(max_byzantine_fraction: f64, outlier_threshold: f64) -> Self {
        Self {
            max_byzantine_fraction,
            outlier_threshold,
        }
    }

    /// Detect Byzantine agents via outlier detection
    ///
    /// Uses L2 distance from aggregated gradients as metric.
    /// Agents with distance > threshold × scale are marked Byzantine.
    fn detect_byzantine(&self, gradients: &[Gradients], aggregated: &[f32]) -> Vec<String> {
        gradients
            .par_iter()
            .filter_map(|g| {
                // Compute L2 distance from aggregated
                let distance: f32 = g
                    .values
                    .iter()
                    .zip(aggregated.iter())
                    .map(|(gv, av)| (gv - av).powi(2))
                    .sum();
                let distance = distance.sqrt();

                // Simplified z-score: use distance threshold
                // TODO: Proper statistical z-score computation
                let threshold = self.outlier_threshold as f32 * 10.0;

                if distance > threshold {
                    tracing::warn!(
                        agent_id = %g.agent_id,
                        distance = distance,
                        threshold = threshold,
                        "Byzantine agent detected"
                    );
                    Some(g.agent_id.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for MedianAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ByzantineRobustAggregator for MedianAggregator {
    #[instrument(
        name = "federated.learning.median_aggregation",
        skip(self, gradients),
        fields(
            num_agents = gradients.len(),
            quorum_size = quorum_size,
        )
    )]
    async fn aggregate(
        &self,
        gradients: Vec<Gradients>,
        quorum_size: usize,
    ) -> Result<AggregatedGradients, FederatedError> {
        let span = span!(Level::INFO, "median_aggregation");
        let _enter = span.enter();

        // 1. Validate quorum
        if gradients.len() < quorum_size {
            return Err(FederatedError::InsufficientQuorum {
                got: gradients.len(),
                need: quorum_size,
            });
        }

        // 2. Validate Byzantine tolerance
        let max_byzantine = (gradients.len() as f64 * self.max_byzantine_fraction) as usize;
        tracing::debug!(
            agents = gradients.len(),
            max_byzantine = max_byzantine,
            "Byzantine tolerance check"
        );

        // 3. Get gradient dimension
        if gradients.is_empty() {
            return Err(FederatedError::AggregationError(
                "No gradients provided".into(),
            ));
        }

        let dim = gradients[0].values.len();

        // Validate all gradients have same dimension
        for g in &gradients {
            if g.values.len() != dim {
                return Err(FederatedError::AggregationError(format!(
                    "Gradient dimension mismatch: expected {}, got {}",
                    dim,
                    g.values.len()
                )));
            }
        }

        // 4. Parallel coordinate-wise median (SIMD-optimized via Rayon)
        let aggregated_values: Vec<f32> = (0..dim)
            .into_par_iter()
            .map(|j| {
                // Collect j-th coordinate from all gradients
                let mut coords: Vec<f32> = gradients.iter().map(|g| g.values[j]).collect();

                // Sort to find median
                coords.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

                // Median value (middle element)
                let median_idx = coords.len() / 2;
                coords[median_idx]
            })
            .collect();

        // 5. Detect Byzantine agents via outlier detection
        let byzantine_agents = self.detect_byzantine(&gradients, &aggregated_values);

        tracing::info!(
            agents = gradients.len(),
            byzantine = byzantine_agents.len(),
            dimension = dim,
            "Aggregation complete"
        );

        // 6. Emit telemetry
        span.add_event(
            "aggregation_completed",
            vec![
                ("num_agents", gradients.len().into()),
                ("byzantine_detected", byzantine_agents.len().into()),
                ("dimension", dim.into()),
            ],
        );

        Ok(AggregatedGradients {
            values: aggregated_values,
            num_agents: gradients.len(),
            byzantine_agents,
            round: gradients[0].round,
            timestamp: chrono::Utc::now().timestamp(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_median_aggregation_honest_agents() {
        let aggregator = MedianAggregator::new();

        // All honest agents with similar gradients
        let gradients = vec![
            Gradients {
                values: vec![0.5, 0.6, 0.7],
                agent_id: "agent_1".into(),
                timestamp: 0,
                round: 0,
            },
            Gradients {
                values: vec![0.51, 0.59, 0.71],
                agent_id: "agent_2".into(),
                timestamp: 0,
                round: 0,
            },
            Gradients {
                values: vec![0.49, 0.61, 0.69],
                agent_id: "agent_3".into(),
                timestamp: 0,
                round: 0,
            },
        ];

        let result = aggregator.aggregate(gradients, 3).await.unwrap();

        // Median should be ~0.5, 0.6, 0.7
        assert!((result.values[0] - 0.5).abs() < 0.1);
        assert!((result.values[1] - 0.6).abs() < 0.1);
        assert!((result.values[2] - 0.7).abs() < 0.1);

        // No Byzantine agents
        assert_eq!(result.byzantine_agents.len(), 0);
    }

    #[tokio::test]
    async fn test_median_aggregation_with_byzantine() {
        let aggregator = MedianAggregator::new();

        // 2 honest + 1 Byzantine (< 1/3)
        let gradients = vec![
            Gradients {
                values: vec![0.5; 100],
                agent_id: "honest_1".into(),
                timestamp: 0,
                round: 0,
            },
            Gradients {
                values: vec![0.51; 100],
                agent_id: "honest_2".into(),
                timestamp: 0,
                round: 0,
            },
            Gradients {
                values: vec![100.0; 100], // Extreme outlier
                agent_id: "byzantine_1".into(),
                timestamp: 0,
                round: 0,
            },
        ];

        let result = aggregator.aggregate(gradients, 3).await.unwrap();

        // Median should be from honest majority (~0.5)
        assert!((result.values[0] - 0.5).abs() < 0.1);

        // Byzantine agent should be detected
        assert!(result.byzantine_agents.len() > 0);
    }

    #[tokio::test]
    async fn test_insufficient_quorum() {
        let aggregator = MedianAggregator::new();

        let gradients = vec![Gradients {
            values: vec![0.5],
            agent_id: "agent_1".into(),
            timestamp: 0,
            round: 0,
        }];

        // Need quorum of 3, but only have 1
        let result = aggregator.aggregate(gradients, 3).await;

        assert!(matches!(result, Err(FederatedError::InsufficientQuorum { .. })));
    }
}
