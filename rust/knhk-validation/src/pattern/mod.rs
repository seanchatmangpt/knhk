// Pattern Matrix Validator Module
// Enforces Covenant 4: All Patterns Expressible via Permutations

pub mod validator;
pub mod matrix;
pub mod rules;

pub use validator::{PatternValidator, ValidationError, ValidationResult};
pub use matrix::{PermutationMatrix, PatternCombination, SplitType, JoinType, PatternModifiers};
pub use rules::{ValidationRules, CombinationRule, ModifierRule};
