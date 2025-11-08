//! Pattern registry and implementations for all 43 Van der Aalst patterns

pub mod adapter;
pub mod advanced;
pub mod advanced_control;
pub mod basic;
pub mod cancellation;
pub mod multiple_instance;
pub mod state_based;
pub mod trigger;

use std::collections::HashMap;

use crate::error::{WorkflowError, WorkflowResult};

/// Pattern identifier (1-43)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PatternId(pub u32);

impl PatternId {
    /// Create pattern ID (must be 1-43)
    pub fn new(id: u32) -> WorkflowResult<Self> {
        if id >= 1 && id <= 43 {
            Ok(Self(id))
        } else {
            Err(WorkflowError::PatternNotFound(id))
        }
    }
}

impl std::fmt::Display for PatternId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_'>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
/// Pattern identifier (String format: "pattern:1:sequence", etc.)

/// Pattern metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PatternMetadata {
    /// Pattern ID (1-43)
    pub id: PatternId,
    /// Pattern name
    pub name: String,
    /// Pattern category
    pub category: String,
    /// Average execution ticks
    pub avg_ticks: u32,
    /// Supports SIMD optimization
    pub supports_simd: bool,
    /// Implementation hint
    pub hint: String,
}

/// Pattern execution context
#[derive(Debug, Clone)]
pub struct PatternExecutionContext {
    /// Case ID
    pub case_id: crate::case::CaseId,
    /// Variables for pattern execution
    pub variables: HashMap<String, String>,
    /// Input data
    pub input: serde_json::Value,
}

/// Pattern execution result
#[derive(Debug, Clone)]
pub struct PatternExecutionResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Next state (if any)
    pub next_state: Option<String>,
    /// Output variables
    pub variables: HashMap<String, String>,
}

/// Pattern executor trait
pub trait PatternExecutor: Send + Sync {
    /// Execute pattern with context
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult;
}

/// Pattern registry
pub struct PatternRegistry {
    /// Registered patterns
    patterns: HashMap<PatternId, Box<dyn PatternExecutor>>,
}

impl PatternRegistry {
    /// Create new pattern registry
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
        }
    }

    /// Register a pattern
    pub fn register(&mut self, pattern_id: PatternId, executor: Box<dyn PatternExecutor>) {
        self.patterns.insert(pattern_id, executor);
    }

    /// Get pattern executor
    pub fn get(&self, pattern_id: &PatternId) -> Option<&dyn PatternExecutor> {
        self.patterns.get(pattern_id).map(|e| e.as_ref())
    }

    /// Get pattern executor
    pub fn get(&self, pattern_id: &PatternId) -> Option<&dyn PatternExecutor> {
        self.patterns.get(pattern_id).map(|e| e.as_ref())
    }

    /// Execute a pattern
    pub fn execute(
        &self,
        id: &PatternId,
        ctx: &PatternExecutionContext,
    ) -> Option<PatternExecutionResult> {
        self.patterns.get(id).map(|executor| executor.execute(ctx))
    }

    /// List all registered patterns
    pub fn list(&self) -> Vec<PatternId> {
        self.patterns.keys().cloned().collect()
    }

    /// List all registered pattern IDs (alias for list)
    pub fn list_patterns(&self) -> Vec<PatternId> {
        self.list()
    }
}

impl Default for PatternRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Register all 43 Van der Aalst patterns
pub fn register_all_patterns(registry: &mut PatternRegistry) {
    // Basic Control Flow (1-5)
    let (id, executor) = basic::create_sequence_pattern();
    registry.register(id, executor);
    let (id, executor) = basic::create_parallel_split_pattern();
    registry.register(id, executor);
    let (id, executor) = basic::create_synchronization_pattern();
    registry.register(id, executor);
    let (id, executor) = basic::create_exclusive_choice_pattern();
    registry.register(id, executor);
    let (id, executor) = basic::create_simple_merge_pattern();
    registry.register(id, executor);

    // Advanced Branching (6-11)
    let (id, executor) = advanced::create_multi_choice_pattern();
    registry.register(id, executor);
    let (id, executor) = advanced::create_structured_synchronizing_merge_pattern();
    registry.register(id, executor);
    let (id, executor) = advanced::create_multi_merge_pattern();
    registry.register(id, executor);
    let (id, executor) = advanced::create_discriminator_pattern();
    registry.register(id, executor);
    let (id, executor) = advanced::create_arbitrary_cycles_pattern();
    registry.register(id, executor);
    let (id, executor) = advanced::create_implicit_termination_pattern();
    registry.register(id, executor);

    // Multiple Instance (12-15)
    let (id, executor) = multiple_instance::create_pattern_12();
    registry.register(id, executor);
    let (id, executor) = multiple_instance::create_pattern_13();
    registry.register(id, executor);
    let (id, executor) = multiple_instance::create_pattern_14();
    registry.register(id, executor);
    let (id, executor) = multiple_instance::create_pattern_15();
    registry.register(id, executor);

    // State-Based (16-18)
    let (id, executor) = state_based::create_deferred_choice_pattern();
    registry.register(id, executor);
    let (id, executor) = state_based::create_pattern_17();
    registry.register(id, executor);
    let (id, executor) = state_based::create_pattern_18();
    registry.register(id, executor);

    // Cancellation (19-25)
    let (id, executor) = cancellation::create_pattern_19();
    registry.register(id, executor);
    let (id, executor) = cancellation::create_timeout_pattern();
    registry.register(id, executor);
    let (id, executor) = cancellation::create_cancellation_pattern();
    registry.register(id, executor);
    let (id, executor) = cancellation::create_pattern_22();
    registry.register(id, executor);
    let (id, executor) = cancellation::create_pattern_23();
    registry.register(id, executor);
    let (id, executor) = cancellation::create_pattern_24();
    registry.register(id, executor);
    let (id, executor) = cancellation::create_pattern_25();
    registry.register(id, executor);

    // Advanced Control (26-39)
    let (id, executor) = advanced_control::create_pattern_26();
    registry.register(id, executor);
    let (id, executor) = advanced_control::create_pattern_27();
    registry.register(id, executor);
    let (id, executor) = advanced_control::create_pattern_28();
    registry.register(id, executor);
    let (id, executor) = advanced_control::create_pattern_29();
    registry.register(id, executor);
    let (id, executor) = advanced_control::create_pattern_30();
    registry.register(id, executor);
    let (id, executor) = advanced_control::create_pattern_31();
    registry.register(id, executor);
    let (id, executor) = advanced_control::create_pattern_32();
    registry.register(id, executor);
    let (id, executor) = advanced_control::create_pattern_33();
    registry.register(id, executor);
    let (id, executor) = advanced_control::create_pattern_34();
    registry.register(id, executor);
    let (id, executor) = advanced_control::create_pattern_35();
    registry.register(id, executor);
    let (id, executor) = advanced_control::create_pattern_36();
    registry.register(id, executor);
    let (id, executor) = advanced_control::create_pattern_37();
    registry.register(id, executor);
    let (id, executor) = advanced_control::create_pattern_38();
    registry.register(id, executor);
    let (id, executor) = advanced_control::create_pattern_39();
    registry.register(id, executor);

    // Trigger (40-43)
    let (id, executor) = trigger::create_pattern_40();
    registry.register(id, executor);
    let (id, executor) = trigger::create_pattern_41();
    registry.register(id, executor);
    let (id, executor) = trigger::create_pattern_42();
    registry.register(id, executor);
    let (id, executor) = trigger::create_pattern_43();
    registry.register(id, executor);
}
