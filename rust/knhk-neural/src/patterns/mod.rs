//! Pattern Discovery and Recognition Module
//!
//! Implements unsupervised learning algorithms to discover workflow patterns
//! from execution traces. Supports clustering, anomaly detection, and
//! pattern recommendation for optimizing workflow executions.
//!
//! DOCTRINE ALIGNMENT:
//! - Principle: O (Observation) - Learn from all execution observations
//! - Principle: Σ (Ontology) - Discovered patterns extend the ontology
//! - Covenant 6: All observations feed the learning pipeline

pub mod discovery;

pub use discovery::{
    ClusterResult, DiscoveryConfig, ExecutionTrace, PatternCluster, PatternDiscovery,
    PatternFeatures, TaskPrediction,
};

use serde::{Deserialize, Serialize};

/// Workflow pattern identified through unsupervised learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPattern {
    /// Unique pattern identifier
    pub id: String,

    /// Pattern type (sequential, parallel, choice, loop, etc.)
    pub pattern_type: PatternType,

    /// Frequency of occurrence in traces
    pub frequency: usize,

    /// Average execution time (ms)
    pub avg_execution_time: f32,

    /// Average resource usage (0-100%)
    pub avg_resource_usage: f32,

    /// Success rate (0.0-1.0)
    pub success_rate: f32,

    /// Pattern confidence score
    pub confidence: f32,

    /// Feature vector for similarity matching
    pub features: Vec<f32>,
}

/// Types of workflow patterns (based on YAWL/van der Aalst patterns)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternType {
    /// Sequential execution
    Sequence,
    /// Parallel split (AND-split)
    ParallelSplit,
    /// Synchronization (AND-join)
    Synchronization,
    /// Exclusive choice (XOR-split)
    ExclusiveChoice,
    /// Simple merge (XOR-join)
    SimpleMerge,
    /// Multi-choice (OR-split)
    MultiChoice,
    /// Multi-merge (OR-join)
    MultiMerge,
    /// Loop pattern
    Loop,
    /// Deferred choice
    DeferredChoice,
    /// Interleaved parallel routing
    InterleavedParallel,
    /// Unknown/composite pattern
    Unknown,
}

impl WorkflowPattern {
    /// Create a new pattern from features
    pub fn new(
        id: String,
        pattern_type: PatternType,
        features: Vec<f32>,
    ) -> Self {
        Self {
            id,
            pattern_type,
            frequency: 0,
            avg_execution_time: 0.0,
            avg_resource_usage: 0.0,
            success_rate: 1.0,
            confidence: 0.0,
            features,
        }
    }

    /// Update pattern statistics with new observation
    pub fn update_stats(
        &mut self,
        execution_time: f32,
        resource_usage: f32,
        success: bool,
    ) {
        let n = self.frequency as f32;

        // Incremental average updates
        self.avg_execution_time =
            (self.avg_execution_time * n + execution_time) / (n + 1.0);
        self.avg_resource_usage =
            (self.avg_resource_usage * n + resource_usage) / (n + 1.0);

        // Update success rate
        let success_value = if success { 1.0 } else { 0.0 };
        self.success_rate =
            (self.success_rate * n + success_value) / (n + 1.0);

        self.frequency += 1;

        // Update confidence based on frequency
        self.confidence = (self.frequency as f32).ln() / 10.0;
        self.confidence = self.confidence.min(1.0);
    }

    /// Calculate similarity to another pattern (cosine similarity)
    pub fn similarity(&self, other: &Self) -> f32 {
        if self.features.len() != other.features.len() {
            return 0.0;
        }

        let dot_product: f32 = self.features.iter()
            .zip(&other.features)
            .map(|(a, b)| a * b)
            .sum();

        let norm_a: f32 = self.features.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = other.features.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            (dot_product / (norm_a * norm_b)).max(0.0).min(1.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_creation() {
        let pattern = WorkflowPattern::new(
            "pat_001".to_string(),
            PatternType::Sequence,
            vec![1.0, 0.0, 0.5],
        );

        assert_eq!(pattern.id, "pat_001");
        assert_eq!(pattern.pattern_type, PatternType::Sequence);
        assert_eq!(pattern.frequency, 0);
    }

    #[test]
    fn test_pattern_stats_update() {
        let mut pattern = WorkflowPattern::new(
            "pat_002".to_string(),
            PatternType::ParallelSplit,
            vec![1.0, 1.0],
        );

        pattern.update_stats(100.0, 50.0, true);
        assert_eq!(pattern.frequency, 1);
        assert_eq!(pattern.avg_execution_time, 100.0);
        assert_eq!(pattern.avg_resource_usage, 50.0);
        assert_eq!(pattern.success_rate, 1.0);

        pattern.update_stats(200.0, 60.0, false);
        assert_eq!(pattern.frequency, 2);
        assert_eq!(pattern.avg_execution_time, 150.0);
        assert_eq!(pattern.avg_resource_usage, 55.0);
        assert_eq!(pattern.success_rate, 0.5);
    }

    #[test]
    fn test_pattern_similarity() {
        let pattern1 = WorkflowPattern::new(
            "pat_001".to_string(),
            PatternType::Sequence,
            vec![1.0, 0.0, 0.0],
        );

        let pattern2 = WorkflowPattern::new(
            "pat_002".to_string(),
            PatternType::Sequence,
            vec![1.0, 0.0, 0.0],
        );

        let pattern3 = WorkflowPattern::new(
            "pat_003".to_string(),
            PatternType::ParallelSplit,
            vec![0.0, 1.0, 0.0],
        );

        // Identical patterns should have similarity 1.0
        assert!((pattern1.similarity(&pattern2) - 1.0).abs() < 0.001);

        // Orthogonal patterns should have similarity 0.0
        assert!((pattern1.similarity(&pattern3) - 0.0).abs() < 0.001);
    }
}
