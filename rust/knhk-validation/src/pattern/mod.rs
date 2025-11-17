// Pattern Matrix Validator Module
// Enforces Covenant 4: All Patterns Expressible via Permutations

pub mod matrix;
pub mod rules;
pub mod validator;

pub use matrix::{JoinType, PatternCombination, PatternModifiers, PermutationMatrix, SplitType};
pub use rules::{CombinationRule, ModifierRule, ValidationRules};
pub use validator::{PatternValidator, ValidationError, ValidationResult};
