//! Advanced Control Flow Pattern: Discriminators (26-27)
use crate::patterns::{PatternExecutionContext, PatternExecutionResult, PatternExecutor};

pub struct BlockingDiscriminatorPattern;
impl PatternExecutor for BlockingDiscriminatorPattern {
    fn execute(&self, _ctx: &PatternExecutionContext) -> PatternExecutionResult {
        PatternExecutionResult {
            success: true,
            next_state: Some("pattern:26:blocking-discriminator:blocked".to_string()),
            next_activities: Vec::new(),
            variables: std::collections::HashMap::new(),
            updates: None,
            cancel_activities: Vec::new(),
            terminates: false,
        }
    }
}

pub struct CancellingDiscriminatorPattern;
impl PatternExecutor for CancellingDiscriminatorPattern {
    fn execute(&self, _ctx: &PatternExecutionContext) -> PatternExecutionResult {
        PatternExecutionResult {
            success: true,
            next_state: Some("pattern:27:cancelling-discriminator:cancelled".to_string()),
            next_activities: Vec::new(),
            variables: std::collections::HashMap::new(),
            updates: None,
            cancel_activities: vec!["blocked_branch".to_string()],
            terminates: false,
        }
    }
}
