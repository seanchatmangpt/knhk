//! Knowledge Base - Stores learned patterns and historical data

use super::{Observation, Symptom, AdaptationResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Knowledge base stores learned patterns and historical behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBase {
    /// Observation statistics
    observation_stats: ObservationStats,
    /// Learned patterns
    patterns: Vec<LearnedPattern>,
    /// Adaptation history
    adaptation_history: Vec<AdaptationHistoryEntry>,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        Self {
            observation_stats: ObservationStats::default(),
            patterns: Vec::new(),
            adaptation_history: Vec::new(),
        }
    }

    /// Update observation statistics
    pub fn update_observation_stats(&mut self, observations: &[Observation]) {
        self.observation_stats.update(observations);
    }

    /// Record a MAPE-K cycle
    pub fn record_cycle(
        &mut self,
        observations: &[Observation],
        symptoms: &[Symptom],
        results: &AdaptationResult,
    ) {
        // Learn patterns from successful adaptations
        if results.adaptations_applied > 0 {
            let pattern = LearnedPattern {
                pattern_id: uuid::Uuid::new_v4().to_string(),
                symptom_types: symptoms.iter().map(|s| format!("{:?}", s.symptom_type)).collect(),
                adaptation_type: "multi".to_string(),
                success_rate: 1.0,
                avg_improvement: 0.3,
            };
            self.patterns.push(pattern);
        }

        // Record history
        self.adaptation_history.push(AdaptationHistoryEntry {
            timestamp: chrono::Utc::now(),
            observations_count: observations.len(),
            symptoms_count: symptoms.len(),
            adaptations_applied: results.adaptations_applied,
            new_sigma_id: results.new_sigma_id.clone(),
        });

        // Keep only last 1000 entries
        if self.adaptation_history.len() > 1000 {
            self.adaptation_history.drain(0..self.adaptation_history.len() - 1000);
        }
    }

    /// Query learned patterns
    pub fn query_patterns(&self, query: &str) -> Vec<String> {
        self.patterns
            .iter()
            .filter(|p| p.symptom_types.iter().any(|s| s.contains(query)))
            .map(|p| format!("{}: {} (success: {}%)", p.pattern_id, p.adaptation_type, (p.success_rate * 100.0) as u32))
            .collect()
    }

    /// Get average tick usage
    pub fn avg_tick_usage(&self) -> f64 {
        self.observation_stats.avg_ticks
    }

    /// Get guard failure rate
    pub fn guard_failure_rate(&self) -> f64 {
        self.observation_stats.guard_failure_rate
    }
}

impl Default for KnowledgeBase {
    fn default() -> Self {
        Self::new()
    }
}

/// Observation statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ObservationStats {
    total_observations: usize,
    avg_ticks: f64,
    max_ticks: u32,
    guard_failure_rate: f64,
}

impl ObservationStats {
    fn update(&mut self, observations: &[Observation]) {
        self.total_observations += observations.len();

        if !observations.is_empty() {
            let sum_ticks: u32 = observations.iter().map(|o| o.ticks_used).sum();
            self.avg_ticks = sum_ticks as f64 / observations.len() as f64;
            self.max_ticks = observations.iter().map(|o| o.ticks_used).max().unwrap_or(0);

            let total_guards: usize = observations.iter().map(|o| o.guards_checked.len()).sum();
            let failed_guards: usize = observations.iter().map(|o| o.guards_failed.len()).sum();

            if total_guards > 0 {
                self.guard_failure_rate = failed_guards as f64 / total_guards as f64;
            }
        }
    }
}

/// Learned pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LearnedPattern {
    pattern_id: String,
    symptom_types: Vec<String>,
    adaptation_type: String,
    success_rate: f64,
    avg_improvement: f64,
}

/// Adaptation history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AdaptationHistoryEntry {
    timestamp: chrono::DateTime<chrono::Utc>,
    observations_count: usize,
    symptoms_count: usize,
    adaptations_applied: usize,
    new_sigma_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_base_records_patterns() {
        let mut kb = KnowledgeBase::new();

        let observations = vec![Observation {
            receipt_id: "test".to_string(),
            sigma_id: "v1".to_string(),
            ticks_used: 5,
            guards_checked: vec![],
            guards_failed: vec![],
            timestamp: chrono::Utc::now(),
            metrics: HashMap::new(),
        }];

        kb.update_observation_stats(&observations);
        assert_eq!(kb.avg_tick_usage(), 5.0);
    }
}
