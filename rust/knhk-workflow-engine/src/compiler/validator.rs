//! Pattern Validator
//!
//! Validates extracted patterns against the pattern permutation matrix.
//! Ensures all patterns are valid according to YAWL semantics.

use crate::compiler::extractor::{ExtractedPattern, PatternType, Guard, GuardType};
use crate::error::{WorkflowError, WorkflowResult};
use oxigraph::{
    io::{RdfFormat, RdfParser},
    model::{NamedNode, Term},
    sparql::QueryResults,
    store::Store,
};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tracing::{debug, info, instrument, warn, error};

/// Pattern validator
pub struct PatternValidator {
    /// Pattern matrix store
    matrix_store: Option<Store>,
    /// Valid pattern combinations
    valid_patterns: HashSet<PatternSignature>,
    /// Validation rules
    rules: ValidationRules,
}

/// Pattern signature for validation
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct PatternSignature {
    pattern_id: u8,
    has_guards: bool,
    has_timeout: bool,
    has_mi: bool,
}

/// Validation rules
#[derive(Debug, Clone)]
struct ValidationRules {
    /// Max guards per pattern
    max_guards: usize,
    /// Max variables per pattern
    max_variables: usize,
    /// Max tick budget
    max_ticks: u8,
    /// Require guards for certain patterns
    guard_requirements: HashMap<u8, bool>,
    /// Pattern compatibility matrix
    compatibility: HashMap<(u8, u8), bool>,
}

impl Default for ValidationRules {
    fn default() -> Self {
        let mut guard_requirements = HashMap::new();
        // Patterns that require guards
        guard_requirements.insert(4, true); // ExclusiveChoice
        guard_requirements.insert(6, true); // MultiChoice
        guard_requirements.insert(10, true); // ArbitraryLoop

        let mut compatibility = HashMap::new();
        // Define which patterns can follow each other
        compatibility.insert((1, 1), true); // Sequence -> Sequence
        compatibility.insert((1, 2), true); // Sequence -> ParallelSplit
        compatibility.insert((2, 3), true); // ParallelSplit -> Synchronization
        compatibility.insert((4, 5), true); // ExclusiveChoice -> SimpleMerge
        compatibility.insert((6, 7), true); // MultiChoice -> SynchronizingMerge
        // Add more as needed

        Self {
            max_guards: 10,
            max_variables: 50,
            max_ticks: 8, // Chatman constant
            guard_requirements,
            compatibility,
        }
    }
}

impl PatternValidator {
    /// Create new validator
    pub fn new(pattern_matrix_path: &str) -> Self {
        Self {
            matrix_store: None,
            valid_patterns: HashSet::new(),
            rules: ValidationRules::default(),
        }
    }

    /// Load pattern matrix
    pub async fn load_pattern_matrix(&mut self, path: &Path) -> WorkflowResult<()> {
        info!("Loading pattern matrix from: {:?}", path);

        let content = std::fs::read_to_string(path)
            .map_err(|e| WorkflowError::Io(format!("Cannot read pattern matrix: {}", e)))?;

        let store = Store::new()
            .map_err(|e| WorkflowError::Internal(format!("Cannot create store: {}", e)))?;

        let parser = RdfParser::from_format(RdfFormat::Turtle)
            .with_base_iri("http://bitflow.ai/ontology/yawl/patterns#")
            .map_err(|e| WorkflowError::Parse(format!("Invalid base IRI: {}", e)))?;

        let quads = parser
            .parse_from_read(content.as_bytes())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| WorkflowError::Parse(format!("Failed to parse pattern matrix: {}", e)))?;

        for quad in quads {
            store.insert(&quad)
                .map_err(|e| WorkflowError::Internal(format!("Failed to insert quad: {}", e)))?;
        }

        self.matrix_store = Some(store.clone());

        // Extract valid patterns from matrix
        self.extract_valid_patterns(&store)?;

        info!("Loaded {} valid pattern signatures", self.valid_patterns.len());
        Ok(())
    }

    /// Extract valid patterns from matrix
    fn extract_valid_patterns(&mut self, store: &Store) -> WorkflowResult<()> {
        let query = r#"
            PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

            SELECT ?pattern ?id ?guards ?timeout ?mi WHERE {
                ?pattern rdf:type yawl:ValidPattern .
                ?pattern yawl:patternId ?id .
                OPTIONAL { ?pattern yawl:requiresGuards ?guards }
                OPTIONAL { ?pattern yawl:allowsTimeout ?timeout }
                OPTIONAL { ?pattern yawl:allowsMultipleInstance ?mi }
            }
        "#;

        let results = store.query(query)
            .map_err(|e| WorkflowError::Parse(format!("Pattern extraction failed: {}", e)))?;

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution
                    .map_err(|e| WorkflowError::Parse(format!("Solution error: {}", e)))?;

                if let Some(Term::Literal(id_lit)) = solution.get("id") {
                    if let Ok(pattern_id) = id_lit.value().parse::<u8>() {
                        let has_guards = solution.get("guards")
                            .and_then(|t| if let Term::Literal(lit) = t {
                                lit.value().parse::<bool>().ok()
                            } else { None })
                            .unwrap_or(false);

                        let has_timeout = solution.get("timeout")
                            .and_then(|t| if let Term::Literal(lit) = t {
                                lit.value().parse::<bool>().ok()
                            } else { None })
                            .unwrap_or(false);

                        let has_mi = solution.get("mi")
                            .and_then(|t| if let Term::Literal(lit) = t {
                                lit.value().parse::<bool>().ok()
                            } else { None })
                            .unwrap_or(false);

                        self.valid_patterns.insert(PatternSignature {
                            pattern_id,
                            has_guards,
                            has_timeout,
                            has_mi,
                        });
                    }
                }
            }
        }

        // If no patterns found in matrix, use default set
        if self.valid_patterns.is_empty() {
            self.populate_default_patterns();
        }

        Ok(())
    }

    /// Populate default valid patterns
    fn populate_default_patterns(&mut self) {
        // All 43 workflow patterns
        for id in 1..=43 {
            // Basic patterns without guards
            self.valid_patterns.insert(PatternSignature {
                pattern_id: id,
                has_guards: false,
                has_timeout: false,
                has_mi: false,
            });

            // Patterns with guards
            if id == 4 || id == 6 || id == 10 {
                self.valid_patterns.insert(PatternSignature {
                    pattern_id: id,
                    has_guards: true,
                    has_timeout: false,
                    has_mi: false,
                });
            }

            // Multiple instance pattern
            if id == 13 {
                self.valid_patterns.insert(PatternSignature {
                    pattern_id: id,
                    has_guards: false,
                    has_timeout: false,
                    has_mi: true,
                });
            }
        }
    }

    /// Validate all patterns
    #[instrument(skip(self, patterns))]
    pub async fn validate_patterns(&self, patterns: &[ExtractedPattern]) -> WorkflowResult<()> {
        info!("Validating {} patterns", patterns.len());

        for pattern in patterns {
            self.validate_single_pattern(pattern)?;
        }

        // Validate pattern relationships
        self.validate_pattern_relationships(patterns)?;

        info!("All patterns validated successfully");
        Ok(())
    }

    /// Validate single pattern
    fn validate_single_pattern(&self, pattern: &ExtractedPattern) -> WorkflowResult<()> {
        debug!("Validating pattern: {}", pattern.iri);

        // Check pattern ID range
        if pattern.pattern_id < 1 || pattern.pattern_id > 43 {
            return Err(WorkflowError::Validation(
                format!("Invalid pattern ID: {} (must be 1-43)", pattern.pattern_id)
            ));
        }

        // Check tick budget
        if pattern.tick_budget > self.rules.max_ticks {
            return Err(WorkflowError::Validation(
                format!("Pattern {} exceeds tick budget: {} > {}",
                    pattern.iri, pattern.tick_budget, self.rules.max_ticks)
            ));
        }

        // Check guard requirements
        if let Some(&requires_guards) = self.rules.guard_requirements.get(&pattern.pattern_id) {
            if requires_guards && pattern.guards.is_empty() {
                return Err(WorkflowError::Validation(
                    format!("Pattern {} (type {}) requires guards",
                        pattern.iri, pattern.pattern_id)
                ));
            }
        }

        // Check guard count
        if pattern.guards.len() > self.rules.max_guards {
            return Err(WorkflowError::Validation(
                format!("Pattern {} has too many guards: {} > {}",
                    pattern.iri, pattern.guards.len(), self.rules.max_guards)
            ));
        }

        // Check variable count
        if pattern.variables.len() > self.rules.max_variables {
            return Err(WorkflowError::Validation(
                format!("Pattern {} has too many variables: {} > {}",
                    pattern.iri, pattern.variables.len(), self.rules.max_variables)
            ));
        }

        // Validate against pattern matrix
        self.validate_against_matrix(pattern)?;

        // Validate guards
        for guard in &pattern.guards {
            self.validate_guard(guard)?;
        }

        // Validate constraints
        self.validate_constraints(pattern)?;

        Ok(())
    }

    /// Validate pattern against matrix
    fn validate_against_matrix(&self, pattern: &ExtractedPattern) -> WorkflowResult<()> {
        if self.valid_patterns.is_empty() {
            // If no matrix loaded, allow all patterns
            warn!("No pattern matrix loaded, skipping matrix validation");
            return Ok(());
        }

        let signature = PatternSignature {
            pattern_id: pattern.pattern_id,
            has_guards: !pattern.guards.is_empty(),
            has_timeout: pattern.timeout_ms.is_some(),
            has_mi: pattern.pattern_type == PatternType::MultipleInstance,
        };

        if !self.valid_patterns.contains(&signature) {
            // Try without optional features
            let basic_signature = PatternSignature {
                pattern_id: pattern.pattern_id,
                has_guards: false,
                has_timeout: false,
                has_mi: false,
            };

            if !self.valid_patterns.contains(&basic_signature) {
                return Err(WorkflowError::Validation(
                    format!("Pattern {} with signature {:?} is not in valid pattern matrix",
                        pattern.iri, signature)
                ));
            }
        }

        Ok(())
    }

    /// Validate guard
    fn validate_guard(&self, guard: &Guard) -> WorkflowResult<()> {
        // Check expression is not empty
        if guard.expression.trim().is_empty() {
            return Err(WorkflowError::Validation(
                format!("Guard {} has empty expression", guard.id)
            ));
        }

        // Check variables referenced exist
        if guard.variables.is_empty() && guard.expression != "true" && guard.expression != "false" {
            warn!("Guard {} references no variables: {}", guard.id, guard.expression);
        }

        // Validate guard type specific rules
        match guard.guard_type {
            GuardType::PreCondition => {
                // Pre-conditions should not have side effects
                if guard.expression.contains("=") && !guard.expression.contains("==") {
                    warn!("Pre-condition guard {} may have side effects", guard.id);
                }
            }
            GuardType::PostCondition => {
                // Post-conditions can have assignments
            }
            GuardType::Invariant => {
                // Invariants must be pure boolean expressions
                if guard.expression.contains(":=") || guard.expression.contains("++") {
                    return Err(WorkflowError::Validation(
                        format!("Invariant guard {} must not have side effects", guard.id)
                    ));
                }
            }
            GuardType::ExceptionHandler => {
                // Exception handlers have special semantics
            }
        }

        Ok(())
    }

    /// Validate constraints
    fn validate_constraints(&self, pattern: &ExtractedPattern) -> WorkflowResult<()> {
        for constraint in &pattern.constraints {
            // Check expression is valid
            if constraint.expression.trim().is_empty() {
                return Err(WorkflowError::Validation(
                    format!("Pattern {} has constraint with empty expression", pattern.iri)
                ));
            }

            // Type-specific validation
            match constraint.constraint_type {
                crate::compiler::extractor::ConstraintType::Temporal => {
                    // Temporal constraints should reference time
                    if !constraint.expression.contains("time") &&
                       !constraint.expression.contains("duration") &&
                       !constraint.expression.contains("timeout") {
                        warn!("Temporal constraint may not reference time: {}",
                            constraint.expression);
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Validate pattern relationships
    fn validate_pattern_relationships(&self, patterns: &[ExtractedPattern]) -> WorkflowResult<()> {
        // Build pattern graph
        let mut pattern_map: HashMap<String, &ExtractedPattern> = HashMap::new();
        for pattern in patterns {
            pattern_map.insert(pattern.iri.clone(), pattern);
        }

        // Check for cycles
        self.check_for_cycles(&pattern_map)?;

        // Validate pattern compatibility
        self.validate_compatibility(&pattern_map)?;

        Ok(())
    }

    /// Check for invalid cycles
    fn check_for_cycles(&self, patterns: &HashMap<String, &ExtractedPattern>) -> WorkflowResult<()> {
        // Simple cycle detection
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for (iri, _) in patterns.iter() {
            if !visited.contains(iri) {
                if self.has_cycle_util(iri, patterns, &mut visited, &mut rec_stack)? {
                    return Err(WorkflowError::Validation(
                        format!("Cycle detected in workflow patterns starting at {}", iri)
                    ));
                }
            }
        }

        Ok(())
    }

    /// Utility for cycle detection
    fn has_cycle_util(
        &self,
        iri: &str,
        patterns: &HashMap<String, &ExtractedPattern>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> WorkflowResult<bool> {
        visited.insert(iri.to_string());
        rec_stack.insert(iri.to_string());

        // Check data flows for cycles
        if let Some(pattern) = patterns.get(iri) {
            for flow in &pattern.data_flows {
                // If target references another pattern
                if let Some(target_pattern) = patterns.get(&flow.target) {
                    if !visited.contains(&target_pattern.iri) {
                        if self.has_cycle_util(&target_pattern.iri, patterns, visited, rec_stack)? {
                            return Ok(true);
                        }
                    } else if rec_stack.contains(&target_pattern.iri) {
                        return Ok(true);
                    }
                }
            }
        }

        rec_stack.remove(iri);
        Ok(false)
    }

    /// Validate pattern compatibility
    fn validate_compatibility(&self, patterns: &HashMap<String, &ExtractedPattern>) -> WorkflowResult<()> {
        // Check if patterns that flow into each other are compatible
        for (_, pattern) in patterns.iter() {
            for flow in &pattern.data_flows {
                if let Some(target_pattern) = patterns.get(&flow.target) {
                    let key = (pattern.pattern_id, target_pattern.pattern_id);
                    if let Some(&compatible) = self.rules.compatibility.get(&key) {
                        if !compatible {
                            return Err(WorkflowError::Validation(
                                format!("Incompatible pattern flow: {} ({}) -> {} ({})",
                                    pattern.iri, pattern.pattern_id,
                                    target_pattern.iri, target_pattern.pattern_id)
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate pattern completeness
    pub fn validate_completeness(&self, patterns: &[ExtractedPattern]) -> WorkflowResult<()> {
        // Check for start patterns
        let has_start = patterns.iter().any(|p|
            p.guards.iter().any(|g| g.guard_type == GuardType::PreCondition &&
                                    g.expression.contains("start")));

        if !has_start {
            warn!("No explicit start condition found in patterns");
        }

        // Check for end patterns
        let has_end = patterns.iter().any(|p|
            p.guards.iter().any(|g| g.guard_type == GuardType::PostCondition &&
                                    g.expression.contains("complete")));

        if !has_end {
            warn!("No explicit completion condition found in patterns");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::extractor::{Variable, DataType, Constraint, DataFlow, EventHandler};

    fn create_test_pattern(id: u8) -> ExtractedPattern {
        ExtractedPattern {
            pattern_id: id,
            iri: format!("http://example.org/pattern/{}", id),
            pattern_type: PatternType::Sequence,
            guards: Vec::new(),
            variables: Vec::new(),
            timeout_ms: None,
            tick_budget: 3,
            constraints: Vec::new(),
            data_flows: Vec::new(),
            event_handlers: Vec::new(),
        }
    }

    #[tokio::test]
    async fn test_validator_creation() {
        let validator = PatternValidator::new("test.ttl");
        assert!(validator.valid_patterns.is_empty());
    }

    #[tokio::test]
    async fn test_pattern_validation() {
        let validator = PatternValidator::new("test.ttl");

        let pattern = create_test_pattern(1);
        validator.validate_single_pattern(&pattern).unwrap();
    }

    #[tokio::test]
    async fn test_invalid_pattern_id() {
        let validator = PatternValidator::new("test.ttl");

        let mut pattern = create_test_pattern(44); // Invalid ID
        let result = validator.validate_single_pattern(&pattern);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tick_budget_validation() {
        let validator = PatternValidator::new("test.ttl");

        let mut pattern = create_test_pattern(1);
        pattern.tick_budget = 9; // Exceeds Chatman constant

        let result = validator.validate_single_pattern(&pattern);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_guard_validation() {
        let validator = PatternValidator::new("test.ttl");

        let guard = Guard {
            id: "guard1".to_string(),
            expression: "x > 0".to_string(),
            variables: vec!["x".to_string()],
            guard_type: GuardType::PreCondition,
        };

        validator.validate_guard(&guard).unwrap();
    }
}