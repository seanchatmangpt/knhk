// Learning System: Continuous improvement from proposal outcomes
// Tracks accepted/rejected proposals, adapts prompts, builds few-shot corpus

use crate::proposer::{Proposal, ValidationReport, Sector, FewShotExample, SigmaDiff};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LearningError {
    #[error("Corpus serialization failed: {0}")]
    SerializationFailed(String),

    #[error("Corpus deserialization failed: {0}")]
    DeserializationFailed(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, LearningError>;

/// Proposal outcome (accepted or rejected with feedback)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProposalOutcome {
    pub proposal: Proposal,
    pub validation_report: ValidationReport,
    pub timestamp: DateTime<Utc>,
    pub feedback: Option<String>,
}

impl ProposalOutcome {
    pub fn from_proposal(proposal: Proposal, report: ValidationReport) -> Self {
        ProposalOutcome {
            proposal,
            validation_report: report,
            timestamp: Utc::now(),
            feedback: None,
        }
    }

    pub fn with_feedback(mut self, feedback: String) -> Self {
        self.feedback = Some(feedback);
        self
    }
}

/// Corpus of accepted and rejected proposals for learning
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProposalCorpus {
    pub accepted_proposals: Vec<ProposalOutcome>,
    pub rejected_proposals: Vec<ProposalOutcome>,
    pub constraint_violations: HashMap<String, Vec<String>>, // constraint -> proposal IDs
    pub sector_examples: HashMap<Sector, Vec<FewShotExample>>,
    pub metrics: LearningMetrics,
}

impl ProposalCorpus {
    pub fn new() -> Self {
        ProposalCorpus {
            accepted_proposals: Vec::new(),
            rejected_proposals: Vec::new(),
            constraint_violations: HashMap::new(),
            sector_examples: HashMap::new(),
            metrics: LearningMetrics::default(),
        }
    }

    pub fn total_proposals(&self) -> usize {
        self.accepted_proposals.len() + self.rejected_proposals.len()
    }
}

impl Default for ProposalCorpus {
    fn default() -> Self {
        Self::new()
    }
}

/// Learning metrics for tracking improvement
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LearningMetrics {
    pub acceptance_rate: f64,
    pub acceptance_rate_trend: Vec<(DateTime<Utc>, f64)>,
    pub q3_violation_rate: f64,
    pub doctrine_violation_rate: f64,
    pub avg_confidence_accepted: f64,
    pub avg_confidence_rejected: f64,
    pub first_try_success_rate: f64,
}

impl LearningMetrics {
    pub fn new() -> Self {
        LearningMetrics {
            acceptance_rate: 0.0,
            acceptance_rate_trend: Vec::new(),
            q3_violation_rate: 0.0,
            doctrine_violation_rate: 0.0,
            avg_confidence_accepted: 0.0,
            avg_confidence_rejected: 0.0,
            first_try_success_rate: 0.0,
        }
    }
}

impl Default for LearningMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Main learning system for proposal adaptation
pub struct LearningSystem {
    corpus: ProposalCorpus,
    prompt_adapter: PromptAdapter,
}

impl LearningSystem {
    pub fn new() -> Self {
        LearningSystem {
            corpus: ProposalCorpus::new(),
            prompt_adapter: PromptAdapter::new(),
        }
    }

    /// Record a proposal outcome (accepted or rejected)
    pub fn record_outcome(
        &mut self,
        proposal: Proposal,
        report: ValidationReport,
    ) -> Result<()> {
        let outcome = ProposalOutcome::from_proposal(proposal.clone(), report.clone());

        if report.passed {
            // Add to accepted proposals
            self.corpus.accepted_proposals.push(outcome.clone());

            // Extract as few-shot example
            let example = self.create_few_shot_example(&outcome);
            self.corpus.sector_examples
                .entry(proposal.pattern.sector.clone())
                .or_default()
                .push(example);

            tracing::info!(
                proposal_id = %proposal.id,
                confidence = proposal.confidence,
                sector = %proposal.pattern.sector,
                "Accepted proposal recorded for learning"
            );
        } else {
            // Add to rejected proposals
            self.corpus.rejected_proposals.push(outcome.clone());

            // Track constraint violations
            for stage in &report.stages {
                if !stage.passed {
                    self.corpus.constraint_violations
                        .entry(stage.name.clone())
                        .or_default()
                        .push(proposal.id.clone());
                }
            }

            tracing::warn!(
                proposal_id = %proposal.id,
                violations = ?self.get_violations(&report),
                "Rejected proposal recorded for learning"
            );
        }

        // Update metrics
        self.update_metrics();

        // Adapt prompts based on patterns
        self.adapt_prompts_from_patterns()?;

        Ok(())
    }

    fn create_few_shot_example(&self, outcome: &ProposalOutcome) -> FewShotExample {
        FewShotExample {
            pattern: outcome.proposal.pattern.description.clone(),
            proposal: outcome.proposal.delta_sigma.clone(),
            reasoning: outcome.proposal.reasoning.clone(),
            confidence: outcome.proposal.confidence,
            validation_result: outcome.validation_report.clone(),
        }
    }

    fn get_violations(&self, report: &ValidationReport) -> Vec<String> {
        report.stages.iter()
            .filter(|s| !s.passed)
            .map(|s| s.name.clone())
            .collect()
    }

    fn update_metrics(&mut self) {
        let total = self.corpus.total_proposals();

        if total == 0 {
            return;
        }

        // Acceptance rate
        self.corpus.metrics.acceptance_rate =
            self.corpus.accepted_proposals.len() as f64 / total as f64;

        // Track trend
        self.corpus.metrics.acceptance_rate_trend.push((
            Utc::now(),
            self.corpus.metrics.acceptance_rate,
        ));

        // Q3 violation rate
        let q3_violations = self.corpus.constraint_violations
            .get("invariant_Q3")
            .map(|v| v.len())
            .unwrap_or(0);
        self.corpus.metrics.q3_violation_rate = q3_violations as f64 / total as f64;

        // Doctrine violation rate
        let doctrine_violations = self.corpus.constraint_violations
            .get("doctrine_check")
            .map(|v| v.len())
            .unwrap_or(0);
        self.corpus.metrics.doctrine_violation_rate = doctrine_violations as f64 / total as f64;

        // Average confidence (accepted)
        if !self.corpus.accepted_proposals.is_empty() {
            self.corpus.metrics.avg_confidence_accepted =
                self.corpus.accepted_proposals.iter()
                    .map(|o| o.proposal.confidence)
                    .sum::<f64>() / self.corpus.accepted_proposals.len() as f64;
        }

        // Average confidence (rejected)
        if !self.corpus.rejected_proposals.is_empty() {
            self.corpus.metrics.avg_confidence_rejected =
                self.corpus.rejected_proposals.iter()
                    .map(|o| o.proposal.confidence)
                    .sum::<f64>() / self.corpus.rejected_proposals.len() as f64;
        }

        tracing::debug!(
            acceptance_rate = self.corpus.metrics.acceptance_rate,
            q3_violations = self.corpus.metrics.q3_violation_rate,
            "Updated learning metrics"
        );
    }

    fn adapt_prompts_from_patterns(&mut self) -> Result<()> {
        // If Q3 is frequently violated, increase emphasis
        if self.corpus.metrics.q3_violation_rate > 0.2 {
            self.prompt_adapter.increase_emphasis("performance_budget", 1.5);
            tracing::info!("Increasing prompt emphasis on performance budget due to Q3 violations");
        }

        // If acceptance rate is low, add more examples
        if self.corpus.metrics.acceptance_rate < 0.5
            && self.corpus.accepted_proposals.len() > 10
        {
            self.prompt_adapter.set_example_count(5);
            tracing::info!("Increasing few-shot example count to 5");
        }

        // If doctrine violations are common, emphasize doctrines
        if self.corpus.metrics.doctrine_violation_rate > 0.15 {
            self.prompt_adapter.increase_emphasis("doctrines", 1.3);
            tracing::info!("Increasing prompt emphasis on doctrines");
        }

        Ok(())
    }

    /// Get few-shot examples for a specific sector
    pub fn get_few_shot_examples(
        &self,
        sector: &Sector,
        count: usize,
    ) -> Vec<FewShotExample> {
        self.corpus.sector_examples
            .get(sector)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .take(count)
            .collect()
    }

    /// Get learning metrics
    pub fn metrics(&self) -> &LearningMetrics {
        &self.corpus.metrics
    }

    /// Export corpus for persistence
    pub fn export_corpus(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.corpus)
            .map_err(|e| LearningError::SerializationFailed(e.to_string()))
    }

    /// Import corpus from persistent storage
    pub fn import_corpus(&mut self, json: &str) -> Result<()> {
        self.corpus = serde_json::from_str(json)
            .map_err(|e| LearningError::DeserializationFailed(e.to_string()))?;
        Ok(())
    }
}

impl Default for LearningSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Prompt adapter for dynamic emphasis and example selection
pub struct PromptAdapter {
    emphasis_weights: HashMap<String, f64>,
    example_count: usize,
}

impl PromptAdapter {
    pub fn new() -> Self {
        PromptAdapter {
            emphasis_weights: HashMap::new(),
            example_count: 3,
        }
    }

    pub fn increase_emphasis(&mut self, section: &str, weight: f64) {
        *self.emphasis_weights.entry(section.to_string()).or_insert(1.0) = weight;
        tracing::debug!(section = section, weight = weight, "Updated prompt emphasis");
    }

    pub fn set_example_count(&mut self, count: usize) {
        self.example_count = count;
        tracing::debug!(count = count, "Updated few-shot example count");
    }

    pub fn get_emphasis(&self, section: &str) -> f64 {
        *self.emphasis_weights.get(section).unwrap_or(&1.0)
    }

    pub fn get_example_count(&self) -> usize {
        self.example_count
    }
}

impl Default for PromptAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proposer::ValidationStage;
    use crate::observation::{DetectedPattern, PatternAction};

    fn create_test_proposal(sector: Sector, estimated_ticks: u32) -> Proposal {
        Proposal {
            id: format!("test-prop-{}", uuid::Uuid::new_v4()),
            pattern_id: "test-pattern".to_string(),
            llm_prompt: String::new(),
            llm_response: String::new(),
            delta_sigma: SigmaDiff::default(),
            reasoning: "Test reasoning".to_string(),
            confidence: 0.85,
            estimated_ticks,
            doctrines_satisfied: vec![],
            invariants_satisfied: vec!["Q1".to_string(), "Q2".to_string()],
            can_rollback: true,
            timestamp: Utc::now(),
            pattern: DetectedPattern {
                id: "test-pattern".to_string(),
                name: "Test Pattern".to_string(),
                description: "Test pattern description".to_string(),
                sector,
                confidence: 0.85,
                observations: vec![],
                timestamp: Utc::now(),
                recommended_action: PatternAction::ProposeChange {
                    description: "Test change".to_string(),
                },
            },
        }
    }

    fn create_passing_report(proposal_id: String) -> ValidationReport {
        let mut report = ValidationReport::new(proposal_id);
        report.add_pass("static_check");
        report.add_pass("invariant_Q1");
        report.add_pass("invariant_Q2");
        report.add_pass("invariant_Q3");
        report
    }

    fn create_failing_report(proposal_id: String) -> ValidationReport {
        let mut report = ValidationReport::new(proposal_id);
        report.add_pass("static_check");
        report.add_fail("invariant_Q3", "Estimated ticks exceed 8".to_string());
        report
    }

    #[test]
    fn test_record_accepted_proposal() {
        let mut learning = LearningSystem::new();
        let proposal = create_test_proposal(Sector::Finance, 6);
        let report = create_passing_report(proposal.id.clone());

        learning.record_outcome(proposal.clone(), report).unwrap();

        assert_eq!(learning.corpus.accepted_proposals.len(), 1);
        assert_eq!(learning.corpus.rejected_proposals.len(), 0);
        assert_eq!(learning.corpus.metrics.acceptance_rate, 1.0);
    }

    #[test]
    fn test_record_rejected_proposal() {
        let mut learning = LearningSystem::new();
        let proposal = create_test_proposal(Sector::Finance, 12);
        let report = create_failing_report(proposal.id.clone());

        learning.record_outcome(proposal.clone(), report).unwrap();

        assert_eq!(learning.corpus.accepted_proposals.len(), 0);
        assert_eq!(learning.corpus.rejected_proposals.len(), 1);
        assert_eq!(learning.corpus.metrics.acceptance_rate, 0.0);

        // Check Q3 violation tracked
        assert!(learning.corpus.constraint_violations.contains_key("invariant_Q3"));
    }

    #[test]
    fn test_metrics_calculation() {
        let mut learning = LearningSystem::new();

        // Add 3 accepted proposals
        for i in 0..3 {
            let proposal = create_test_proposal(Sector::Finance, 6);
            let report = create_passing_report(proposal.id.clone());
            learning.record_outcome(proposal, report).unwrap();
        }

        // Add 1 rejected proposal
        let proposal = create_test_proposal(Sector::Finance, 12);
        let report = create_failing_report(proposal.id.clone());
        learning.record_outcome(proposal, report).unwrap();

        // Metrics should be: 3/4 = 0.75 acceptance rate
        assert!((learning.corpus.metrics.acceptance_rate - 0.75).abs() < 0.01);

        // Q3 violation rate: 1/4 = 0.25
        assert!((learning.corpus.metrics.q3_violation_rate - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_few_shot_example_extraction() {
        let mut learning = LearningSystem::new();
        let proposal = create_test_proposal(Sector::Healthcare, 6);
        let report = create_passing_report(proposal.id.clone());

        learning.record_outcome(proposal, report).unwrap();

        let examples = learning.get_few_shot_examples(&Sector::Healthcare, 5);

        assert_eq!(examples.len(), 1);
        assert_eq!(examples[0].confidence, 0.85);
    }

    #[test]
    fn test_prompt_adaptation() {
        let mut learning = LearningSystem::new();

        // Record multiple Q3 violations
        for _ in 0..5 {
            let proposal = create_test_proposal(Sector::Finance, 12);
            let report = create_failing_report(proposal.id.clone());
            learning.record_outcome(proposal, report).unwrap();
        }

        // Q3 violation rate should be 1.0, triggering emphasis increase
        assert_eq!(learning.corpus.metrics.q3_violation_rate, 1.0);
        assert!(learning.prompt_adapter.get_emphasis("performance_budget") > 1.0);
    }

    #[test]
    fn test_corpus_export_import() {
        let mut learning = LearningSystem::new();
        let proposal = create_test_proposal(Sector::Finance, 6);
        let report = create_passing_report(proposal.id.clone());

        learning.record_outcome(proposal, report).unwrap();

        // Export
        let json = learning.export_corpus().unwrap();
        assert!(!json.is_empty());

        // Import into new system
        let mut learning2 = LearningSystem::new();
        learning2.import_corpus(&json).unwrap();

        assert_eq!(learning2.corpus.accepted_proposals.len(), 1);
        assert_eq!(learning2.corpus.metrics.acceptance_rate, 1.0);
    }
}
