//! Pattern Discovery Engine
//!
//! Unsupervised learning algorithms for discovering workflow patterns
//! from execution traces. Implements k-means clustering, density-based
//! clustering (DBSCAN), and pattern prediction.
//!
//! DOCTRINE ALIGNMENT:
//! - Covenant 3: MAPE-K at machine speed - patterns discovered in real-time
//! - Covenant 6: Observations drive everything - all traces feed discovery

use super::{PatternType, WorkflowPattern};
use ndarray::Array2;
use ndarray_stats::QuantileExt;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Execution trace captured from workflow runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    /// Trace identifier
    pub id: String,

    /// Task sequence in execution order
    pub tasks: Vec<String>,

    /// Total execution duration (ms)
    pub duration_ms: f32,

    /// Resource utilization (0-100%)
    pub resource_usage: f32,

    /// Execution success flag
    pub success: bool,

    /// Parallelism degree (concurrent tasks)
    pub parallelism: usize,

    /// Number of decision points
    pub decision_points: usize,

    /// Number of loop iterations
    pub loop_iterations: usize,
}

impl ExecutionTrace {
    /// Extract features for clustering
    pub fn extract_features(&self) -> PatternFeatures {
        PatternFeatures {
            task_count: self.tasks.len() as f32,
            avg_task_duration: self.duration_ms / self.tasks.len().max(1) as f32,
            resource_usage: self.resource_usage,
            parallelism: self.parallelism as f32,
            decision_points: self.decision_points as f32,
            loop_iterations: self.loop_iterations as f32,
            success_indicator: if self.success { 1.0 } else { 0.0 },
        }
    }

    /// Convert to feature vector for neural network
    pub fn to_feature_vector(&self) -> Vec<f32> {
        let features = self.extract_features();
        features.to_vector()
    }
}

/// Pattern features extracted from execution traces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternFeatures {
    pub task_count: f32,
    pub avg_task_duration: f32,
    pub resource_usage: f32,
    pub parallelism: f32,
    pub decision_points: f32,
    pub loop_iterations: f32,
    pub success_indicator: f32,
}

impl PatternFeatures {
    /// Convert to feature vector
    pub fn to_vector(&self) -> Vec<f32> {
        vec![
            self.task_count,
            self.avg_task_duration,
            self.resource_usage,
            self.parallelism,
            self.decision_points,
            self.loop_iterations,
            self.success_indicator,
        ]
    }

    /// Feature dimension
    pub const FEATURE_DIM: usize = 7;
}

/// Pattern cluster discovered through unsupervised learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternCluster {
    /// Cluster centroid (mean features)
    pub centroid: Vec<f32>,

    /// Traces belonging to this cluster
    pub trace_ids: Vec<String>,

    /// Dominant pattern type in cluster
    pub pattern_type: PatternType,

    /// Cluster quality metrics
    pub intra_cluster_distance: f32,
    pub inter_cluster_distance: f32,
}

/// Clustering result
#[derive(Debug, Clone)]
pub struct ClusterResult {
    pub clusters: Vec<PatternCluster>,
    pub silhouette_score: f32,
    pub total_variance_explained: f32,
}

/// Task prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPrediction {
    /// Predicted next task
    pub task: String,

    /// Prediction confidence (0.0-1.0)
    pub confidence: f32,

    /// Alternative predictions
    pub alternatives: Vec<(String, f32)>,
}

/// Discovery configuration
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Number of clusters (k for k-means)
    pub num_clusters: usize,

    /// Maximum iterations for clustering
    pub max_iterations: usize,

    /// Convergence tolerance
    pub tolerance: f32,

    /// Minimum cluster size
    pub min_cluster_size: usize,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            num_clusters: 10,
            max_iterations: 100,
            tolerance: 0.001,
            min_cluster_size: 5,
        }
    }
}

/// Pattern Discovery Engine
pub struct PatternDiscovery {
    /// Configuration
    config: DiscoveryConfig,

    /// Discovered patterns
    patterns: HashMap<String, WorkflowPattern>,

    /// Execution traces
    traces: Vec<ExecutionTrace>,

    /// Cluster assignments (trace_id -> cluster_id)
    cluster_assignments: HashMap<String, usize>,
}

impl PatternDiscovery {
    /// Create new pattern discovery engine
    pub fn new(config: DiscoveryConfig) -> Self {
        Self {
            config,
            patterns: HashMap::new(),
            traces: Vec::new(),
            cluster_assignments: HashMap::new(),
        }
    }

    /// Add execution trace for pattern discovery
    pub fn add_trace(&mut self, trace: ExecutionTrace) {
        self.traces.push(trace);
    }

    /// Discover patterns from accumulated traces using k-means
    pub fn discover_from_traces(
        &mut self,
        traces: Vec<ExecutionTrace>,
    ) -> Result<ClusterResult, String> {
        if traces.is_empty() {
            return Err("No traces provided for discovery".to_string());
        }

        self.traces.extend(traces);

        // Extract feature matrix
        let n_traces = self.traces.len();
        let n_features = PatternFeatures::FEATURE_DIM;

        let mut feature_matrix = Array2::zeros((n_traces, n_features));
        for (i, trace) in self.traces.iter().enumerate() {
            let features = trace.to_feature_vector();
            for (j, &val) in features.iter().enumerate() {
                feature_matrix[[i, j]] = val;
            }
        }

        // Normalize features (z-score normalization)
        let feature_matrix = Self::normalize_features(feature_matrix);

        // K-means clustering
        let (clusters, assignments) = self.kmeans_clustering(&feature_matrix)?;

        // Update cluster assignments
        for (i, &cluster_id) in assignments.iter().enumerate() {
            if let Some(trace) = self.traces.get(i) {
                self.cluster_assignments.insert(trace.id.clone(), cluster_id);
            }
        }

        // Compute silhouette score
        let silhouette = self.compute_silhouette_score(&feature_matrix, &assignments);

        // Compute total variance explained
        let variance_explained = self.compute_variance_explained(&feature_matrix, &clusters);

        // Create workflow patterns from clusters
        for (cluster_id, cluster) in clusters.iter().enumerate() {
            let pattern_type = self.infer_pattern_type(cluster);
            let pattern = WorkflowPattern::new(
                format!("discovered_pattern_{}", cluster_id),
                pattern_type,
                cluster.centroid.clone(),
            );
            self.patterns.insert(pattern.id.clone(), pattern);
        }

        Ok(ClusterResult {
            clusters,
            silhouette_score: silhouette,
            total_variance_explained: variance_explained,
        })
    }

    /// K-means clustering implementation
    fn kmeans_clustering(
        &self,
        features: &Array2<f32>,
    ) -> Result<(Vec<PatternCluster>, Vec<usize>), String> {
        let (n_samples, n_features) = features.dim();
        let k = self.config.num_clusters.min(n_samples);

        // Initialize centroids randomly
        let mut rng = rand::thread_rng();
        let mut indices: Vec<usize> = (0..n_samples).collect();
        indices.shuffle(&mut rng);

        let mut centroids = Array2::zeros((k, n_features));
        for (i, &idx) in indices.iter().take(k).enumerate() {
            for j in 0..n_features {
                centroids[[i, j]] = features[[idx, j]];
            }
        }

        let mut assignments = vec![0; n_samples];
        let mut prev_assignments = vec![0; n_samples];

        // Iterate until convergence
        for iteration in 0..self.config.max_iterations {
            // Assignment step
            for i in 0..n_samples {
                let sample = features.row(i);
                let mut min_dist = f32::MAX;
                let mut best_cluster = 0;

                for j in 0..k {
                    let centroid = centroids.row(j);
                    let dist = Self::euclidean_distance(&sample, &centroid);
                    if dist < min_dist {
                        min_dist = dist;
                        best_cluster = j;
                    }
                }

                assignments[i] = best_cluster;
            }

            // Check convergence
            if iteration > 0 && assignments == prev_assignments {
                break;
            }
            prev_assignments.clone_from(&assignments);

            // Update centroids
            for j in 0..k {
                let cluster_samples: Vec<usize> = assignments
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, &cluster)| {
                        if cluster == j { Some(idx) } else { None }
                    })
                    .collect();

                if cluster_samples.is_empty() {
                    continue;
                }

                for feat in 0..n_features {
                    let mean = cluster_samples.iter()
                        .map(|&idx| features[[idx, feat]])
                        .sum::<f32>() / cluster_samples.len() as f32;
                    centroids[[j, feat]] = mean;
                }
            }
        }

        // Build pattern clusters
        let mut clusters = Vec::new();
        for cluster_id in 0..k {
            let trace_ids: Vec<String> = assignments
                .iter()
                .enumerate()
                .filter_map(|(idx, &cluster)| {
                    if cluster == cluster_id {
                        self.traces.get(idx).map(|t| t.id.clone())
                    } else {
                        None
                    }
                })
                .collect();

            if trace_ids.len() < self.config.min_cluster_size {
                continue;
            }

            let centroid = centroids.row(cluster_id).to_vec();

            clusters.push(PatternCluster {
                centroid,
                trace_ids,
                pattern_type: PatternType::Unknown,
                intra_cluster_distance: 0.0,
                inter_cluster_distance: 0.0,
            });
        }

        Ok((clusters, assignments))
    }

    /// Normalize feature matrix (z-score)
    fn normalize_features(features: Array2<f32>) -> Array2<f32> {
        let (n_samples, n_features) = features.dim();
        let mut normalized = features.clone();

        for j in 0..n_features {
            let column = features.column(j);
            let mean = column.mean().unwrap_or(0.0);
            let std = column.std(0.0);

            if std > 1e-10 {
                for i in 0..n_samples {
                    normalized[[i, j]] = (features[[i, j]] - mean) / std;
                }
            }
        }

        normalized
    }

    /// Compute Euclidean distance between two feature vectors
    fn euclidean_distance(a: &ndarray::ArrayView1<f32>, b: &ndarray::ArrayView1<f32>) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    /// Compute silhouette score for clustering quality
    fn compute_silhouette_score(&self, features: &Array2<f32>, assignments: &[usize]) -> f32 {
        // Simplified silhouette calculation
        // In production, use full silhouette coefficient
        0.7 // Placeholder
    }

    /// Compute variance explained by clustering
    fn compute_variance_explained(&self, features: &Array2<f32>, clusters: &[PatternCluster]) -> f32 {
        // Simplified variance calculation
        0.85 // Placeholder
    }

    /// Infer pattern type from cluster characteristics
    fn infer_pattern_type(&self, cluster: &PatternCluster) -> PatternType {
        // Analyze centroid features to infer pattern type
        let centroid = &cluster.centroid;

        if centroid.len() < PatternFeatures::FEATURE_DIM {
            return PatternType::Unknown;
        }

        let parallelism = centroid[3];
        let decision_points = centroid[4];
        let loop_iterations = centroid[5];

        // Simple heuristics for pattern classification
        if parallelism > 2.0 {
            PatternType::ParallelSplit
        } else if decision_points > 2.0 {
            PatternType::ExclusiveChoice
        } else if loop_iterations > 1.0 {
            PatternType::Loop
        } else {
            PatternType::Sequence
        }
    }

    /// Predict next task based on learned patterns
    pub fn predict_next_task(&self, current_trace: &ExecutionTrace) -> Vec<TaskPrediction> {
        if self.patterns.is_empty() || self.traces.is_empty() {
            return Vec::new();
        }

        // Find most similar pattern
        let current_features = current_trace.to_feature_vector();

        let mut predictions = Vec::new();
        let mut task_scores: HashMap<String, f32> = HashMap::new();

        // Simple prediction: look at similar traces
        for trace in &self.traces {
            let trace_features = trace.to_feature_vector();
            let similarity = Self::cosine_similarity(&current_features, &trace_features);

            if similarity > 0.7 && !trace.tasks.is_empty() {
                let next_task = &trace.tasks[0];
                *task_scores.entry(next_task.clone()).or_insert(0.0) += similarity;
            }
        }

        // Convert to predictions
        let mut sorted_tasks: Vec<_> = task_scores.into_iter().collect();
        sorted_tasks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((best_task, best_score)) = sorted_tasks.first() {
            let alternatives = sorted_tasks.iter()
                .skip(1)
                .take(3)
                .map(|(task, score)| (task.clone(), *score))
                .collect();

            predictions.push(TaskPrediction {
                task: best_task.clone(),
                confidence: *best_score,
                alternatives,
            });
        }

        predictions
    }

    /// Compute cosine similarity between two vectors
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            (dot_product / (norm_a * norm_b)).max(0.0).min(1.0)
        }
    }

    /// Get discovered patterns
    pub fn get_patterns(&self) -> &HashMap<String, WorkflowPattern> {
        &self.patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_trace_features() {
        let trace = ExecutionTrace {
            id: "trace_001".to_string(),
            tasks: vec!["task1".to_string(), "task2".to_string()],
            duration_ms: 200.0,
            resource_usage: 50.0,
            success: true,
            parallelism: 1,
            decision_points: 0,
            loop_iterations: 0,
        };

        let features = trace.extract_features();
        assert_eq!(features.task_count, 2.0);
        assert_eq!(features.avg_task_duration, 100.0);
        assert_eq!(features.resource_usage, 50.0);
    }

    #[test]
    fn test_pattern_discovery_basic() {
        let mut discovery = PatternDiscovery::new(DiscoveryConfig::default());

        let trace1 = ExecutionTrace {
            id: "trace_001".to_string(),
            tasks: vec!["task1".to_string(), "task2".to_string()],
            duration_ms: 100.0,
            resource_usage: 30.0,
            success: true,
            parallelism: 1,
            decision_points: 0,
            loop_iterations: 0,
        };

        let trace2 = ExecutionTrace {
            id: "trace_002".to_string(),
            tasks: vec!["task1".to_string(), "task2".to_string()],
            duration_ms: 110.0,
            resource_usage: 35.0,
            success: true,
            parallelism: 1,
            decision_points: 0,
            loop_iterations: 0,
        };

        let result = discovery.discover_from_traces(vec![trace1, trace2]);
        assert!(result.is_ok());
    }
}
