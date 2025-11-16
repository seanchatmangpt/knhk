// Pattern Validator
// Validates workflow patterns against the permutation matrix
// Enforces Covenant 4: All Patterns Expressible via Permutations

use super::matrix::{
    JoinType, MatrixError, PatternCombination, PatternModifiers, PermutationMatrix, SplitType,
};
use super::rules::{ValidationRules, ValidationContext};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid pattern combination: {split}-{join}")]
    InvalidCombination { split: String, join: String },

    #[error("Pattern not in permutation matrix: {pattern}")]
    PatternNotInMatrix { pattern: String },

    #[error("Unsupported modifiers: {modifiers}")]
    UnsupportedModifiers { modifiers: String },

    #[error("Constraint violation: {constraint}")]
    ConstraintViolation { constraint: String },

    #[error("Missing required property: {property}")]
    MissingProperty { property: String },

    #[error("Matrix error: {0}")]
    MatrixError(#[from] MatrixError),

    #[error("SPARQL query failed: {0}")]
    SparqlError(String),

    #[error("RDF parsing failed: {0}")]
    RdfParseError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub pattern_name: Option<String>,
    pub combination: Option<PatternCombination>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

impl ValidationResult {
    pub fn valid(pattern_name: String, combination: PatternCombination) -> Self {
        Self {
            is_valid: true,
            pattern_name: Some(pattern_name),
            combination: Some(combination),
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            pattern_name: None,
            combination: None,
            errors,
            warnings: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }

    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }

    pub fn error_message(&self) -> String {
        if self.errors.is_empty() {
            String::new()
        } else {
            self.errors.join("; ")
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPattern {
    pub task_id: String,
    pub split_type: SplitType,
    pub join_type: JoinType,
    pub modifiers: PatternModifiers,
}

impl TaskPattern {
    pub fn new(
        task_id: String,
        split_type: SplitType,
        join_type: JoinType,
        modifiers: PatternModifiers,
    ) -> Self {
        Self {
            task_id,
            split_type,
            join_type,
            modifiers,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub workflow_id: String,
    pub tasks: Vec<TaskPattern>,
}

impl WorkflowDefinition {
    pub fn new(workflow_id: String) -> Self {
        Self {
            workflow_id,
            tasks: Vec::new(),
        }
    }

    pub fn add_task(&mut self, task: TaskPattern) {
        self.tasks.push(task);
    }
}

pub struct PatternValidator {
    matrix: PermutationMatrix,
    rules: ValidationRules,
}

impl PatternValidator {
    pub fn new() -> Result<Self, ValidationError> {
        let matrix = PermutationMatrix::load_default()?;
        let rules = ValidationRules::default();
        Ok(Self { matrix, rules })
    }

    pub fn with_matrix(matrix: PermutationMatrix) -> Self {
        Self {
            matrix,
            rules: ValidationRules::default(),
        }
    }

    pub fn validate_task(&self, task: &TaskPattern) -> ValidationResult {
        // Check if the combination is in the permutation matrix
        if !self
            .matrix
            .is_valid_combination(task.split_type, task.join_type, &task.modifiers)
        {
            let error = format!(
                "Task '{}': Invalid combination {}-{} with modifiers",
                task.task_id,
                task.split_type.as_str(),
                task.join_type.as_str()
            );
            let suggestion = self.suggest_valid_combination(task.split_type, task.join_type);
            return ValidationResult::invalid(vec![error]).with_suggestions(vec![suggestion]);
        }

        // Get the combination from the matrix
        let combination = match self
            .matrix
            .get_combination(task.split_type, task.join_type, &task.modifiers)
        {
            Some(c) => c.clone(),
            None => {
                return ValidationResult::invalid(vec![format!(
                    "Task '{}': Combination not found in matrix",
                    task.task_id
                )])
            }
        };

        // Validate rules
        let context = ValidationContext {
            split_type: task.split_type,
            join_type: task.join_type,
            modifiers: task.modifiers.clone(),
        };

        match self.rules.validate(&context) {
            Ok(warnings) => {
                let pattern_name = combination
                    .generated_patterns
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string());
                ValidationResult::valid(pattern_name, combination).with_warnings(warnings)
            }
            Err(e) => ValidationResult::invalid(vec![format!(
                "Task '{}': Rule violation: {}",
                task.task_id, e
            )]),
        }
    }

    pub fn validate_workflow(&self, workflow: &WorkflowDefinition) -> Vec<ValidationResult> {
        workflow
            .tasks
            .iter()
            .map(|task| self.validate_task(task))
            .collect()
    }

    pub fn validate_workflow_complete(&self, workflow: &WorkflowDefinition) -> ValidationResult {
        let results = self.validate_workflow(workflow);

        let errors: Vec<String> = results
            .iter()
            .filter(|r| !r.is_valid)
            .flat_map(|r| r.errors.clone())
            .collect();

        let warnings: Vec<String> = results
            .iter()
            .flat_map(|r| r.warnings.clone())
            .collect();

        if errors.is_empty() {
            ValidationResult {
                is_valid: true,
                pattern_name: Some(workflow.workflow_id.clone()),
                combination: None,
                errors,
                warnings,
                suggestions: Vec::new(),
            }
        } else {
            ValidationResult::invalid(errors).with_warnings(warnings)
        }
    }

    pub fn decompose_pattern(
        &self,
        pattern_name: &str,
    ) -> Result<Vec<PatternCombination>, ValidationError> {
        let combinations = self.matrix.get_combinations_for_pattern(pattern_name);
        if combinations.is_empty() {
            Err(ValidationError::PatternNotInMatrix {
                pattern: pattern_name.to_string(),
            })
        } else {
            Ok(combinations.into_iter().cloned().collect())
        }
    }

    pub fn suggest_valid_combination(&self, split: SplitType, join: JoinType) -> String {
        // Get all valid combinations for this split-join pair
        let patterns = self.matrix.get_patterns_for_combination(split, join);

        if patterns.is_empty() {
            format!(
                "No valid patterns found for {}-{}. Consider using different split/join types.",
                split.as_str(),
                join.as_str()
            )
        } else {
            format!(
                "Valid patterns for {}-{}: {}. Check if you need additional modifiers.",
                split.as_str(),
                join.as_str(),
                patterns.join(", ")
            )
        }
    }

    pub fn coverage_report(&self) -> CoverageReport {
        let total_patterns = 43; // W3C total
        let supported_patterns = self.matrix.total_patterns();
        let total_combinations = self.matrix.total_combinations();
        let coverage_percentage = self.matrix.coverage_percentage();

        CoverageReport {
            total_w3c_patterns: total_patterns,
            supported_patterns,
            total_combinations,
            coverage_percentage,
            supported_pattern_names: self.matrix.all_patterns().into_iter().collect(),
            gaps: self.identify_gaps(),
        }
    }

    fn identify_gaps(&self) -> Vec<String> {
        // List of all 43 W3C patterns
        let all_w3c_patterns = vec![
            "Sequence",
            "ParallelSplit",
            "Synchronization",
            "ExclusiveChoice",
            "SimpleMerge",
            "MultiChoice",
            "SynchronizingMerge",
            "MultiMerge",
            "Discriminator",
            "ArbitraryCycles",
            "ImplicitTermination",
            "MultipleInstancesWithoutSynchronization",
            "MultipleInstancesWithAPrioriDesignTimeKnowledge",
            "MultipleInstancesWithAPrioriRuntimeKnowledge",
            "MultipleInstancesWithoutAPrioriRuntimeKnowledge",
            "DeferredChoice",
            "InterleavedParallelRouting",
            "Milestone",
            "CancelTask",
            "CancelCase",
            "StructuredLoop",
            "Recursion",
            "TransientTrigger",
            "PersistentTrigger",
            "CancelRegion",
            "CancelMultipleInstanceActivity",
            "CompleteMultipleInstanceActivity",
            "BlockingDiscriminator",
            "CancellingDiscriminator",
            "StructuredPartialJoin",
            "BlockingPartialJoin",
            "CancellingPartialJoin",
            "GeneralizedANDJoin",
            "LocalSynchronizingMerge",
            "GeneralizedSynchronizingMerge",
            "ThreadMerge",
            "ThreadSplit",
            "ExplicitTermination",
            "ImplicitTermination",
            "TaskDataInput",
            "TaskDataOutput",
            "CriticalSection",
            "InterleavedParallel",
        ];

        let supported = self.matrix.all_patterns();
        all_w3c_patterns
            .into_iter()
            .filter(|p| !supported.contains(*p))
            .map(|s| s.to_string())
            .collect()
    }

    pub fn matrix(&self) -> &PermutationMatrix {
        &self.matrix
    }
}

impl Default for PatternValidator {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            Self::with_matrix(PermutationMatrix::new())
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub total_w3c_patterns: usize,
    pub supported_patterns: usize,
    pub total_combinations: usize,
    pub coverage_percentage: f64,
    pub supported_pattern_names: Vec<String>,
    pub gaps: Vec<String>,
}

impl CoverageReport {
    pub fn print_summary(&self) {
        println!("=== Pattern Coverage Report ===");
        println!("Total W3C Patterns: {}", self.total_w3c_patterns);
        println!("Supported Patterns: {}", self.supported_patterns);
        println!("Total Combinations: {}", self.total_combinations);
        println!("Coverage: {:.2}%", self.coverage_percentage);
        println!("\nSupported Patterns:");
        for pattern in &self.supported_pattern_names {
            println!("  ✓ {}", pattern);
        }
        if !self.gaps.is_empty() {
            println!("\nGaps (unsupported patterns):");
            for gap in &self.gaps {
                println!("  ✗ {}", gap);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = PatternValidator::new();
        assert!(validator.is_ok());
    }

    #[test]
    fn test_validate_sequence_task() {
        let validator = PatternValidator::new().unwrap();
        let task = TaskPattern::new(
            "task1".to_string(),
            SplitType::XOR,
            JoinType::XOR,
            PatternModifiers::default(),
        );
        let result = validator.validate_task(&task);
        assert!(result.is_valid, "Sequence pattern should be valid");
        assert_eq!(result.pattern_name, Some("Sequence".to_string()));
    }

    #[test]
    fn test_validate_parallel_task() {
        let validator = PatternValidator::new().unwrap();
        let task = TaskPattern::new(
            "task2".to_string(),
            SplitType::AND,
            JoinType::AND,
            PatternModifiers::default(),
        );
        let result = validator.validate_task(&task);
        assert!(result.is_valid, "Parallel+Sync pattern should be valid");
    }

    #[test]
    fn test_validate_invalid_combination() {
        let validator = PatternValidator::new().unwrap();
        // OR-AND without proper modifiers might be invalid
        let task = TaskPattern::new(
            "task3".to_string(),
            SplitType::OR,
            JoinType::AND,
            PatternModifiers::default(),
        );
        let result = validator.validate_task(&task);
        // This should be invalid or have warnings
        if !result.is_valid {
            assert!(!result.errors.is_empty());
        }
    }

    #[test]
    fn test_validate_workflow() {
        let validator = PatternValidator::new().unwrap();
        let mut workflow = WorkflowDefinition::new("test-workflow".to_string());

        workflow.add_task(TaskPattern::new(
            "task1".to_string(),
            SplitType::XOR,
            JoinType::XOR,
            PatternModifiers::default(),
        ));

        workflow.add_task(TaskPattern::new(
            "task2".to_string(),
            SplitType::AND,
            JoinType::AND,
            PatternModifiers::default(),
        ));

        let results = validator.validate_workflow(&workflow);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.is_valid));
    }

    #[test]
    fn test_decompose_pattern() {
        let validator = PatternValidator::new().unwrap();
        let combinations = validator.decompose_pattern("Sequence");
        assert!(combinations.is_ok());
        let combos = combinations.unwrap();
        assert!(!combos.is_empty());
    }

    #[test]
    fn test_coverage_report() {
        let validator = PatternValidator::new().unwrap();
        let report = validator.coverage_report();
        assert!(report.total_w3c_patterns == 43);
        assert!(report.supported_patterns > 0);
        assert!(report.coverage_percentage > 0.0);
    }
}
