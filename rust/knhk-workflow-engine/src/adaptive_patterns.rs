//! Adaptive Pattern Selector
//!
//! Uses MAPE-K feedback to dynamically select optimal workflow patterns
//! based on runtime observations.

use crate::error::WorkflowResult;
use crate::patterns::PatternId;
use crate::mape::{Observation, KnowledgeBase};
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;

/// Adaptive pattern selector that learns from execution history
pub struct AdaptivePatternSelector {
    knowledge: Arc<RwLock<KnowledgeBase>>,
    pattern_performance: HashMap<PatternId, PatternPerformance>,
}

impl AdaptivePatternSelector {
    pub fn new(knowledge: Arc<RwLock<KnowledgeBase>>) -> Self {
        Self {
            knowledge,
            pattern_performance: HashMap::new(),
        }
    }

    /// Select optimal pattern based on context and learned behavior
    ///
    /// This implements adaptive pattern selection:
    /// 1. Consider current context (data size, concurrency, etc.)
    /// 2. Query knowledge base for historical performance
    /// 3. Select pattern with best expected performance
    /// 4. Fall back to defaults if no data available
    pub fn select_pattern(
        &mut self,
        context: &PatternSelectionContext,
    ) -> WorkflowResult<PatternId> {
        // Get candidate patterns for this context
        let candidates = self.get_candidate_patterns(context);

        // Score each candidate based on learned performance
        let mut best_pattern = PatternId::Sequence; // Default
        let mut best_score = 0.0;

        for pattern in candidates {
            let score = self.score_pattern(&pattern, context);
            if score > best_score {
                best_score = score;
                best_pattern = pattern;
            }
        }

        tracing::debug!(
            "Selected pattern {:?} with score {} for context: data_size={}, concurrency={}",
            best_pattern,
            best_score,
            context.data_size,
            context.concurrency_level
        );

        Ok(best_pattern)
    }

    /// Update pattern performance based on execution observation
    pub fn record_execution(&mut self, pattern: PatternId, observation: &Observation) {
        let perf = self.pattern_performance
            .entry(pattern)
            .or_insert_with(|| PatternPerformance::default());

        perf.total_executions += 1;
        perf.total_ticks += observation.ticks_used as u64;
        perf.avg_ticks = perf.total_ticks as f64 / perf.total_executions as f64;

        if !observation.guards_failed.is_empty() {
            perf.failures += 1;
        }

        perf.success_rate = 1.0 - (perf.failures as f64 / perf.total_executions as f64);
    }

    /// Get candidate patterns for context
    fn get_candidate_patterns(&self, context: &PatternSelectionContext) -> Vec<PatternId> {
        // Select candidates based on context
        if context.requires_parallelism {
            vec![
                PatternId::ParallelSplit,
                PatternId::MultiChoice,
                PatternId::MultipleInstancesWithDesignTimeKnowledge,
            ]
        } else if context.requires_exclusive_choice {
            vec![
                PatternId::ExclusiveChoice,
                PatternId::MultiChoice,
                PatternId::DeferredChoice,
            ]
        } else {
            vec![
                PatternId::Sequence,
                PatternId::ParallelSplit,
            ]
        }
    }

    /// Score a pattern for given context
    fn score_pattern(&self, pattern: &PatternId, context: &PatternSelectionContext) -> f64 {
        let perf = self.pattern_performance.get(pattern);

        match perf {
            Some(p) if p.total_executions > 10 => {
                // Use learned performance
                let tick_score = 1.0 - (p.avg_ticks / 8.0); // Lower ticks = higher score
                let success_score = p.success_rate;

                // Weight by context
                let context_score = self.context_affinity(pattern, context);

                (tick_score * 0.4 + success_score * 0.3 + context_score * 0.3)
            }
            _ => {
                // Fall back to default scoring based on pattern characteristics
                self.default_score(pattern, context)
            }
        }
    }

    /// Calculate context affinity (how well pattern matches context)
    fn context_affinity(&self, pattern: &PatternId, context: &PatternSelectionContext) -> f64 {
        match pattern {
            PatternId::ParallelSplit => {
                if context.requires_parallelism { 1.0 } else { 0.3 }
            }
            PatternId::ExclusiveChoice => {
                if context.requires_exclusive_choice { 1.0 } else { 0.5 }
            }
            PatternId::MultiChoice => {
                if context.concurrency_level > 2 { 0.9 } else { 0.4 }
            }
            PatternId::Sequence => {
                if !context.requires_parallelism { 1.0 } else { 0.2 }
            }
            _ => 0.5, // Neutral affinity
        }
    }

    /// Default scoring when no learned data available
    fn default_score(&self, pattern: &PatternId, context: &PatternSelectionContext) -> f64 {
        // Simple heuristics
        match pattern {
            PatternId::Sequence => 0.7, // Safe default
            PatternId::ParallelSplit => {
                if context.requires_parallelism { 0.8 } else { 0.3 }
            }
            PatternId::ExclusiveChoice => {
                if context.requires_exclusive_choice { 0.8 } else { 0.4 }
            }
            _ => 0.5,
        }
    }

    /// Get pattern performance statistics
    pub fn get_pattern_stats(&self, pattern: &PatternId) -> Option<PatternPerformanceStats> {
        self.pattern_performance.get(pattern).map(|p| PatternPerformanceStats {
            total_executions: p.total_executions,
            avg_ticks: p.avg_ticks,
            success_rate: p.success_rate,
            failures: p.failures,
        })
    }
}

/// Context for pattern selection
#[derive(Debug, Clone)]
pub struct PatternSelectionContext {
    /// Data size being processed
    pub data_size: usize,
    /// Required concurrency level
    pub concurrency_level: usize,
    /// Whether parallelism is required
    pub requires_parallelism: bool,
    /// Whether exclusive choice is required
    pub requires_exclusive_choice: bool,
    /// SLO requirements (max ticks)
    pub max_ticks: u32,
}

/// Pattern performance tracking
#[derive(Debug, Clone, Default)]
struct PatternPerformance {
    total_executions: u64,
    total_ticks: u64,
    avg_ticks: f64,
    failures: u64,
    success_rate: f64,
}

/// Pattern performance statistics
#[derive(Debug, Clone)]
pub struct PatternPerformanceStats {
    pub total_executions: u64,
    pub avg_ticks: f64,
    pub success_rate: f64,
    pub failures: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_pattern_selection() {
        let knowledge = Arc::new(RwLock::new(KnowledgeBase::new()));
        let mut selector = AdaptivePatternSelector::new(knowledge);

        let context = PatternSelectionContext {
            data_size: 1000,
            concurrency_level: 4,
            requires_parallelism: true,
            requires_exclusive_choice: false,
            max_ticks: 8,
        };

        let pattern = selector.select_pattern(&context).unwrap();

        // Should select a parallel pattern
        assert!(matches!(
            pattern,
            PatternId::ParallelSplit | PatternId::MultiChoice | PatternId::MultipleInstancesWithDesignTimeKnowledge
        ));
    }

    #[test]
    fn test_pattern_learning() {
        let knowledge = Arc::new(RwLock::new(KnowledgeBase::new()));
        let mut selector = AdaptivePatternSelector::new(knowledge);

        let observation = Observation {
            receipt_id: "test".to_string(),
            sigma_id: "v1".to_string(),
            ticks_used: 3,
            guards_checked: vec![],
            guards_failed: vec![],
            timestamp: chrono::Utc::now(),
            metrics: HashMap::new(),
        };

        selector.record_execution(PatternId::Sequence, &observation);

        let stats = selector.get_pattern_stats(&PatternId::Sequence).unwrap();
        assert_eq!(stats.total_executions, 1);
        assert_eq!(stats.avg_ticks, 3.0);
        assert_eq!(stats.success_rate, 1.0);
    }
}
