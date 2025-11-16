// rust/knhk-workflow-engine/src/neural/features.rs
//! Feature Engineering Pipeline for Workflow Embeddings
//!
//! Extracts features from workflow definitions for ML prediction:
//! - Workflow structure (graph topology)
//! - Data size and complexity
//! - Task dependencies
//! - Historical execution patterns
//! - Resource requirements

use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Workflow features for ML models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowFeatures {
    /// Pattern embedding (64-dimensional)
    pub pattern_embedding: Vec<f64>,
    /// Data size in bytes
    pub data_size_bytes: f64,
    /// Number of tasks in workflow
    pub num_tasks: f64,
    /// Number of dependencies
    pub num_dependencies: f64,
    /// Graph depth (longest path)
    pub graph_depth: f64,
    /// Graph width (max parallel tasks)
    pub graph_width: f64,
    /// Average task complexity score
    pub avg_task_complexity: f64,
}

impl WorkflowFeatures {
    /// Create default features
    pub fn default_with_embedding(embedding: Vec<f64>) -> Self {
        Self {
            pattern_embedding: embedding,
            data_size_bytes: 0.0,
            num_tasks: 1.0,
            num_dependencies: 0.0,
            graph_depth: 1.0,
            graph_width: 1.0,
            avg_task_complexity: 0.5,
        }
    }

    /// Convert features to flat vector for ML model input
    pub fn to_vector(&self) -> Vec<f64> {
        let mut vec = self.pattern_embedding.clone();
        vec.push(self.data_size_bytes);
        vec.push(self.num_tasks);
        vec.push(self.num_dependencies);
        vec.push(self.graph_depth);
        vec.push(self.graph_width);
        vec.push(self.avg_task_complexity);
        vec
    }

    /// Create from flat vector
    pub fn from_vector(vec: Vec<f64>) -> WorkflowResult<Self> {
        if vec.len() < 64 + 6 {
            return Err(WorkflowError::Configuration(
                format!("Invalid feature vector size: expected >= 70, got {}", vec.len())
            ));
        }

        Ok(Self {
            pattern_embedding: vec[0..64].to_vec(),
            data_size_bytes: vec[64],
            num_tasks: vec[65],
            num_dependencies: vec[66],
            graph_depth: vec[67],
            graph_width: vec[68],
            avg_task_complexity: vec[69],
        })
    }
}

/// Feature extractor for workflows
pub struct FeatureExtractor {
    /// Pattern embeddings cache (pattern_name -> embedding)
    pattern_cache: HashMap<String, Vec<f64>>,
    /// Task complexity estimator
    complexity_weights: HashMap<String, f64>,
}

impl FeatureExtractor {
    /// Create new feature extractor
    pub fn new() -> Self {
        Self {
            pattern_cache: Self::initialize_pattern_embeddings(),
            complexity_weights: Self::initialize_complexity_weights(),
        }
    }

    /// Initialize pattern embeddings (pre-trained or random)
    fn initialize_pattern_embeddings() -> HashMap<String, Vec<f64>> {
        let mut embeddings = HashMap::new();

        // Common YAWL patterns with random embeddings
        // In production, these would be learned from training data
        embeddings.insert("sequence".to_string(), Self::random_embedding(64));
        embeddings.insert("parallel_split".to_string(), Self::random_embedding(64));
        embeddings.insert("synchronization".to_string(), Self::random_embedding(64));
        embeddings.insert("exclusive_choice".to_string(), Self::random_embedding(64));
        embeddings.insert("simple_merge".to_string(), Self::random_embedding(64));
        embeddings.insert("multi_choice".to_string(), Self::random_embedding(64));
        embeddings.insert("multi_merge".to_string(), Self::random_embedding(64));

        embeddings
    }

    /// Initialize task complexity weights
    fn initialize_complexity_weights() -> HashMap<String, f64> {
        let mut weights = HashMap::new();

        weights.insert("compute".to_string(), 0.8);
        weights.insert("io".to_string(), 0.6);
        weights.insert("network".to_string(), 0.7);
        weights.insert("database".to_string(), 0.75);
        weights.insert("default".to_string(), 0.5);

        weights
    }

    /// Generate random embedding (placeholder)
    fn random_embedding(dim: usize) -> Vec<f64> {
        (0..dim).map(|_| fastrand::f64() * 2.0 - 1.0).collect()
    }

    /// Extract features from workflow definition
    pub fn extract(&self, workflow_def: &WorkflowDefinition) -> WorkflowResult<WorkflowFeatures> {
        // Get pattern embedding
        let pattern_embedding = self.get_pattern_embedding(&workflow_def.pattern_type);

        // Calculate graph metrics
        let (depth, width) = self.calculate_graph_metrics(&workflow_def.tasks);

        // Calculate average task complexity
        let avg_complexity = self.calculate_avg_complexity(&workflow_def.tasks);

        Ok(WorkflowFeatures {
            pattern_embedding,
            data_size_bytes: workflow_def.data_size_bytes as f64,
            num_tasks: workflow_def.tasks.len() as f64,
            num_dependencies: workflow_def.count_dependencies() as f64,
            graph_depth: depth,
            graph_width: width,
            avg_task_complexity: avg_complexity,
        })
    }

    /// Get pattern embedding from cache or generate new
    fn get_pattern_embedding(&self, pattern_type: &str) -> Vec<f64> {
        self.pattern_cache
            .get(pattern_type)
            .cloned()
            .unwrap_or_else(|| {
                tracing::debug!(pattern_type, "Unknown pattern, using random embedding");
                Self::random_embedding(64)
            })
    }

    /// Calculate graph depth and width
    fn calculate_graph_metrics(&self, tasks: &[Task]) -> (f64, f64) {
        if tasks.is_empty() {
            return (0.0, 0.0);
        }

        // Simplified: assume sequential for now
        // In production, this would analyze the full DAG
        let depth = tasks.len() as f64;
        let width = 1.0; // Placeholder

        (depth, width)
    }

    /// Calculate average task complexity
    fn calculate_avg_complexity(&self, tasks: &[Task]) -> f64 {
        if tasks.is_empty() {
            return 0.5;
        }

        let total_complexity: f64 = tasks
            .iter()
            .map(|task| {
                self.complexity_weights
                    .get(&task.task_type)
                    .copied()
                    .unwrap_or(0.5)
            })
            .sum();

        total_complexity / tasks.len() as f64
    }

    /// Update pattern embedding (for online learning)
    pub fn update_pattern_embedding(&mut self, pattern_type: String, embedding: Vec<f64>) {
        self.pattern_cache.insert(pattern_type, embedding);
    }
}

impl Default for FeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Simplified workflow definition for feature extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub workflow_id: String,
    pub pattern_type: String,
    pub tasks: Vec<Task>,
    pub data_size_bytes: usize,
}

impl WorkflowDefinition {
    fn count_dependencies(&self) -> usize {
        self.tasks.iter().map(|t| t.dependencies.len()).sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub task_id: String,
    pub task_type: String,
    pub dependencies: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_vector_conversion() {
        let embedding = vec![1.0; 64];
        let features = WorkflowFeatures::default_with_embedding(embedding);

        let vec = features.to_vector();
        assert_eq!(vec.len(), 70); // 64 + 6 additional features

        let reconstructed = WorkflowFeatures::from_vector(vec).unwrap();
        assert_eq!(reconstructed.num_tasks, 1.0);
    }

    #[test]
    fn test_feature_extractor() {
        let extractor = FeatureExtractor::new();

        let workflow = WorkflowDefinition {
            workflow_id: "test-wf".to_string(),
            pattern_type: "sequence".to_string(),
            tasks: vec![
                Task {
                    task_id: "task1".to_string(),
                    task_type: "compute".to_string(),
                    dependencies: vec![],
                },
                Task {
                    task_id: "task2".to_string(),
                    task_type: "io".to_string(),
                    dependencies: vec!["task1".to_string()],
                },
            ],
            data_size_bytes: 1024,
        };

        let features = extractor.extract(&workflow).unwrap();
        assert_eq!(features.num_tasks, 2.0);
        assert_eq!(features.num_dependencies, 1.0);
        assert_eq!(features.data_size_bytes, 1024.0);
    }

    #[test]
    fn test_pattern_embeddings() {
        let extractor = FeatureExtractor::new();

        // Should have common patterns
        assert!(extractor.pattern_cache.contains_key("sequence"));
        assert!(extractor.pattern_cache.contains_key("parallel_split"));
    }
}
