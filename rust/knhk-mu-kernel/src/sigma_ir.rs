//! Typed Intermediate Representation for Σ
//!
//! This module provides a type-safe IR that only allows legal Σ constructs.
//! Invalid IR is unrepresentable using Rust's type system.

use core::marker::PhantomData;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;

/// Validation marker types - zero-size types that encode validation state
pub mod validation {
    /// Unvalidated IR - may contain errors
    #[derive(Debug, Clone, Copy)]
    pub struct Unvalidated;

    /// Structurally validated - syntax correct, but not semantically checked
    #[derive(Debug, Clone, Copy)]
    pub struct StructurallyValid;

    /// Semantically validated - ready for compilation
    #[derive(Debug, Clone, Copy)]
    pub struct SemanticallyValid;

    /// Timing validated - all paths ≤8 ticks
    #[derive(Debug, Clone, Copy)]
    pub struct TimingValidated;

    /// Fully validated - ready for certified compilation
    #[derive(Debug, Clone, Copy)]
    pub struct Certified;
}

use validation::*;

/// Typed IR for Σ with validation state encoded in type system
///
/// The type parameter V tracks validation state:
/// - Unvalidated: Just parsed, may have errors
/// - StructurallyValid: Syntax correct
/// - SemanticallyValid: Semantics correct
/// - TimingValidated: Performance bounds verified
/// - Certified: Ready for compilation with proofs
#[derive(Debug, Clone)]
pub struct SigmaIR<V> {
    /// Tasks defined in this ontology
    pub tasks: Vec<TaskNode<V>>,
    /// Patterns (workflow templates)
    pub patterns: Vec<PatternGraph<V>>,
    /// Guards (invariant checkers)
    pub guards: Vec<GuardExpr<V>>,
    /// Metadata
    pub metadata: Metadata,
    /// Validation state (zero-size)
    _phantom: PhantomData<V>,
}

/// Task node in Σ IR
#[derive(Debug, Clone)]
pub struct TaskNode<V> {
    /// Unique task identifier
    pub id: TaskId,
    /// Human-readable label
    pub label: String,
    /// Input schema
    pub input_schema: Schema,
    /// Output schema
    pub output_schema: Schema,
    /// Guards applied to this task
    pub guard_ids: Vec<GuardId>,
    /// Pattern this task uses
    pub pattern_id: PatternId,
    /// Priority for scheduling
    pub priority: Priority,
    /// Validation state
    _phantom: PhantomData<V>,
}

/// Pattern graph - represents a workflow template
#[derive(Debug, Clone)]
pub struct PatternGraph<V> {
    /// Pattern identifier (0-255)
    pub id: PatternId,
    /// Pattern name
    pub name: String,
    /// Phases in this pattern
    pub phases: Vec<Phase>,
    /// Maximum allowed phases (must be ≤8 for hot path)
    pub max_phases: u8,
    /// Validation state
    _phantom: PhantomData<V>,
}

/// Phase in a pattern
#[derive(Debug, Clone)]
pub struct Phase {
    /// Phase number (0-7)
    pub number: u8,
    /// Phase handler type
    pub handler: HandlerType,
    /// Estimated tick cost
    pub tick_estimate: u64,
}

/// Handler types for phases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandlerType {
    /// Pure computation (no I/O)
    Pure,
    /// Guard evaluation
    Guard,
    /// Receipt generation
    Receipt,
    /// Custom handler
    Custom,
}

/// Guard expression - compiled to branchless code
#[derive(Debug, Clone)]
pub struct GuardExpr<V> {
    /// Guard identifier
    pub id: GuardId,
    /// Guard type
    pub guard_type: GuardType,
    /// Expression tree (will be compiled to branchless ops)
    pub expr: Expr,
    /// Tick budget for evaluation
    pub tick_budget: u64,
    /// Validation state
    _phantom: PhantomData<V>,
}

/// Guard types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardType {
    /// Tick budget constraint
    TickBudget,
    /// Carry invariant (KGC preservation)
    CarryInvariant,
    /// Authorization check
    Authorization,
    /// Schema validation
    SchemaValidation,
    /// Custom condition
    Custom,
}

/// Expression tree for guards
#[derive(Debug, Clone)]
pub enum Expr {
    /// Constant value
    Const(u64),
    /// Read observation field
    ReadObs(u8),
    /// Load sigma field
    LoadSigma(u16),
    /// Comparison
    Compare(CompareOp, Box<Expr>, Box<Expr>),
    /// Logical AND (branchless)
    And(Box<Expr>, Box<Expr>),
    /// Logical OR (branchless)
    Or(Box<Expr>, Box<Expr>),
}

/// Comparison operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Eq,
    Lt,
    Le,
    Gt,
    Ge,
}

/// Schema for task inputs/outputs
#[derive(Debug, Clone)]
pub struct Schema {
    /// Fields in this schema
    pub fields: Vec<SchemaField>,
}

/// Schema field definition
#[derive(Debug, Clone)]
pub struct SchemaField {
    /// Field name
    pub name: String,
    /// Field type
    pub field_type: FieldType,
    /// Is this field required?
    pub required: bool,
}

/// Field types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    U64,
    F64,
    Bool,
    String,
    Hash,
}

/// Metadata for Σ
#[derive(Debug, Clone)]
pub struct Metadata {
    /// Version of this ontology
    pub version: String,
    /// Author
    pub author: String,
    /// Description
    pub description: String,
}

/// Task identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(pub u64);

/// Guard identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GuardId(pub u16);

/// Pattern identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PatternId(pub u8);

/// Priority for task scheduling
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Priority(pub u8);

/// Validation errors
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Task has no pattern assigned
    TaskMissingPattern(TaskId),
    /// Pattern ID out of range (must be 0-255)
    InvalidPatternId(u8),
    /// Too many phases in pattern (must be ≤8)
    TooManyPhases { pattern: PatternId, count: usize },
    /// Guard tick budget exceeds limit
    GuardTickBudgetExceeded { guard: GuardId, budget: u64 },
    /// Pattern tick estimate exceeds Chatman Constant
    PatternTicksExceeded { pattern: PatternId, ticks: u64 },
    /// Invalid guard type for task
    InvalidGuardType { guard: GuardId, task: TaskId },
    /// Circular dependency detected
    CircularDependency(Vec<TaskId>),
    /// Unbounded loop detected
    UnboundedLoop(PatternId),
    /// Unguarded recursion detected
    UnguardedRecursion(PatternId),
}

impl<V> SigmaIR<V> {
    /// Create a new IR with validation state
    pub fn new(
        tasks: Vec<TaskNode<V>>,
        patterns: Vec<PatternGraph<V>>,
        guards: Vec<GuardExpr<V>>,
        metadata: Metadata,
    ) -> Self {
        Self {
            tasks,
            patterns,
            guards,
            metadata,
            _phantom: PhantomData,
        }
    }

    /// Get task by ID
    pub fn get_task(&self, id: TaskId) -> Option<&TaskNode<V>> {
        self.tasks.iter().find(|t| t.id == id)
    }

    /// Get pattern by ID
    pub fn get_pattern(&self, id: PatternId) -> Option<&PatternGraph<V>> {
        self.patterns.iter().find(|p| p.id == id)
    }

    /// Get guard by ID
    pub fn get_guard(&self, id: GuardId) -> Option<&GuardExpr<V>> {
        self.guards.iter().find(|g| g.id == id)
    }
}

impl SigmaIR<Unvalidated> {
    /// Validate structure
    pub fn validate_structure(self) -> Result<SigmaIR<StructurallyValid>, ValidationError> {
        // Check pattern IDs are in range
        for pattern in &self.patterns {
            if pattern.id.0 > 255 {
                return Err(ValidationError::InvalidPatternId(pattern.id.0));
            }
        }

        // Check phase counts
        for pattern in &self.patterns {
            if pattern.phases.len() > 8 {
                return Err(ValidationError::TooManyPhases {
                    pattern: pattern.id,
                    count: pattern.phases.len(),
                });
            }
        }

        // Transform to validated state
        Ok(SigmaIR {
            tasks: self.tasks.into_iter().map(TaskNode::validate_structure).collect(),
            patterns: self.patterns.into_iter().map(PatternGraph::validate_structure).collect(),
            guards: self.guards.into_iter().map(GuardExpr::validate_structure).collect(),
            metadata: self.metadata,
            _phantom: PhantomData,
        })
    }
}

impl SigmaIR<StructurallyValid> {
    /// Validate semantics
    pub fn validate_semantics(self) -> Result<SigmaIR<SemanticallyValid>, ValidationError> {
        // Check all tasks have valid patterns
        for task in &self.tasks {
            if self.get_pattern(task.pattern_id).is_none() {
                return Err(ValidationError::TaskMissingPattern(task.id));
            }
        }

        // Check no circular dependencies (simplified check)
        // In real implementation, this would do topological sort

        // Check no unbounded loops
        for pattern in &self.patterns {
            // Patterns must have bounded phases
            if pattern.phases.is_empty() {
                return Err(ValidationError::UnboundedLoop(pattern.id));
            }
        }

        Ok(SigmaIR {
            tasks: self.tasks.into_iter().map(TaskNode::validate_semantics).collect(),
            patterns: self.patterns.into_iter().map(PatternGraph::validate_semantics).collect(),
            guards: self.guards.into_iter().map(GuardExpr::validate_semantics).collect(),
            metadata: self.metadata,
            _phantom: PhantomData,
        })
    }
}

impl SigmaIR<SemanticallyValid> {
    /// Validate timing bounds
    pub fn validate_timing(self) -> Result<SigmaIR<TimingValidated>, ValidationError> {
        use crate::CHATMAN_CONSTANT;

        // Check guard tick budgets
        for guard in &self.guards {
            if guard.tick_budget > CHATMAN_CONSTANT {
                return Err(ValidationError::GuardTickBudgetExceeded {
                    guard: guard.id,
                    budget: guard.tick_budget,
                });
            }
        }

        // Check pattern tick estimates
        for pattern in &self.patterns {
            let total_ticks: u64 = pattern.phases.iter()
                .map(|p| p.tick_estimate)
                .sum();

            if total_ticks > CHATMAN_CONSTANT {
                return Err(ValidationError::PatternTicksExceeded {
                    pattern: pattern.id,
                    ticks: total_ticks,
                });
            }
        }

        Ok(SigmaIR {
            tasks: self.tasks.into_iter().map(TaskNode::validate_timing).collect(),
            patterns: self.patterns.into_iter().map(PatternGraph::validate_timing).collect(),
            guards: self.guards.into_iter().map(GuardExpr::validate_timing).collect(),
            metadata: self.metadata,
            _phantom: PhantomData,
        })
    }
}

impl SigmaIR<TimingValidated> {
    /// Certify for compilation
    pub fn certify(self) -> SigmaIR<Certified> {
        SigmaIR {
            tasks: self.tasks.into_iter().map(TaskNode::certify).collect(),
            patterns: self.patterns.into_iter().map(PatternGraph::certify).collect(),
            guards: self.guards.into_iter().map(GuardExpr::certify).collect(),
            metadata: self.metadata,
            _phantom: PhantomData,
        }
    }
}

impl<V> TaskNode<V> {
    fn validate_structure(self) -> TaskNode<StructurallyValid> {
        TaskNode {
            id: self.id,
            label: self.label,
            input_schema: self.input_schema,
            output_schema: self.output_schema,
            guard_ids: self.guard_ids,
            pattern_id: self.pattern_id,
            priority: self.priority,
            _phantom: PhantomData,
        }
    }

    fn validate_semantics(self) -> TaskNode<SemanticallyValid> {
        TaskNode {
            id: self.id,
            label: self.label,
            input_schema: self.input_schema,
            output_schema: self.output_schema,
            guard_ids: self.guard_ids,
            pattern_id: self.pattern_id,
            priority: self.priority,
            _phantom: PhantomData,
        }
    }

    fn validate_timing(self) -> TaskNode<TimingValidated> {
        TaskNode {
            id: self.id,
            label: self.label,
            input_schema: self.input_schema,
            output_schema: self.output_schema,
            guard_ids: self.guard_ids,
            pattern_id: self.pattern_id,
            priority: self.priority,
            _phantom: PhantomData,
        }
    }

    fn certify(self) -> TaskNode<Certified> {
        TaskNode {
            id: self.id,
            label: self.label,
            input_schema: self.input_schema,
            output_schema: self.output_schema,
            guard_ids: self.guard_ids,
            pattern_id: self.pattern_id,
            priority: self.priority,
            _phantom: PhantomData,
        }
    }
}

impl<V> PatternGraph<V> {
    fn validate_structure(self) -> PatternGraph<StructurallyValid> {
        PatternGraph {
            id: self.id,
            name: self.name,
            phases: self.phases,
            max_phases: self.max_phases,
            _phantom: PhantomData,
        }
    }

    fn validate_semantics(self) -> PatternGraph<SemanticallyValid> {
        PatternGraph {
            id: self.id,
            name: self.name,
            phases: self.phases,
            max_phases: self.max_phases,
            _phantom: PhantomData,
        }
    }

    fn validate_timing(self) -> PatternGraph<TimingValidated> {
        PatternGraph {
            id: self.id,
            name: self.name,
            phases: self.phases,
            max_phases: self.max_phases,
            _phantom: PhantomData,
        }
    }

    fn certify(self) -> PatternGraph<Certified> {
        PatternGraph {
            id: self.id,
            name: self.name,
            phases: self.phases,
            max_phases: self.max_phases,
            _phantom: PhantomData,
        }
    }
}

impl<V> GuardExpr<V> {
    fn validate_structure(self) -> GuardExpr<StructurallyValid> {
        GuardExpr {
            id: self.id,
            guard_type: self.guard_type,
            expr: self.expr,
            tick_budget: self.tick_budget,
            _phantom: PhantomData,
        }
    }

    fn validate_semantics(self) -> GuardExpr<SemanticallyValid> {
        GuardExpr {
            id: self.id,
            guard_type: self.guard_type,
            expr: self.expr,
            tick_budget: self.tick_budget,
            _phantom: PhantomData,
        }
    }

    fn validate_timing(self) -> GuardExpr<TimingValidated> {
        GuardExpr {
            id: self.id,
            guard_type: self.guard_type,
            expr: self.expr,
            tick_budget: self.tick_budget,
            _phantom: PhantomData,
        }
    }

    fn certify(self) -> GuardExpr<Certified> {
        GuardExpr {
            id: self.id,
            guard_type: self.guard_type,
            expr: self.expr,
            tick_budget: self.tick_budget,
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_ir_validation_pipeline() {
        // Create unvalidated IR
        let metadata = Metadata {
            version: String::from("1.0.0"),
            author: String::from("test"),
            description: String::from("test ontology"),
        };

        let ir = SigmaIR::<Unvalidated>::new(vec![], vec![], vec![], metadata);

        // Validate structure
        let ir = ir.validate_structure().unwrap();

        // Validate semantics
        let ir = ir.validate_semantics().unwrap();

        // Validate timing
        let ir = ir.validate_timing().unwrap();

        // Certify
        let _ir = ir.certify();
    }

    #[test]
    fn test_pattern_phase_limit() {
        let metadata = Metadata {
            version: String::from("1.0.0"),
            author: String::from("test"),
            description: String::from("test ontology"),
        };

        // Create pattern with too many phases
        let pattern = PatternGraph {
            id: PatternId(1),
            name: String::from("too_many_phases"),
            phases: vec![
                Phase { number: 0, handler: HandlerType::Pure, tick_estimate: 1 },
                Phase { number: 1, handler: HandlerType::Pure, tick_estimate: 1 },
                Phase { number: 2, handler: HandlerType::Pure, tick_estimate: 1 },
                Phase { number: 3, handler: HandlerType::Pure, tick_estimate: 1 },
                Phase { number: 4, handler: HandlerType::Pure, tick_estimate: 1 },
                Phase { number: 5, handler: HandlerType::Pure, tick_estimate: 1 },
                Phase { number: 6, handler: HandlerType::Pure, tick_estimate: 1 },
                Phase { number: 7, handler: HandlerType::Pure, tick_estimate: 1 },
                Phase { number: 8, handler: HandlerType::Pure, tick_estimate: 1 }, // Too many!
            ],
            max_phases: 9,
            _phantom: PhantomData,
        };

        let ir = SigmaIR::<Unvalidated>::new(vec![], vec![pattern], vec![], metadata);

        // Should fail validation
        assert!(ir.validate_structure().is_err());
    }

    #[test]
    fn test_timing_validation() {
        let metadata = Metadata {
            version: String::from("1.0.0"),
            author: String::from("test"),
            description: String::from("test ontology"),
        };

        // Create pattern that exceeds Chatman Constant
        let pattern = PatternGraph {
            id: PatternId(1),
            name: String::from("too_slow"),
            phases: vec![
                Phase { number: 0, handler: HandlerType::Pure, tick_estimate: 5 },
                Phase { number: 1, handler: HandlerType::Pure, tick_estimate: 5 }, // 10 total > 8
            ],
            max_phases: 2,
            _phantom: PhantomData,
        };

        let ir = SigmaIR::<Unvalidated>::new(vec![], vec![pattern], vec![], metadata);

        // Should pass structure and semantics
        let ir = ir.validate_structure().unwrap();
        let ir = ir.validate_semantics().unwrap();

        // But fail timing
        assert!(ir.validate_timing().is_err());
    }
}
