//! MAPE-K Integration for Neural Learning
//!
//! Implements hooks that connect the neural learning system to the
//! MAPE-K autonomic control loop. This enables continuous learning
//! from workflow executions and automated optimization.
//!
//! DOCTRINE ALIGNMENT:
//! - Principle: MAPE-K (Monitor-Analyze-Plan-Execute-Knowledge)
//! - Covenant 3: Feedback loops run at machine speed
//! - Covenant 5: Chatman constant guards all complexity (≤8 ticks hot path)
//! - Covenant 6: Observations drive everything

use crate::patterns::{ExecutionTrace, PatternDiscovery};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Recommendation for workflow optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    /// Recommendation type
    pub recommendation_type: RecommendationType,

    /// Target workflow or task
    pub target: String,

    /// Confidence score (0.0-1.0)
    pub confidence: f32,

    /// Expected improvement (%)
    pub expected_improvement: f32,

    /// Rationale for recommendation
    pub rationale: String,

    /// Specific parameters to adjust
    pub parameters: HashMap<String, f32>,
}

/// Types of optimization recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    /// Increase parallelism degree
    IncreaseParallelism,

    /// Reduce resource allocation
    ReduceResources,

    /// Reorder task execution
    ReorderTasks,

    /// Cache intermediate results
    EnableCaching,

    /// Switch to different pattern
    SwitchPattern { new_pattern: String },

    /// Adjust timeout values
    AdjustTimeouts,

    /// Enable prefetching
    EnablePrefetching,
}

/// MAPE-K neural hooks integration
pub struct MapekNeuralHooks {
    /// Pattern discovery engine
    discovery: Arc<RwLock<PatternDiscovery>>,

    /// Execution trace buffer
    trace_buffer: Arc<RwLock<Vec<ExecutionTrace>>>,

    /// Learned recommendations
    recommendations: Arc<RwLock<Vec<OptimizationRecommendation>>>,

    /// Learning statistics
    stats: Arc<RwLock<LearningStats>>,
}

/// Learning statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct LearningStats {
    pub total_observations: usize,
    pub patterns_discovered: usize,
    pub recommendations_generated: usize,
    pub recommendations_applied: usize,
    pub avg_improvement: f32,
    pub learning_rate: f32,
}

impl MapekNeuralHooks {
    /// Create new MAPE-K neural hooks
    pub fn new(discovery: PatternDiscovery) -> Self {
        Self {
            discovery: Arc::new(RwLock::new(discovery)),
            trace_buffer: Arc::new(RwLock::new(Vec::new())),
            recommendations: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(LearningStats::default())),
        }
    }

    /// Monitor phase: Collect observations from execution
    /// HOT PATH: Must complete in ≤8 ticks (Chatman constant)
    pub async fn monitor_with_learning(&self, trace: ExecutionTrace) -> Result<(), String> {
        let start = Instant::now();

        // Add trace to buffer (fast operation)
        {
            let mut buffer = self.trace_buffer.write()
                .map_err(|e| format!("Lock error: {}", e))?;
            buffer.push(trace);

            // Update stats
            let mut stats = self.stats.write()
                .map_err(|e| format!("Lock error: {}", e))?;
            stats.total_observations += 1;
        }

        // Check hot path constraint (≤8 ticks ≈ 2ns on modern hardware)
        let elapsed = start.elapsed();
        if elapsed > Duration::from_nanos(2) {
            eprintln!(
                "MAPE-K Monitor violated Chatman constant: {:?} > 2ns",
                elapsed
            );
        }

        Ok(())
    }

    /// Analyze phase: Detect patterns and anomalies
    /// WARM PATH: Can take longer, runs asynchronously
    pub async fn analyze_with_recommendations(
        &self,
    ) -> Result<Vec<OptimizationRecommendation>, String> {
        let start = Instant::now();

        // Get traces from buffer
        let traces = {
            let mut buffer = self.trace_buffer.write()
                .map_err(|e| format!("Lock error: {}", e))?;
            std::mem::take(&mut *buffer)
        };

        if traces.is_empty() {
            return Ok(Vec::new());
        }

        // Discover patterns
        let patterns = {
            let mut discovery = self.discovery.write()
                .map_err(|e| format!("Lock error: {}", e))?;

            let result = discovery.discover_from_traces(traces.clone())?;

            // Update stats
            let mut stats = self.stats.write()
                .map_err(|e| format!("Lock error: {}", e))?;
            stats.patterns_discovered = discovery.get_patterns().len();

            result
        };

        // Generate recommendations based on patterns
        let recommendations = self.generate_recommendations(&patterns.clusters, &traces);

        // Store recommendations
        {
            let mut recs = self.recommendations.write()
                .map_err(|e| format!("Lock error: {}", e))?;
            *recs = recommendations.clone();

            let mut stats = self.stats.write()
                .map_err(|e| format!("Lock error: {}", e))?;
            stats.recommendations_generated += recommendations.len();
        }

        let elapsed = start.elapsed();
        tracing::info!(
            "MAPE-K Analyze completed in {:?}, generated {} recommendations",
            elapsed,
            recommendations.len()
        );

        Ok(recommendations)
    }

    /// Plan phase: Select best recommendations to execute
    pub async fn plan_optimizations(
        &self,
        max_recommendations: usize,
    ) -> Result<Vec<OptimizationRecommendation>, String> {
        let recommendations = {
            let recs = self.recommendations.read()
                .map_err(|e| format!("Lock error: {}", e))?;
            recs.clone()
        };

        // Sort by expected improvement and confidence
        let mut sorted = recommendations;
        sorted.sort_by(|a, b| {
            let score_a = a.expected_improvement * a.confidence;
            let score_b = b.expected_improvement * b.confidence;
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(sorted.into_iter().take(max_recommendations).collect())
    }

    /// Execute phase: Apply selected recommendations
    pub async fn execute_learned_decisions(
        &self,
        recommendations: Vec<OptimizationRecommendation>,
    ) -> Result<Vec<String>, String> {
        let mut applied = Vec::new();

        for rec in recommendations {
            // In real implementation, this would:
            // 1. Validate recommendation against Q invariants
            // 2. Apply changes to workflow configuration
            // 3. Monitor for improvement
            // 4. Rollback if degradation detected

            tracing::info!(
                "Applying recommendation: {:?} for {} (confidence: {:.2}, improvement: {:.1}%)",
                rec.recommendation_type,
                rec.target,
                rec.confidence,
                rec.expected_improvement
            );

            applied.push(rec.target);

            // Update stats
            let mut stats = self.stats.write()
                .map_err(|e| format!("Lock error: {}", e))?;
            stats.recommendations_applied += 1;
        }

        Ok(applied)
    }

    /// Knowledge phase: Update knowledge base with learned patterns
    pub async fn knowledge_update(
        &self,
        improvement: f32,
        decision: &OptimizationRecommendation,
    ) -> Result<(), String> {
        // Update learning statistics
        let mut stats = self.stats.write()
            .map_err(|e| format!("Lock error: {}", e))?;

        // Update rolling average of improvement
        let n = stats.recommendations_applied as f32;
        stats.avg_improvement = (stats.avg_improvement * (n - 1.0) + improvement) / n;

        // Adjust learning rate based on success
        if improvement > 0.0 {
            stats.learning_rate = (stats.learning_rate * 1.01).min(1.0);
        } else {
            stats.learning_rate = (stats.learning_rate * 0.99).max(0.01);
        }

        tracing::info!(
            "Knowledge update: improvement={:.2}%, avg_improvement={:.2}%, learning_rate={:.4}",
            improvement,
            stats.avg_improvement,
            stats.learning_rate
        );

        Ok(())
    }

    /// Generate recommendations from discovered patterns
    fn generate_recommendations(
        &self,
        clusters: &[crate::patterns::PatternCluster],
        traces: &[ExecutionTrace],
    ) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        for cluster in clusters {
            // Analyze cluster characteristics
            let avg_parallelism = cluster.centroid.get(3).copied().unwrap_or(0.0);
            let avg_resource_usage = cluster.centroid.get(2).copied().unwrap_or(0.0);

            // Generate recommendations based on patterns
            if avg_parallelism < 2.0 && avg_resource_usage < 50.0 {
                // Low parallelism, low resource usage → increase parallelism
                recommendations.push(OptimizationRecommendation {
                    recommendation_type: RecommendationType::IncreaseParallelism,
                    target: format!("cluster_{}", cluster.trace_ids.first().unwrap_or(&"unknown".to_string())),
                    confidence: 0.8,
                    expected_improvement: 25.0,
                    rationale: "Low resource usage indicates opportunity for parallelization".to_string(),
                    parameters: HashMap::from([
                        ("parallelism_degree".to_string(), 4.0),
                    ]),
                });
            }

            if avg_resource_usage > 80.0 {
                // High resource usage → optimize or reduce resources
                recommendations.push(OptimizationRecommendation {
                    recommendation_type: RecommendationType::ReduceResources,
                    target: format!("cluster_{}", cluster.trace_ids.first().unwrap_or(&"unknown".to_string())),
                    confidence: 0.7,
                    expected_improvement: 15.0,
                    rationale: "High resource usage detected, optimization possible".to_string(),
                    parameters: HashMap::from([
                        ("resource_limit".to_string(), 70.0),
                    ]),
                });
            }
        }

        recommendations
    }

    /// Get current learning statistics
    pub fn get_stats(&self) -> Result<LearningStats, String> {
        let stats = self.stats.read()
            .map_err(|e| format!("Lock error: {}", e))?;
        Ok(stats.clone())
    }
}

/// Full MAPE-K cycle execution
pub async fn run_mapek_cycle(hooks: &MapekNeuralHooks) -> Result<(), String> {
    // Monitor: traces are added continuously via monitor_with_learning()

    // Analyze: discover patterns and generate recommendations
    let recommendations = hooks.analyze_with_recommendations().await?;

    if recommendations.is_empty() {
        return Ok(());
    }

    // Plan: select top recommendations
    let planned = hooks.plan_optimizations(5).await?;

    // Execute: apply recommendations
    let applied = hooks.execute_learned_decisions(planned.clone()).await?;

    // Knowledge: update based on results
    for rec in planned {
        // Simulate improvement measurement
        let improvement = rec.expected_improvement * rec.confidence;
        hooks.knowledge_update(improvement, &rec).await?;
    }

    tracing::info!("MAPE-K cycle completed, applied {} recommendations", applied.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::{DiscoveryConfig, PatternDiscovery};

    #[tokio::test]
    async fn test_mapek_monitor() {
        let discovery = PatternDiscovery::new(DiscoveryConfig::default());
        let hooks = MapekNeuralHooks::new(discovery);

        let trace = ExecutionTrace {
            id: "trace_001".to_string(),
            tasks: vec!["task1".to_string()],
            duration_ms: 100.0,
            resource_usage: 50.0,
            success: true,
            parallelism: 1,
            decision_points: 0,
            loop_iterations: 0,
        };

        let result = hooks.monitor_with_learning(trace).await;
        assert!(result.is_ok());

        let stats = hooks.get_stats().unwrap();
        assert_eq!(stats.total_observations, 1);
    }

    #[tokio::test]
    async fn test_mapek_full_cycle() {
        let discovery = PatternDiscovery::new(DiscoveryConfig::default());
        let hooks = MapekNeuralHooks::new(discovery);

        // Add some traces
        for i in 0..10 {
            let trace = ExecutionTrace {
                id: format!("trace_{:03}", i),
                tasks: vec!["task1".to_string(), "task2".to_string()],
                duration_ms: 100.0 + (i as f32 * 10.0),
                resource_usage: 50.0,
                success: true,
                parallelism: 1,
                decision_points: 0,
                loop_iterations: 0,
            };
            hooks.monitor_with_learning(trace).await.unwrap();
        }

        // Run full MAPE-K cycle
        let result = run_mapek_cycle(&hooks).await;
        assert!(result.is_ok());
    }
}
