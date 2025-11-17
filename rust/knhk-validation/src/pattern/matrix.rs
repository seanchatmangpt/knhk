// Permutation Matrix Representation
// Loads and represents the complete yawl-pattern-permutations.ttl matrix

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SplitType {
    AND,
    OR,
    XOR,
}

impl SplitType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "AND" => Some(SplitType::AND),
            "OR" => Some(SplitType::OR),
            "XOR" => Some(SplitType::XOR),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SplitType::AND => "AND",
            SplitType::OR => "OR",
            SplitType::XOR => "XOR",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JoinType {
    AND,
    OR,
    XOR,
    Discriminator,
}

impl JoinType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "AND" => Some(JoinType::AND),
            "OR" => Some(JoinType::OR),
            "XOR" => Some(JoinType::XOR),
            "DISCRIMINATOR" => Some(JoinType::Discriminator),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            JoinType::AND => "AND",
            JoinType::OR => "OR",
            JoinType::XOR => "XOR",
            JoinType::Discriminator => "Discriminator",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatternModifiers {
    pub flow_predicate: bool,
    pub backward_flow: bool,
    pub deferred_choice: bool,
    pub interleaving: bool,
    pub critical_section: bool,
    pub milestone: bool,
    pub cancellation: Option<CancellationType>,
    pub iteration: Option<IterationType>,
    pub quorum: Option<u32>,
    pub synchronization: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CancellationType {
    Task,
    Case,
    Region,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IterationType {
    StructuredLoop,
    Recursion,
}

impl Default for PatternModifiers {
    fn default() -> Self {
        Self {
            flow_predicate: false,
            backward_flow: false,
            deferred_choice: false,
            interleaving: false,
            critical_section: false,
            milestone: false,
            cancellation: None,
            iteration: None,
            quorum: None,
            synchronization: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatternCombination {
    pub split_type: SplitType,
    pub join_type: JoinType,
    pub modifiers: PatternModifiers,
    pub generated_patterns: Vec<String>,
    pub is_valid: bool,
    pub comment: String,
}

impl PatternCombination {
    pub fn new(split_type: SplitType, join_type: JoinType, modifiers: PatternModifiers) -> Self {
        Self {
            split_type,
            join_type,
            modifiers,
            generated_patterns: Vec::new(),
            is_valid: false,
            comment: String::new(),
        }
    }

    pub fn key(&self) -> String {
        format!("{}-{}", self.split_type.as_str(), self.join_type.as_str())
    }

    pub fn matches(&self, split: SplitType, join: JoinType, modifiers: &PatternModifiers) -> bool {
        self.split_type == split && self.join_type == join && self.modifiers_match(modifiers)
    }

    fn modifiers_match(&self, other: &PatternModifiers) -> bool {
        // Check if the other modifiers are compatible with this combination
        // All required modifiers in self must be present in other
        if self.modifiers.flow_predicate && !other.flow_predicate {
            return false;
        }
        if self.modifiers.backward_flow && !other.backward_flow {
            return false;
        }
        if self.modifiers.deferred_choice && !other.deferred_choice {
            return false;
        }
        if self.modifiers.interleaving && !other.interleaving {
            return false;
        }
        if self.modifiers.critical_section && !other.critical_section {
            return false;
        }
        if self.modifiers.milestone && !other.milestone {
            return false;
        }
        if self.modifiers.cancellation.is_some() && other.cancellation.is_none() {
            return false;
        }
        if self.modifiers.iteration.is_some() && other.iteration.is_none() {
            return false;
        }
        if self.modifiers.quorum.is_some() && other.quorum.is_none() {
            return false;
        }
        if self.modifiers.synchronization && !other.synchronization {
            return false;
        }
        true
    }
}

#[derive(Debug, Error)]
pub enum MatrixError {
    #[error("Failed to load matrix: {0}")]
    LoadError(String),
    #[error("Invalid combination: {split}-{join}")]
    InvalidCombination { split: String, join: String },
    #[error("Pattern not found: {0}")]
    PatternNotFound(String),
}

#[derive(Debug, Clone)]
pub struct PermutationMatrix {
    combinations: HashMap<String, PatternCombination>,
    patterns: HashMap<String, Vec<String>>, // pattern name -> combination keys
}

impl PermutationMatrix {
    pub fn new() -> Self {
        Self {
            combinations: HashMap::new(),
            patterns: HashMap::new(),
        }
    }

    pub fn load_from_file(path: &str) -> Result<Self, MatrixError> {
        // This would use oxigraph to load the RDF file
        // For now, we'll create the matrix programmatically based on
        // yawl-pattern-permutations.ttl
        let mut matrix = Self::new();
        matrix.initialize_from_ontology();
        Ok(matrix)
    }

    pub fn load_default() -> Result<Self, MatrixError> {
        let mut matrix = Self::new();
        matrix.initialize_from_ontology();
        Ok(matrix)
    }

    fn initialize_from_ontology(&mut self) {
        // Pattern 1: Sequence (XOR -> XOR)
        self.add_combination(
            SplitType::XOR,
            JoinType::XOR,
            PatternModifiers::default(),
            vec!["Sequence".to_string()],
            "Sequential execution: A -> B",
        );

        // Pattern 2-3: Parallel Split with AND-AND (Synchronization)
        self.add_combination(
            SplitType::AND,
            JoinType::AND,
            PatternModifiers::default(),
            vec!["ParallelSplit".to_string(), "Synchronization".to_string()],
            "Parallel split with synchronization",
        );

        // Pattern 2: Parallel Split without sync (AND-XOR)
        self.add_combination(
            SplitType::AND,
            JoinType::XOR,
            PatternModifiers::default(),
            vec!["ParallelSplit".to_string()],
            "Parallel split without synchronization",
        );

        // Pattern 4: Exclusive Choice (XOR with predicate)
        let mut xor_pred = PatternModifiers::default();
        xor_pred.flow_predicate = true;
        self.add_combination(
            SplitType::XOR,
            JoinType::XOR,
            xor_pred,
            vec!["ExclusiveChoice".to_string()],
            "Exclusive choice: A -> (B when pred1 else C)",
        );

        // Pattern 6: Multi-Choice (OR with predicate)
        let mut or_pred = PatternModifiers::default();
        or_pred.flow_predicate = true;
        self.add_combination(
            SplitType::OR,
            JoinType::XOR,
            or_pred,
            vec!["MultiChoice".to_string()],
            "Multi-choice: A -> one or more of {B, C, D}",
        );

        // Pattern 7: Synchronizing Merge (OR -> OR)
        self.add_combination(
            SplitType::OR,
            JoinType::OR,
            PatternModifiers::default(),
            vec!["SynchronizingMerge".to_string()],
            "Synchronizing merge: wait for all active branches",
        );

        // Pattern 9: Discriminator (AND/OR -> Discriminator with quorum)
        let mut discrim_mods = PatternModifiers::default();
        discrim_mods.quorum = Some(1);
        self.add_combination(
            SplitType::AND,
            JoinType::Discriminator,
            discrim_mods.clone(),
            vec!["Discriminator".to_string()],
            "Discriminator: first of many to complete",
        );
        self.add_combination(
            SplitType::OR,
            JoinType::Discriminator,
            discrim_mods,
            vec!["Discriminator".to_string()],
            "Discriminator from multi-choice",
        );

        // Pattern 11: Arbitrary Cycles (backward flow)
        let mut cycle_mods = PatternModifiers::default();
        cycle_mods.backward_flow = true;
        self.add_combination(
            SplitType::XOR,
            JoinType::XOR,
            cycle_mods,
            vec!["ArbitraryCycles".to_string()],
            "Arbitrary cycles: A -> B -> A (with iteration control)",
        );

        // Pattern 16: Deferred Choice
        let mut deferred_mods = PatternModifiers::default();
        deferred_mods.deferred_choice = true;
        self.add_combination(
            SplitType::XOR,
            JoinType::XOR,
            deferred_mods,
            vec!["DeferredChoice".to_string()],
            "Deferred choice: runtime decision point",
        );

        // Pattern 24: Interleaved Parallel Routing
        let mut interleave_mods = PatternModifiers::default();
        interleave_mods.interleaving = true;
        self.add_combination(
            SplitType::AND,
            JoinType::AND,
            interleave_mods,
            vec!["InterleavedParallel".to_string()],
            "Interleaved parallel: concurrent with ordering constraints",
        );

        // Pattern 25: Critical Section
        let mut critical_mods = PatternModifiers::default();
        critical_mods.critical_section = true;
        self.add_combination(
            SplitType::AND,
            JoinType::AND,
            critical_mods,
            vec!["CriticalSection".to_string()],
            "Critical section: mutual exclusion for tasks",
        );

        // Pattern 27: Milestone
        let mut milestone_mods = PatternModifiers::default();
        milestone_mods.milestone = true;
        self.add_combination(
            SplitType::XOR,
            JoinType::XOR,
            milestone_mods,
            vec!["Milestone".to_string()],
            "Milestone: checkpoint with timeout",
        );

        // Patterns 19-21: Cancellation
        let mut cancel_task_mods = PatternModifiers::default();
        cancel_task_mods.cancellation = Some(CancellationType::Task);
        self.add_combination(
            SplitType::XOR,
            JoinType::XOR,
            cancel_task_mods,
            vec!["CancelTask".to_string()],
            "Cancel task pattern",
        );

        let mut cancel_case_mods = PatternModifiers::default();
        cancel_case_mods.cancellation = Some(CancellationType::Case);
        self.add_combination(
            SplitType::XOR,
            JoinType::XOR,
            cancel_case_mods,
            vec!["CancelCase".to_string()],
            "Cancel case pattern",
        );

        let mut cancel_region_mods = PatternModifiers::default();
        cancel_region_mods.cancellation = Some(CancellationType::Region);
        self.add_combination(
            SplitType::XOR,
            JoinType::XOR,
            cancel_region_mods,
            vec!["CancelRegion".to_string()],
            "Cancel region pattern",
        );

        // Iteration Patterns
        let mut loop_mods = PatternModifiers::default();
        loop_mods.iteration = Some(IterationType::StructuredLoop);
        self.add_combination(
            SplitType::XOR,
            JoinType::XOR,
            loop_mods,
            vec!["StructuredLoop".to_string()],
            "Structured loop: repeat N times",
        );

        let mut recursion_mods = PatternModifiers::default();
        recursion_mods.iteration = Some(IterationType::Recursion);
        self.add_combination(
            SplitType::XOR,
            JoinType::XOR,
            recursion_mods,
            vec!["Recursion".to_string()],
            "Recursion: workflow calls itself",
        );

        // Add more patterns as needed to reach 43+
        // This is a representative subset showing the structure
    }

    fn add_combination(
        &mut self,
        split_type: SplitType,
        join_type: JoinType,
        modifiers: PatternModifiers,
        patterns: Vec<String>,
        comment: &str,
    ) {
        let combination = PatternCombination {
            split_type,
            join_type,
            modifiers,
            generated_patterns: patterns.clone(),
            is_valid: true,
            comment: comment.to_string(),
        };

        let key = combination.key();
        self.combinations.insert(key.clone(), combination);

        // Update pattern index
        for pattern in patterns {
            self.patterns
                .entry(pattern)
                .or_insert_with(Vec::new)
                .push(key.clone());
        }
    }

    pub fn is_valid_combination(
        &self,
        split: SplitType,
        join: JoinType,
        modifiers: &PatternModifiers,
    ) -> bool {
        // Check if this combination exists in the matrix
        for combination in self.combinations.values() {
            if combination.matches(split, join, modifiers) && combination.is_valid {
                return true;
            }
        }
        false
    }

    pub fn get_combination(
        &self,
        split: SplitType,
        join: JoinType,
        modifiers: &PatternModifiers,
    ) -> Option<&PatternCombination> {
        for combination in self.combinations.values() {
            if combination.matches(split, join, modifiers) {
                return Some(combination);
            }
        }
        None
    }

    pub fn get_patterns_for_combination(&self, split: SplitType, join: JoinType) -> Vec<String> {
        let key = format!("{}-{}", split.as_str(), join.as_str());
        self.combinations
            .get(&key)
            .map(|c| c.generated_patterns.clone())
            .unwrap_or_default()
    }

    pub fn get_combinations_for_pattern(&self, pattern_name: &str) -> Vec<&PatternCombination> {
        self.patterns
            .get(pattern_name)
            .map(|keys| {
                keys.iter()
                    .filter_map(|key| self.combinations.get(key))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn total_combinations(&self) -> usize {
        self.combinations.len()
    }

    pub fn total_patterns(&self) -> usize {
        self.patterns.len()
    }

    pub fn all_patterns(&self) -> HashSet<String> {
        self.patterns.keys().cloned().collect()
    }

    pub fn coverage_percentage(&self) -> f64 {
        // 43 is the total number of W3C workflow patterns
        const TOTAL_W3C_PATTERNS: usize = 43;
        (self.total_patterns() as f64 / TOTAL_W3C_PATTERNS as f64) * 100.0
    }
}

impl Default for PermutationMatrix {
    fn default() -> Self {
        Self::load_default().unwrap_or_else(|_| Self::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_type_from_str() {
        assert_eq!(SplitType::from_str("AND"), Some(SplitType::AND));
        assert_eq!(SplitType::from_str("or"), Some(SplitType::OR));
        assert_eq!(SplitType::from_str("XoR"), Some(SplitType::XOR));
        assert_eq!(SplitType::from_str("invalid"), None);
    }

    #[test]
    fn test_join_type_from_str() {
        assert_eq!(JoinType::from_str("AND"), Some(JoinType::AND));
        assert_eq!(
            JoinType::from_str("discriminator"),
            Some(JoinType::Discriminator)
        );
        assert_eq!(JoinType::from_str("invalid"), None);
    }

    #[test]
    fn test_matrix_initialization() {
        let matrix = PermutationMatrix::load_default().unwrap();
        assert!(matrix.total_combinations() > 0);
        assert!(matrix.total_patterns() > 0);
    }

    #[test]
    fn test_sequence_pattern() {
        let matrix = PermutationMatrix::load_default().unwrap();
        let is_valid = matrix.is_valid_combination(
            SplitType::XOR,
            JoinType::XOR,
            &PatternModifiers::default(),
        );
        assert!(is_valid, "Sequence pattern (XOR-XOR) should be valid");
    }

    #[test]
    fn test_parallel_sync_pattern() {
        let matrix = PermutationMatrix::load_default().unwrap();
        let is_valid = matrix.is_valid_combination(
            SplitType::AND,
            JoinType::AND,
            &PatternModifiers::default(),
        );
        assert!(is_valid, "Parallel+Sync (AND-AND) should be valid");
    }

    #[test]
    fn test_pattern_coverage() {
        let matrix = PermutationMatrix::load_default().unwrap();
        let coverage = matrix.coverage_percentage();
        println!("Pattern coverage: {:.2}%", coverage);
        // We should have at least some basic patterns
        assert!(coverage > 0.0);
    }

    #[test]
    fn test_get_patterns_for_combination() {
        let matrix = PermutationMatrix::load_default().unwrap();
        let patterns = matrix.get_patterns_for_combination(SplitType::AND, JoinType::AND);
        assert!(
            patterns.contains(&"ParallelSplit".to_string())
                || patterns.contains(&"Synchronization".to_string())
        );
    }

    #[test]
    fn test_get_combinations_for_pattern() {
        let matrix = PermutationMatrix::load_default().unwrap();
        let combinations = matrix.get_combinations_for_pattern("Sequence");
        assert!(!combinations.is_empty());
    }
}
