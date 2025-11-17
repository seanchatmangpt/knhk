//! Ontology-Driven Workflow Executor
//!
//! Directly executes workflows from RDF/Turtle ontologies without manual code generation.
//! Implements the complete Σ → Π → μ → O → MAPE-K pipeline.

use crate::adaptive_patterns::{AdaptivePatternSelector, PatternSelectionContext};
use crate::error::{WorkflowError, WorkflowResult};
use crate::mape::KnowledgeBase;
use crate::orchestrator::SelfExecutingOrchestrator;
use crate::parser::{WorkflowParser, WorkflowSpec};
use crate::patterns::PatternId;
use parking_lot::RwLock;
use std::path::Path;
use std::sync::Arc;

/// Ontology-driven executor that bridges RDF ontologies to executable workflows
pub struct OntologyExecutor {
    orchestrator: SelfExecutingOrchestrator,
    pattern_selector: AdaptivePatternSelector,
    knowledge: Arc<RwLock<KnowledgeBase>>,
}

impl OntologyExecutor {
    /// Create a new ontology executor
    pub fn new(snapshot_dir: &str, receipt_dir: &str) -> WorkflowResult<Self> {
        let orchestrator = SelfExecutingOrchestrator::new(snapshot_dir, receipt_dir)?;
        let knowledge = Arc::new(RwLock::new(KnowledgeBase::new()));
        let pattern_selector = AdaptivePatternSelector::new(knowledge.clone());

        Ok(Self {
            orchestrator,
            pattern_selector,
            knowledge,
        })
    }

    /// Load and execute workflow directly from RDF ontology
    ///
    /// This demonstrates the complete flow:
    /// 1. Parse RDF ontology (Σ)
    /// 2. Extract workflow structure via SPARQL
    /// 3. Select patterns adaptively
    /// 4. Execute with hook engine (μ)
    /// 5. Generate receipt (O)
    /// 6. Feed to MAPE-K
    pub async fn execute_from_ontology(
        &mut self,
        ontology_path: &str,
        input_data: serde_json::Value,
    ) -> WorkflowResult<OntologyExecutionResult> {
        // 1. Load workflow from ontology
        let workflow_id = self
            .orchestrator
            .load_workflow_from_ontology(ontology_path)
            .await?;

        // 2. Adaptively select execution strategy
        let context = self.infer_context(&input_data)?;
        let selected_pattern = self.pattern_selector.select_pattern(&context)?;

        tracing::info!(
            "Executing workflow {} from ontology {} with pattern {:?}",
            workflow_id,
            ontology_path,
            selected_pattern
        );

        // 3. Execute workflow
        let result = self
            .orchestrator
            .execute_workflow(&workflow_id, input_data)
            .await?;

        // 4. Record execution for learning
        let observation = crate::mape::Observation {
            receipt_id: result.receipt_id.clone(),
            sigma_id: result.sigma_id.clone(),
            ticks_used: result.ticks_used,
            guards_checked: vec![],
            guards_failed: vec![],
            timestamp: chrono::Utc::now(),
            metrics: std::collections::HashMap::new(),
        };

        self.pattern_selector
            .record_execution(selected_pattern, &observation);

        Ok(OntologyExecutionResult {
            workflow_id: result.workflow_id,
            output_data: result.output_data,
            ticks_used: result.ticks_used,
            receipt_id: result.receipt_id,
            sigma_id: result.sigma_id,
            pattern_used: selected_pattern,
            ontology_path: ontology_path.to_string(),
        })
    }

    /// Execute multiple workflows in batch from directory of ontologies
    pub async fn execute_batch(
        &mut self,
        ontology_dir: &str,
        inputs: Vec<serde_json::Value>,
    ) -> WorkflowResult<Vec<OntologyExecutionResult>> {
        let mut results = Vec::new();

        // Find all .ttl files in directory
        let paths = std::fs::read_dir(ontology_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s == "ttl")
                    .unwrap_or(false)
            })
            .map(|entry| entry.path())
            .collect::<Vec<_>>();

        for (path, input) in paths.iter().zip(inputs.iter()) {
            match self
                .execute_from_ontology(path.to_str().unwrap(), input.clone())
                .await
            {
                Ok(result) => results.push(result),
                Err(e) => {
                    tracing::error!("Failed to execute {}: {:?}", path.display(), e);
                }
            }
        }

        Ok(results)
    }

    /// Start continuous monitoring with MAPE-K
    pub async fn start_autonomic_monitoring(&self, interval_ms: u64) -> WorkflowResult<()> {
        self.orchestrator.start_autonomic_loop(interval_ms).await
    }

    /// Infer execution context from input data
    fn infer_context(&self, input: &serde_json::Value) -> WorkflowResult<PatternSelectionContext> {
        let data_size = match input {
            serde_json::Value::Array(arr) => arr.len(),
            serde_json::Value::Object(obj) => obj.len(),
            _ => 1,
        };

        // Heuristics for pattern selection
        let requires_parallelism = data_size > 10;
        let concurrency_level = if data_size > 50 {
            8
        } else if data_size > 10 {
            4
        } else {
            1
        };

        Ok(PatternSelectionContext {
            data_size,
            concurrency_level,
            requires_parallelism,
            requires_exclusive_choice: false,
            max_ticks: 8, // Chatman Constant
        })
    }

    /// Get execution statistics
    pub fn get_stats(&self) -> OntologyExecutorStats {
        let orchestrator_stats = self.orchestrator.get_statistics();

        OntologyExecutorStats {
            workflows_executed: orchestrator_stats.workflows_loaded,
            current_sigma: orchestrator_stats.current_sigma_id,
            total_receipts: orchestrator_stats.total_receipts,
            learned_patterns: self
                .pattern_selector
                .get_pattern_stats(&PatternId::SEQUENCE)
                .map(|s| s.total_executions as usize)
                .unwrap_or(0),
        }
    }
}

/// Result of ontology-driven execution
#[derive(Debug, Clone)]
pub struct OntologyExecutionResult {
    pub workflow_id: String,
    pub output_data: serde_json::Value,
    pub ticks_used: u32,
    pub receipt_id: String,
    pub sigma_id: String,
    pub pattern_used: PatternId,
    pub ontology_path: String,
}

/// Ontology executor statistics
#[derive(Debug, Clone)]
pub struct OntologyExecutorStats {
    pub workflows_executed: usize,
    pub current_sigma: String,
    pub total_receipts: usize,
    pub learned_patterns: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_ontology_executor_creation() {
        let temp_dir = TempDir::new().unwrap();
        let executor = OntologyExecutor::new(
            temp_dir.path().join("snapshots").to_str().unwrap(),
            temp_dir.path().join("receipts").to_str().unwrap(),
        )
        .unwrap();

        let stats = executor.get_stats();
        assert_eq!(stats.workflows_executed, 0);
    }
}
