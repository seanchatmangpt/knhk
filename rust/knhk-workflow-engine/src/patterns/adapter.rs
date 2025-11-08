//! Pattern adapter to bridge knhk-patterns with workflow engine

use crate::patterns::{
    PatternExecutionContext, PatternExecutionResult, PatternExecutor, PatternId,
};
use knhk_patterns::Pattern;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Adapter to bridge knhk-patterns::Pattern<T> with PatternExecutor
pub struct PatternAdapter {
    pattern: Arc<dyn Pattern<Value>>,
    pattern_id: PatternId,
}

impl PatternAdapter {
    /// Create new pattern adapter
    pub fn new(pattern: Arc<dyn Pattern<Value>>, pattern_id: PatternId) -> Self {
        Self {
            pattern,
            pattern_id,
        }
    }
}

impl PatternExecutor for PatternAdapter {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Convert context variables to JSON value
        let input = serde_json::json!(ctx.variables);

        // Execute pattern
        match self.pattern.execute(input) {
            Ok(results) => {
                // Extract result (first result for single output patterns)
                let result = results
                    .first()
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({}));

                // Convert result back to variables
                let mut variables = HashMap::new();
                if let Value::Object(map) = result {
                    for (k, v) in map {
                        variables.insert(k, v.to_string());
                    }
                }

                PatternExecutionResult {
                    success: true,
                    next_state: Some(format!("{}:completed", self.pattern_id)),
                    variables,
                }
            }
            Err(_e) => PatternExecutionResult {
                success: false,
                next_state: Some(format!("{}:failed", self.pattern_id)),
                variables: {
                    let mut vars = HashMap::new();
                    vars.insert("error".to_string(), e.to_string());
                    vars
                },
            },
        }
    }
}
