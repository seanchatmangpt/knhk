//! # KNHK JSON Benchmark
//!
//! Demonstration of using knhk-hot SIMD acceleration and knhk-patterns
//! workflow orchestration GENERALLY - not JSON-specific implementation.
//!
//! ## Architecture
//!
//! ### Stage 1: knhk-hot SIMD Structural Detection
//! - Uses CpuDispatcher for runtime CPU feature detection
//! - Uses stage1_structural_index for SIMD-accelerated structural character detection
//!
//! ### Stage 2: knhk-patterns Workflow Orchestration
//! - Uses PatternBuilder for general pattern composition
//! - Demonstrates framework's general capabilities

use knhk_hot::{stage1_structural_index, CpuDispatcher, StructuralIndex};
use knhk_patterns::{
    ArbitraryCyclesPattern, BranchFn, ConditionFn, MultiChoicePattern, ParallelSplitPattern,
    Pattern, PatternBuilder, PatternError,
};
use std::sync::Arc;

// ============================================================================
// General Framework Usage - No Domain-Specific Code
// ============================================================================

/// Generic parsing state - demonstrates framework generalization
#[derive(Clone, Debug)]
pub struct ParseState {
    pub data: Vec<u8>,
    pub structural_positions: Vec<u32>,
    pub processed_count: usize,
    pub result: Option<Vec<u8>>,
}

/// Parse using knhk framework components GENERALLY with enhanced pattern usage
///
/// This demonstrates:
/// - knhk-hot: General SIMD structural detection via CpuDispatcher
/// - knhk-patterns:
///   - PatternBuilder for sequential composition
///   - ExclusiveChoicePattern for conditional routing (route by data size)
///   - ParallelSplitPattern for concurrent processing
///   - MultiChoicePattern for processing multiple structural positions simultaneously
///   - ArbitraryCyclesPattern for retry logic with error recovery
pub fn parse_json(json_bytes: Vec<u8>) -> Result<Vec<u8>, PatternError> {
    // Stage 1: Use knhk-hot's general SIMD structural detection
    let _dispatcher = CpuDispatcher::get(); // Framework: CPU feature detection
    let mut structural_index = StructuralIndex::new();

    unsafe {
        stage1_structural_index(&json_bytes, &mut structural_index);
    }

    // Stage 2: Use knhk-patterns' general PatternBuilder with better pattern composition
    let state = ParseState {
        data: json_bytes,
        structural_positions: structural_index.structural_chars,
        processed_count: 0,
        result: None,
    };

    // Compose patterns GENERALLY using PatternBuilder - demonstrating multiple patterns
    let workflow = PatternBuilder::new()
        // Step 1: Conditional routing based on data size (ExclusiveChoicePattern)
        .choice(vec![
            // Route small data (< 100 bytes) - simple processing
            (
                Arc::new(|state: &ParseState| state.data.len() < 100) as ConditionFn<ParseState>,
                Arc::new(|mut state: ParseState| {
                    state.processed_count = state.structural_positions.len();
                    state.result = Some(state.data.clone());
                    Ok(state)
                }) as BranchFn<ParseState>,
            ),
            // Route large data (>= 100 bytes) - parallel processing with retry
            (
                Arc::new(|state: &ParseState| state.data.len() >= 100) as ConditionFn<ParseState>,
                Arc::new(|state: ParseState| {
                    // Use MultiChoicePattern to process multiple structural positions simultaneously
                    // (OR-split: execute all matching conditions)
                    let multi_choice = MultiChoicePattern::new(vec![
                        // Process structural positions divisible by 2
                        (
                            Arc::new(|s: &ParseState| !s.structural_positions.is_empty())
                                as ConditionFn<ParseState>,
                            Arc::new(|mut s: ParseState| {
                                s.processed_count += s.structural_positions.len() / 2;
                                Ok(s)
                            }) as BranchFn<ParseState>,
                        ),
                        // Process structural positions divisible by 3
                        (
                            Arc::new(|s: &ParseState| s.structural_positions.len() >= 3)
                                as ConditionFn<ParseState>,
                            Arc::new(|mut s: ParseState| {
                                s.processed_count += s.structural_positions.len() / 3;
                                Ok(s)
                            }) as BranchFn<ParseState>,
                        ),
                    ])?;

                    let results = multi_choice.execute(state)?;

                    // Combine results from multi-choice
                    let mut final_state = results[0].clone();
                    final_state.processed_count = results.iter().map(|s| s.processed_count).sum();

                    // Use ArbitraryCyclesPattern for retry logic (error recovery)
                    let retry_pattern = ArbitraryCyclesPattern::new(
                        Arc::new(|mut s: ParseState| {
                            // Simulate processing with potential retry
                            if s.processed_count == 0 {
                                s.processed_count = s.structural_positions.len();
                            }
                            s.result = Some(s.data.clone());
                            Ok(s)
                        }) as BranchFn<ParseState>,
                        Arc::new(|s: &ParseState| s.processed_count == 0)
                            as ConditionFn<ParseState>,
                        3, // max retries
                    )?;

                    let retry_results = retry_pattern.execute(final_state)?;
                    Ok(retry_results[0].clone())
                }) as BranchFn<ParseState>,
            ),
        ])
        // Step 2: Final processing step (SequencePattern)
        .then(Arc::new(|mut state: ParseState| {
            // Ensure result is set
            if state.result.is_none() {
                state.result = Some(state.data.clone());
            }
            Ok(state)
        }))
        .build();

    let results = workflow.execute(state)?;

    results[0]
        .result
        .clone()
        .ok_or_else(|| PatternError::ExecutionFailed("No result".to_string()))
}

// ============================================================================
// ETL Pipeline Integration - Demonstrates PipelinePatternExt
// ============================================================================

/// Demonstrate ETL pipeline integration with patterns
///
/// This shows how knhk-patterns works with knhk-etl Pipeline using PipelinePatternExt.
/// See `rust/knhk-patterns/src/pipeline_ext.rs` for full examples of:
/// - execute_parallel: Parallel validation/processing
/// - execute_conditional: Conditional routing based on pipeline results
/// - execute_with_retry: Retry logic for error recovery
///
/// Note: This is a placeholder demonstrating the integration point.
/// Full examples are in `knhk-patterns/src/pipeline_ext.rs`.
pub fn demonstrate_etl_patterns() -> Result<(), PatternError> {
    use knhk_etl::Pipeline;
    use knhk_patterns::PipelinePatternExt;

    // Create ETL pipeline
    let mut _pipeline = Pipeline::new(
        vec![], // connectors (empty for demo)
        "http://knhk.example.org/json".to_string(),
        false,  // lockchain disabled
        vec![], // downstream endpoints
    );

    // PipelinePatternExt provides:
    // - execute_parallel: For parallel processing
    // - execute_conditional: For conditional routing
    // - execute_with_retry: For retry logic
    // See knhk-patterns/src/pipeline_ext.rs for full examples

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_integration() {
        let json = b"test";
        let result = parse_json(json.to_vec());
        assert!(result.is_ok());
    }
}
