//! Van der Aalst Pattern-Level OTEL Integration
//!
//! Provides pattern-specific OTEL spans for all 43 Van der Aalst workflow patterns.
//! Each pattern emits its own span with pattern-specific attributes.

use crate::case::CaseId;
use crate::error::WorkflowResult;
use crate::integration::OtelIntegration;
use crate::patterns::PatternId;
use knhk_otel::{SpanContext, SpanStatus};
use std::collections::HashMap;

/// Pattern name mapping for all 43 Van der Aalst patterns
pub fn get_pattern_name(pattern_id: u32) -> &'static str {
    match pattern_id {
        1 => "Sequence",
        2 => "Parallel Split",
        3 => "Synchronization",
        4 => "Exclusive Choice",
        5 => "Simple Merge",
        6 => "Multi-Choice",
        7 => "Structured Synchronizing Merge",
        8 => "Multi-Merge",
        9 => "Discriminator",
        10 => "Arbitrary Cycles",
        11 => "Implicit Termination",
        12 => "MI Without Sync",
        13 => "MI With Design-Time Knowledge",
        14 => "MI With Runtime Knowledge",
        15 => "MI Without Runtime Knowledge",
        16 => "Deferred Choice",
        17 => "Interleaved Parallel Routing",
        18 => "Milestone",
        19 => "Cancel Activity",
        20 => "Cancel Case",
        21 => "Cancel Region",
        22 => "Cancel MI Activity",
        23 => "Complete MI Activity",
        24 => "Blocking Discriminator",
        25 => "Cancelling Discriminator",
        26..=39 => "Advanced Control",
        40..=43 => "Trigger",
        _ => "Unknown Pattern",
    }
}

/// Pattern category mapping
pub fn get_pattern_category(pattern_id: u32) -> &'static str {
    match pattern_id {
        1..=5 => "Basic Control Flow",
        6..=11 => "Advanced Branching",
        12..=15 => "Multiple Instance",
        16..=18 => "State-Based",
        19..=25 => "Cancellation",
        26..=39 => "Advanced Control",
        40..=43 => "Trigger",
        _ => "Unknown",
    }
}

/// Pattern-specific span name
pub fn get_pattern_span_name(pattern_id: u32) -> String {
    let base_name = match pattern_id {
        1 => "pattern.sequence",
        2 => "pattern.parallel_split",
        3 => "pattern.synchronization",
        4 => "pattern.exclusive_choice",
        5 => "pattern.simple_merge",
        6 => "pattern.multi_choice",
        12 => "pattern.mi_without_sync",
        13 => "pattern.mi_design_time",
        14 => "pattern.mi_runtime",
        15 => "pattern.mi_no_runtime",
        16 => "pattern.deferred_choice",
        19 => "pattern.cancel_activity",
        _ => "pattern.execute",
    };
    format!("knhk.workflow_engine.{}", base_name)
}

/// Pattern-level OTEL helper
pub struct PatternOtelHelper;

impl PatternOtelHelper {
    /// Start a pattern-specific span
    pub async fn start_pattern_span(
        otel: &OtelIntegration,
        pattern_id: &PatternId,
        case_id: &CaseId,
        additional_attributes: Option<HashMap<String, String>>,
    ) -> WorkflowResult<Option<SpanContext>> {
        let _span_name = get_pattern_span_name(pattern_id.0);
        let pattern_name = get_pattern_name(pattern_id.0);
        let pattern_category = get_pattern_category(pattern_id.0);

        // Start the base pattern span
        let span_ctx = otel.start_execute_pattern_span(pattern_id, case_id, None).await?;

        if let Some(ref span) = span_ctx {
            // Add pattern-specific attributes
            otel.add_attribute(
                span.clone(),
                "knhk.workflow_engine.pattern_name".to_string(),
                pattern_name.to_string(),
            )
            .await?;
            otel.add_attribute(
                span.clone(),
                "knhk.workflow_engine.pattern_category".to_string(),
                pattern_category.to_string(),
            )
            .await?;

            // Add additional attributes if provided
            if let Some(attrs) = additional_attributes {
                for (key, value) in attrs {
                    otel.add_attribute(span.clone(), key, value).await?;
                }
            }
        }

        Ok(span_ctx)
    }

    /// Start a pattern span with pattern-specific attributes
    pub async fn start_pattern_span_with_attrs(
        otel: &OtelIntegration,
        pattern_id: &PatternId,
        case_id: &CaseId,
        pattern_attrs: PatternAttributes,
    ) -> WorkflowResult<Option<SpanContext>> {
        let mut attrs = HashMap::new();

        // Add pattern-specific attributes based on pattern type
        match pattern_id.0 {
            2 | 3 => {
                // Parallel Split / Synchronization
                if let Some(branch_count) = pattern_attrs.branch_count {
                    attrs.insert(
                        "knhk.workflow_engine.branch_count".to_string(),
                        branch_count.to_string(),
                    );
                }
                if let Some(sync_count) = pattern_attrs.synchronized_count {
                    attrs.insert(
                        "knhk.workflow_engine.synchronized_count".to_string(),
                        sync_count.to_string(),
                    );
                }
            }
            4 | 6 | 16 => {
                // Exclusive Choice / Multi-Choice / Deferred Choice
                if let Some(ref chosen) = pattern_attrs.chosen_branch {
                    attrs.insert(
                        "knhk.workflow_engine.chosen_branch".to_string(),
                        chosen.clone(),
                    );
                }
                if let Some(ref chosen_branches) = pattern_attrs.chosen_branches {
                    attrs.insert(
                        "knhk.workflow_engine.chosen_branches".to_string(),
                        chosen_branches.clone(),
                    );
                }
            }
            12..=15 => {
                // Multiple Instance patterns
                if let Some(instance_count) = pattern_attrs.instance_count {
                    attrs.insert(
                        "knhk.workflow_engine.instance_count".to_string(),
                        instance_count.to_string(),
                    );
                }
            }
            19..=25 => {
                // Cancellation patterns
                if let Some(ref cancelled) = pattern_attrs.cancelled_activity {
                    attrs.insert(
                        "knhk.workflow_engine.cancelled_activity".to_string(),
                        cancelled.clone(),
                    );
                }
            }
            _ => {}
        }

        Self::start_pattern_span(otel, pattern_id, case_id, Some(attrs)).await
    }

    /// End a pattern span with success status
    pub async fn end_pattern_span(
        otel: &OtelIntegration,
        span_ctx: SpanContext,
        success: bool,
    ) -> WorkflowResult<()> {
        let status = if success {
            SpanStatus::Ok
        } else {
            SpanStatus::Error
        };
        otel.end_span(span_ctx, status).await
    }
}

/// Pattern-specific attributes
#[derive(Debug, Clone, Default)]
pub struct PatternAttributes {
    /// Branch count (for parallel patterns)
    pub branch_count: Option<u32>,
    /// Synchronized count (for synchronization patterns)
    pub synchronized_count: Option<u32>,
    /// Chosen branch (for choice patterns)
    pub chosen_branch: Option<String>,
    /// Chosen branches (for multi-choice patterns)
    pub chosen_branches: Option<String>,
    /// Instance count (for MI patterns)
    pub instance_count: Option<u32>,
    /// Cancelled activity (for cancellation patterns)
    pub cancelled_activity: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_name_mapping() {
        assert_eq!(get_pattern_name(1), "Sequence");
        assert_eq!(get_pattern_name(2), "Parallel Split");
        assert_eq!(get_pattern_name(12), "MI Without Sync");
        assert_eq!(get_pattern_name(16), "Deferred Choice");
        assert_eq!(get_pattern_name(19), "Cancel Activity");
    }

    #[test]
    fn test_pattern_category_mapping() {
        assert_eq!(get_pattern_category(1), "Basic Control Flow");
        assert_eq!(get_pattern_category(6), "Advanced Branching");
        assert_eq!(get_pattern_category(12), "Multiple Instance");
        assert_eq!(get_pattern_category(16), "State-Based");
        assert_eq!(get_pattern_category(19), "Cancellation");
    }

    #[test]
    fn test_pattern_span_name() {
        assert_eq!(
            get_pattern_span_name(1),
            "knhk.workflow_engine.pattern.sequence"
        );
        assert_eq!(
            get_pattern_span_name(2),
            "knhk.workflow_engine.pattern.parallel_split"
        );
        assert_eq!(
            get_pattern_span_name(12),
            "knhk.workflow_engine.pattern.mi_without_sync"
        );
    }
}
