//! Pattern registry and implementations for all 43 Van der Aalst patterns

use crate::error::{WorkflowError, WorkflowResult};
use std::collections::HashMap;

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

/// Pattern category
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PatternCategory {
    /// Basic Control Flow (1-5)
    BasicControlFlow,
    /// Advanced Branching (6-11)
    AdvancedBranching,
    /// Multiple Instance (12-15)
    MultipleInstance,
    /// State-Based (16-18)
    StateBased,
    /// Cancellation (19-25)
    Cancellation,
    /// Advanced Patterns (26-39)
    Advanced,
    /// Trigger Patterns (40-43)
    Trigger,
}

/// Pattern metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PatternMetadata {
    /// Pattern ID (1-43)
    pub id: PatternId,
    /// Pattern name
    pub name: String,
    /// Pattern category
    pub category: PatternCategory,
    /// Average execution ticks
    pub avg_ticks: u32,
    /// Supports SIMD optimization
    pub supports_simd: bool,
    /// Implementation hint
    pub hint: String,
}

/// Pattern executor trait
pub trait PatternExecutor: Send + Sync {
    /// Execute the pattern
    fn execute(&self, context: &PatternExecutionContext) -> WorkflowResult<PatternExecutionResult>;
    
    /// Get pattern metadata
    fn metadata(&self) -> &PatternMetadata;
}

/// Pattern execution context
#[derive(Debug, Clone)]
pub struct PatternExecutionContext {
    /// Case data
    pub case_data: serde_json::Value,
    /// Task inputs
    pub inputs: Vec<serde_json::Value>,
    /// Configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Pattern execution result
#[derive(Debug, Clone)]
pub struct PatternExecutionResult {
    /// Output values
    pub outputs: Vec<serde_json::Value>,
    /// Execution ticks consumed
    pub ticks: u32,
    /// Success flag
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Pattern registry
pub struct PatternRegistry {
    patterns: HashMap<PatternId, Box<dyn PatternExecutor>>,
}

impl PatternRegistry {
    /// Create a new pattern registry
    pub fn new() -> Self {
        let mut registry = Self {
            patterns: HashMap::new(),
        };
        
        // Register all 43 patterns
        registry.register_all_patterns();
        
        registry
    }

    /// Register all 43 patterns
    fn register_all_patterns(&mut self) {
        // Basic Control Flow (1-5)
        self.register_pattern(1, PatternCategory::BasicControlFlow, "Sequence", 1, false, "Direct flow transition");
        self.register_pattern(2, PatternCategory::BasicControlFlow, "Parallel Split", 2, true, "SIMD branch initialization");
        self.register_pattern(3, PatternCategory::BasicControlFlow, "Synchronization", 3, true, "Atomic counter");
        self.register_pattern(4, PatternCategory::BasicControlFlow, "Exclusive Choice", 2, false, "Branch prediction");
        self.register_pattern(5, PatternCategory::BasicControlFlow, "Simple Merge", 1, false, "Direct merge");

        // Advanced Branching (6-11)
        self.register_pattern(6, PatternCategory::AdvancedBranching, "Multi-Choice", 3, true, "Bitmap selection");
        self.register_pattern(7, PatternCategory::AdvancedBranching, "Structured Synchronizing Merge", 4, false, "Track split context");
        self.register_pattern(8, PatternCategory::AdvancedBranching, "Multi-Merge", 2, false, "Queue tokens");
        self.register_pattern(9, PatternCategory::AdvancedBranching, "Discriminator", 3, false, "First-arrival detection");
        self.register_pattern(10, PatternCategory::AdvancedBranching, "Arbitrary Cycles", 2, false, "Loop counter");
        self.register_pattern(11, PatternCategory::AdvancedBranching, "Implicit Termination", 4, false, "Global counter");

        // Multiple Instance (12-15)
        self.register_pattern(12, PatternCategory::MultipleInstance, "MI Without Sync", 2, true, "Fire-and-forget");
        self.register_pattern(13, PatternCategory::MultipleInstance, "MI With Design-Time Knowledge", 3, true, "Pre-allocated pool");
        self.register_pattern(14, PatternCategory::MultipleInstance, "MI With Runtime Knowledge", 4, true, "Dynamic allocation");
        self.register_pattern(15, PatternCategory::MultipleInstance, "MI Without Runtime Knowledge", 5, false, "Growing pool");

        // State-Based (16-18)
        self.register_pattern(16, PatternCategory::StateBased, "Deferred Choice", 3, false, "Event-driven");
        self.register_pattern(17, PatternCategory::StateBased, "Interleaved Parallel Routing", 6, false, "Mutex access");
        self.register_pattern(18, PatternCategory::StateBased, "Milestone", 4, false, "State checkpoint");

        // Cancellation (19-25)
        self.register_pattern(19, PatternCategory::Cancellation, "Cancel Activity", 2, false, "Task cancellation");
        self.register_pattern(20, PatternCategory::Cancellation, "Cancel Case", 1, false, "Case cancellation");
        self.register_pattern(21, PatternCategory::Cancellation, "Cancel Region", 3, false, "Region cancellation");
        self.register_pattern(22, PatternCategory::Cancellation, "Cancel MI Activity", 4, false, "MI cancellation");
        self.register_pattern(23, PatternCategory::Cancellation, "Complete MI Activity", 5, false, "MI completion");
        self.register_pattern(24, PatternCategory::Cancellation, "Blocking Discriminator", 3, false, "Blocking wait");
        self.register_pattern(25, PatternCategory::Cancellation, "Cancelling Discriminator", 4, false, "Cancel others");

        // Advanced Patterns (26-39)
        for id in 26..=39 {
            self.register_pattern(id, PatternCategory::Advanced, &format!("Pattern {}", id), 3, false, "Advanced pattern");
        }

        // Trigger Patterns (40-43)
        self.register_pattern(40, PatternCategory::Trigger, "Trigger Pattern 40", 2, false, "Event trigger");
        self.register_pattern(41, PatternCategory::Trigger, "Trigger Pattern 41", 2, false, "Event trigger");
        self.register_pattern(42, PatternCategory::Trigger, "Trigger Pattern 42", 2, false, "Event trigger");
        self.register_pattern(43, PatternCategory::Trigger, "Trigger Pattern 43", 2, false, "Event trigger");
    }

    /// Register a pattern (placeholder implementation)
    fn register_pattern(
        &mut self,
        id: u32,
        category: PatternCategory,
        name: &str,
        avg_ticks: u32,
        supports_simd: bool,
        hint: &str,
    ) {
        if let Ok(pattern_id) = PatternId::new(id) {
            let metadata = PatternMetadata {
                id: pattern_id,
                name: name.to_string(),
                category,
                avg_ticks,
                supports_simd,
                hint: hint.to_string(),
            };
            
            // Create placeholder executor
            let executor: Box<dyn PatternExecutor> = Box::new(PlaceholderPatternExecutor { metadata });
            self.patterns.insert(pattern_id, executor);
        }
    }

    /// Get pattern executor
    pub fn get_executor(&self, id: PatternId) -> WorkflowResult<&dyn PatternExecutor> {
        self.patterns
            .get(&id)
            .map(|e| e.as_ref())
            .ok_or_else(|| WorkflowError::PatternNotFound(id.0))
    }

    /// Get pattern metadata
    pub fn get_metadata(&self, id: PatternId) -> WorkflowResult<PatternMetadata> {
        self.get_executor(id).map(|e| e.metadata().clone())
    }

    /// List all registered patterns
    pub fn list_patterns(&self) -> Vec<PatternMetadata> {
        self.patterns
            .values()
            .map(|e| e.metadata().clone())
            .collect()
    }
}

impl Default for PatternRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Placeholder pattern executor (to be replaced with real implementations)
struct PlaceholderPatternExecutor {
    metadata: PatternMetadata,
}

impl PatternExecutor for PlaceholderPatternExecutor {
    fn execute(&self, _context: &PatternExecutionContext) -> WorkflowResult<PatternExecutionResult> {
        // Placeholder: return success with empty outputs
        Ok(PatternExecutionResult {
            outputs: vec![],
            ticks: self.metadata.avg_ticks,
            success: true,
            error: None,
        })
    }

    fn metadata(&self) -> &PatternMetadata {
        &self.metadata
    }
}

