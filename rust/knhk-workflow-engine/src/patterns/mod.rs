//! Pattern registry and implementations for all 43 Van der Aalst patterns

pub mod adapter;
pub mod advanced;
pub mod advanced_control;
pub mod basic;
pub mod cancellation;
pub mod combinatorics;
pub mod joins;
pub mod mi;
pub mod multiple_instance;
pub mod permutations;
pub mod rdf;
pub mod state_based;
pub mod trigger;
pub mod validation;

use std::collections::HashMap;

use crate::error::{WorkflowError, WorkflowResult};

/// Pattern identifier (1-43)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PatternId(pub u32);

impl PatternId {
    /// Create pattern ID (must be 1-43)
    pub fn new(id: u32) -> WorkflowResult<Self> {
        if (1..=43).contains(&id) {
            Ok(Self(id))
        } else {
            Err(WorkflowError::PatternNotFound(id))
        }
    }

    // Pattern constants for common patterns
    /// Pattern 1: Sequence
    pub const SEQUENCE: PatternId = PatternId(1);
    /// Pattern 2: Parallel Split
    pub const PARALLEL_SPLIT: PatternId = PatternId(2);
    /// Pattern 4: Exclusive Choice
    pub const EXCLUSIVE_CHOICE: PatternId = PatternId(4);
    /// Pattern 6: Multi-Choice
    pub const MULTI_CHOICE: PatternId = PatternId(6);
    /// Pattern 16: Deferred Choice
    pub const DEFERRED_CHOICE: PatternId = PatternId(16);
    /// Pattern 12: Multiple Instances With Design-Time Knowledge
    pub const MULTIPLE_INSTANCES_WITH_DESIGN_TIME_KNOWLEDGE: PatternId = PatternId(12);
}

impl std::fmt::Display for PatternId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pattern:{}", self.0)
    }
}

/// Pattern execution context
#[derive(Debug, Clone, Default)]
pub struct PatternExecutionContext {
    /// Case ID
    pub case_id: crate::case::CaseId,
    /// Workflow ID
    pub workflow_id: crate::parser::WorkflowSpecId,
    /// Variables for pattern execution
    pub variables: HashMap<String, String>,
    /// Upstream completions present for a join node (edge ids)
    pub arrived_from: std::collections::HashSet<String>,
    /// Region/scope id (for cancel & MI)
    pub scope_id: String,
}

/// Pattern execution result
#[derive(Debug, Clone)]
pub struct PatternExecutionResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Next state (if any)
    pub next_state: Option<String>,
    /// Next activities to schedule
    pub next_activities: Vec<String>,
    /// Output variables
    pub variables: HashMap<String, String>,
    /// State updates (for joins, MI tracking)
    pub updates: Option<serde_json::Value>,
    /// Activities to cancel
    pub cancel_activities: Vec<String>,
    /// Whether this terminates the workflow
    pub terminates: bool,
}

impl PatternExecutionResult {
    /// Create a successful result with next activities
    pub fn ok(next_activities: Vec<String>) -> Self {
        Self {
            success: true,
            next_state: None,
            next_activities,
            variables: HashMap::new(),
            updates: None,
            cancel_activities: Vec::new(),
            terminates: false,
        }
    }

    /// Create a result with updates (for joins, MI)
    pub fn with_updates(next_activities: Vec<String>, updates: serde_json::Value) -> Self {
        Self {
            success: true,
            next_state: None,
            next_activities,
            variables: HashMap::new(),
            updates: Some(updates),
            cancel_activities: Vec::new(),
            terminates: false,
        }
    }

    /// Create a cancellation result
    pub fn cancel_then(next_activities: Vec<String>, cancel_activities: Vec<String>) -> Self {
        Self {
            success: true,
            next_state: None,
            next_activities,
            variables: HashMap::new(),
            updates: None,
            cancel_activities,
            terminates: false,
        }
    }

    /// Create a termination result
    pub fn terminate() -> Self {
        Self {
            success: true,
            next_state: None,
            next_activities: Vec::new(),
            variables: HashMap::new(),
            updates: None,
            cancel_activities: Vec::new(),
            terminates: true,
        }
    }
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

impl std::fmt::Debug for PatternRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PatternRegistry")
            .field("pattern_count", &self.patterns.len())
            .field("pattern_ids", &self.patterns.keys().collect::<Vec<_>>())
            .finish()
    }
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

    /// Check if a pattern is registered
    pub fn has_pattern(&self, pattern_id: &PatternId) -> bool {
        self.patterns.contains_key(pattern_id)
    }
}

impl Default for PatternRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait extension for registering all patterns
pub trait RegisterAllExt {
    /// Register all 43 Van der Aalst workflow patterns
    fn register_all_patterns(&mut self);
}

impl RegisterAllExt for PatternRegistry {
    fn register_all_patterns(&mut self) {
        register_all_patterns(self);
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

// RDF support for patterns
#[cfg(feature = "rdf")]
pub use rdf::{
    deserialize_context_from_rdf, deserialize_metadata_from_rdf, deserialize_result_from_rdf,
};
pub use rdf::{
    get_all_pattern_metadata, load_all_metadata_from_rdf, serialize_context_to_rdf,
    serialize_metadata_to_rdf, serialize_result_to_rdf, PatternMetadata, WORKFLOW_PATTERN_NS,
    YAWL_NS,
};
