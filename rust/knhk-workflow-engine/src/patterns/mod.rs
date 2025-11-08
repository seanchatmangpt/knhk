//! Workflow pattern registry and execution

use std::collections::HashMap;

/// Pattern identifier
pub type PatternId = String;

/// Pattern execution context
pub struct PatternExecutionContext {
    pub case_id: String,
    pub workflow_id: String,
    pub variables: HashMap<String, String>,
}

/// Pattern execution result
pub struct PatternExecutionResult {
    pub success: bool,
    pub next_state: Option<String>,
    pub variables: HashMap<String, String>,
}

/// Pattern registry for all 43 Van der Aalst patterns
pub struct PatternRegistry {
    patterns: HashMap<PatternId, Box<dyn PatternExecutor>>,
}

/// Trait for pattern executors
pub trait PatternExecutor: Send + Sync {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult;
}

impl PatternRegistry {
    /// Create new pattern registry
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
        }
    }

    /// Register a pattern
    pub fn register(&mut self, id: PatternId, executor: Box<dyn PatternExecutor>) {
        self.patterns.insert(id, executor);
    }

    /// Execute a pattern
    pub fn execute(
        &self,
        id: &PatternId,
        ctx: &PatternExecutionContext,
    ) -> Option<PatternExecutionResult> {
        self.patterns.get(id).map(|executor| executor.execute(ctx))
    }
}

impl Default for PatternRegistry {
    fn default() -> Self {
        Self::new()
    }
}
