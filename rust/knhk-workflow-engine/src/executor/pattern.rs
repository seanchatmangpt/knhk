//! Pattern execution methods

use crate::error::{WorkflowError, WorkflowResult};
use crate::integration::fortune5::RuntimeClass;
use crate::patterns::{PatternExecutionContext, PatternExecutionResult, PatternId};
use std::time::Instant;

use super::WorkflowEngine;

impl WorkflowEngine {
    /// Execute a pattern
    pub async fn execute_pattern(
        &self,
        pattern_id: PatternId,
        context: PatternExecutionContext,
    ) -> WorkflowResult<PatternExecutionResult> {
        let start_time = Instant::now();

        // Check Fortune 5 promotion gate if enabled
        if let Some(ref fortune5) = self.fortune5_integration {
            if !fortune5.check_promotion_gate().await? {
                return Err(WorkflowError::Validation(
                    "Promotion gate blocked execution".to_string(),
                ));
            }
        }

        // Execute pattern
        let result = self
            .pattern_registry
            .execute(&pattern_id, &context)
            .ok_or_else(|| {
                WorkflowError::InvalidSpecification(format!("Pattern {} not found", pattern_id))
            })?;

        // LAW: A = μ(O) - Generate receipt proving hash(A) = hash(μ(O))
        // Extract observations (O) from case state
        if let Ok(case) = self.get_case(context.case_id).await {
            let observations = crate::executor::WorkflowEngine::extract_observations(&case);

            // Extract actions (A) from pattern result
            let actions = crate::executor::WorkflowEngine::extract_actions(&result);

            // Generate receipt with provenance verification
            if let Ok(receipt) = crate::executor::WorkflowEngine::generate_receipt(
                self,
                context.case_id,
                context.workflow_id,
                &actions,
                &observations,
            ) {
                // Store receipt for provenance tracking
                if let Some(_lockchain) = self.lockchain_integration.as_ref() {
                    let receipt_bytes = serde_json::to_vec(&receipt).map_err(|e| {
                        WorkflowError::Internal(format!("Failed to serialize receipt: {}", e))
                    })?;
                    let store = self.state_store.read().await;
                    let _ = (*store).append_receipt(&receipt.id, &receipt_bytes);
                }
            }
        }

        // Reflex bridge: promote pattern subgraphs to hot path if promotable
        let mut reflex_bridge = crate::reflex::ReflexBridge::new();
        if let Ok(workflow_spec) = self.get_workflow(context.workflow_id).await {
            if let Ok(promotable_segments) = reflex_bridge.bind_hot_segments(&workflow_spec) {
                // Check if this pattern is in a promotable segment
                for segment in &promotable_segments {
                    if segment.pattern_ids.contains(&(pattern_id.0 as u8))
                        && segment.hot_executor_bound
                    {
                        // Pattern is promotable - would execute via hot path in production
                        // For now, just record that promotion was considered
                    }
                }
            }
        }

        // Record SLO metrics if Fortune 5 is enabled
        if let Some(ref fortune5) = self.fortune5_integration {
            let duration_ns = start_time.elapsed().as_nanos() as u64;
            // Determine runtime class based on pattern ID and duration
            let runtime_class = if duration_ns <= 2_000 {
                RuntimeClass::R1 // Hot path
            } else if duration_ns <= 1_000_000 {
                RuntimeClass::W1 // Warm path
            } else {
                RuntimeClass::C1 // Cold path
            };
            fortune5.record_slo_metric(runtime_class, duration_ns).await;
        }

        Ok(result)
    }
}
