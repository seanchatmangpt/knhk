// Validation Rules
// Defines validation rules for pattern combinations and modifiers

use super::matrix::{
    CancellationType, IterationType, JoinType, PatternModifiers, SplitType,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuleError {
    #[error("Invalid split-join combination: {0}")]
    InvalidCombination(String),

    #[error("Modifier conflict: {0}")]
    ModifierConflict(String),

    #[error("Missing required modifier: {0}")]
    MissingModifier(String),

    #[error("Incompatible modifiers: {0}")]
    IncompatibleModifiers(String),
}

#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub split_type: SplitType,
    pub join_type: JoinType,
    pub modifiers: PatternModifiers,
}

pub struct ValidationRules {
    combination_rules: Vec<Box<dyn CombinationRule>>,
    modifier_rules: Vec<Box<dyn ModifierRule>>,
}

impl ValidationRules {
    pub fn new() -> Self {
        Self {
            combination_rules: Vec::new(),
            modifier_rules: Vec::new(),
        }
    }

    pub fn add_combination_rule(&mut self, rule: Box<dyn CombinationRule>) {
        self.combination_rules.push(rule);
    }

    pub fn add_modifier_rule(&mut self, rule: Box<dyn ModifierRule>) {
        self.modifier_rules.push(rule);
    }

    pub fn validate(&self, context: &ValidationContext) -> Result<Vec<String>, RuleError> {
        let mut warnings = Vec::new();

        // Validate combination rules
        for rule in &self.combination_rules {
            rule.validate(context.split_type, context.join_type)?;
        }

        // Validate modifier rules
        for rule in &self.modifier_rules {
            if let Some(warning) = rule.validate(&context.modifiers)? {
                warnings.push(warning);
            }
        }

        // Apply built-in rules
        self.validate_builtin_rules(context, &mut warnings)?;

        Ok(warnings)
    }

    fn validate_builtin_rules(
        &self,
        context: &ValidationContext,
        warnings: &mut Vec<String>,
    ) -> Result<(), RuleError> {
        // Rule: OR split requires OR join or XOR join
        if context.split_type == SplitType::OR && context.join_type == JoinType::AND {
            return Err(RuleError::InvalidCombination(
                "OR split with AND join is typically invalid without specific modifiers".to_string(),
            ));
        }

        // Rule: XOR split with AND join is invalid
        if context.split_type == SplitType::XOR && context.join_type == JoinType::AND {
            return Err(RuleError::InvalidCombination(
                "XOR split cannot have AND join (only one path active)".to_string(),
            ));
        }

        // Rule: Discriminator join requires quorum
        if context.join_type == JoinType::Discriminator && context.modifiers.quorum.is_none() {
            warnings.push("Discriminator join typically requires quorum setting".to_string());
        }

        // Rule: Backward flow suggests iteration
        if context.modifiers.backward_flow && context.modifiers.iteration.is_none() {
            warnings.push("Backward flow without iteration type may cause unbounded loops".to_string());
        }

        // Rule: Deferred choice requires runtime decision
        if context.modifiers.deferred_choice {
            // Ensure this is compatible with the split type
            if context.split_type != SplitType::XOR && context.split_type != SplitType::OR {
                return Err(RuleError::ModifierConflict(
                    "Deferred choice requires XOR or OR split".to_string(),
                ));
            }
        }

        // Rule: Critical section requires synchronization
        if context.modifiers.critical_section {
            if context.join_type != JoinType::AND {
                warnings.push("Critical section typically requires AND join for proper synchronization".to_string());
            }
        }

        // Rule: Interleaving requires parallel execution
        if context.modifiers.interleaving {
            if context.split_type != SplitType::AND {
                return Err(RuleError::ModifierConflict(
                    "Interleaving requires AND split (parallel execution)".to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self::new()
    }
}

pub trait CombinationRule: Send + Sync {
    fn validate(&self, split: SplitType, join: JoinType) -> Result<(), RuleError>;
}

pub trait ModifierRule: Send + Sync {
    fn validate(&self, modifiers: &PatternModifiers) -> Result<Option<String>, RuleError>;
}

// Example combination rule: Parallel requires AND split
pub struct ParallelRequiresAndSplit;

impl CombinationRule for ParallelRequiresAndSplit {
    fn validate(&self, split: SplitType, join: JoinType) -> Result<(), RuleError> {
        if join == JoinType::AND && split != SplitType::AND {
            return Err(RuleError::InvalidCombination(
                "AND join requires AND split for proper synchronization".to_string(),
            ));
        }
        Ok(())
    }
}

// Example modifier rule: Quorum must be > 0
pub struct QuorumMustBePositive;

impl ModifierRule for QuorumMustBePositive {
    fn validate(&self, modifiers: &PatternModifiers) -> Result<Option<String>, RuleError> {
        if let Some(quorum) = modifiers.quorum {
            if quorum == 0 {
                return Err(RuleError::ModifierConflict(
                    "Quorum must be greater than 0".to_string(),
                ));
            }
        }
        Ok(None)
    }
}

// Example modifier rule: Cancellation type validation
pub struct CancellationTypeValidation;

impl ModifierRule for CancellationTypeValidation {
    fn validate(&self, modifiers: &PatternModifiers) -> Result<Option<String>, RuleError> {
        if let Some(cancel_type) = &modifiers.cancellation {
            match cancel_type {
                CancellationType::Task => Ok(Some(
                    "Task cancellation affects only the specific task".to_string(),
                )),
                CancellationType::Case => Ok(Some(
                    "Case cancellation affects the entire workflow instance".to_string(),
                )),
                CancellationType::Region => Ok(Some(
                    "Region cancellation affects a defined scope".to_string(),
                )),
            }
        } else {
            Ok(None)
        }
    }
}

// Example modifier rule: Iteration type validation
pub struct IterationTypeValidation;

impl ModifierRule for IterationTypeValidation {
    fn validate(&self, modifiers: &PatternModifiers) -> Result<Option<String>, RuleError> {
        if let Some(iter_type) = &modifiers.iteration {
            match iter_type {
                IterationType::StructuredLoop => {
                    Ok(Some("Structured loop requires iteration count or condition".to_string()))
                }
                IterationType::Recursion => Ok(Some(
                    "Recursion must have termination condition to prevent infinite loops"
                        .to_string(),
                )),
            }
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_rules_valid_combination() {
        let rules = ValidationRules::default();
        let context = ValidationContext {
            split_type: SplitType::AND,
            join_type: JoinType::AND,
            modifiers: PatternModifiers::default(),
        };
        let result = rules.validate(&context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_builtin_rules_invalid_xor_and() {
        let rules = ValidationRules::default();
        let context = ValidationContext {
            split_type: SplitType::XOR,
            join_type: JoinType::AND,
            modifiers: PatternModifiers::default(),
        };
        let result = rules.validate(&context);
        assert!(result.is_err());
    }

    #[test]
    fn test_discriminator_quorum_warning() {
        let rules = ValidationRules::default();
        let context = ValidationContext {
            split_type: SplitType::AND,
            join_type: JoinType::Discriminator,
            modifiers: PatternModifiers::default(),
        };
        let result = rules.validate(&context);
        assert!(result.is_ok());
        let warnings = result.unwrap();
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("quorum"));
    }

    #[test]
    fn test_interleaving_requires_and_split() {
        let rules = ValidationRules::default();
        let mut modifiers = PatternModifiers::default();
        modifiers.interleaving = true;
        let context = ValidationContext {
            split_type: SplitType::XOR,
            join_type: JoinType::XOR,
            modifiers,
        };
        let result = rules.validate(&context);
        assert!(result.is_err());
    }

    #[test]
    fn test_deferred_choice_with_xor() {
        let rules = ValidationRules::default();
        let mut modifiers = PatternModifiers::default();
        modifiers.deferred_choice = true;
        let context = ValidationContext {
            split_type: SplitType::XOR,
            join_type: JoinType::XOR,
            modifiers,
        };
        let result = rules.validate(&context);
        assert!(result.is_ok());
    }
}
