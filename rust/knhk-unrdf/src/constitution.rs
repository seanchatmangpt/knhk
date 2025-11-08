// knhk-unrdf: Constitution validation
// Enforces constitution constraints: Typing, Order, Guard, Invariant

use crate::error::{UnrdfError, UnrdfResult};
use crate::types::HookDefinition;
use std::collections::HashSet;

/// Schema representation
/// Note: Full integration with knhk_sigma (Erlang schema registry) planned for v1.0
#[derive(Default)]
pub struct Schema {
    pub predicates: HashSet<String>,
    pub classes: HashSet<String>,
}

/// Invariants representation
/// Note: Full integration with knhk_q (Erlang invariant registry) planned for v1.0
#[derive(Default)]
pub struct Invariants {
    pub constraints: Vec<String>,
}

/// Validate hook against constitution constraints
/// Constitution: ∧(Typing, Order, Guard, Invariant)
pub fn validate_constitution(
    hook: &HookDefinition,
    schema: Option<&Schema>,
    invariants: Option<&Invariants>,
) -> UnrdfResult<()> {
    // Check Guard constraint (max_run_len ≤ 8)
    check_guard(hook)?;

    // Check Typing constraint (O ⊨ Σ)
    if let Some(schema) = schema {
        check_typing(hook, schema)?;
    }

    // Check Invariant constraint (preserve(Q))
    if let Some(invariants) = invariants {
        check_invariant(hook, invariants)?;
    }

    Ok(())
}

/// Check Guard constraint: max_run_len ≤ 8
/// Guard: μ ⊣ H (partial) - validates O ⊨ Σ before A = μ(O)
pub fn check_guard(hook: &HookDefinition) -> UnrdfResult<()> {
    // Extract query from hook definition
    let query = extract_query_from_hook(hook)?;

    // Check if query references predicates that could exceed run_len
    // For now, we validate that the query structure is valid
    // Note: Full query analysis to ensure ≤8 triples planned for v1.0

    // Basic validation: ensure query is not empty
    if query.trim().is_empty() {
        return Err(UnrdfError::GuardViolation(format!(
            "Hook {} has empty query",
            hook.id
        )));
    }

    // Validate query is ASK type (required for hooks)
    if !query.trim().to_uppercase().starts_with("ASK") {
        return Err(UnrdfError::GuardViolation(format!(
            "Hook {} query must be ASK query, got: {}",
            hook.id, query
        )));
    }

    Ok(())
}

/// Check Typing constraint: O ⊨ Σ
/// Validates that hook queries reference valid schema predicates/classes
pub fn check_typing(hook: &HookDefinition, _schema: &Schema) -> UnrdfResult<()> {
    let query = extract_query_from_hook(hook)?;

    // Extract predicates from query (basic validation for now)
    // Note: Full SPARQL parsing and predicate validation against schema planned for v1.0

    // Basic check: ensure query is well-formed
    if query.contains("?") && !query.contains("WHERE") {
        return Err(UnrdfError::TypingViolation(format!(
            "Hook {} query has variables but no WHERE clause",
            hook.id
        )));
    }

    // Planned for v1.0:
    // - Validate all predicates in query exist in schema.predicates
    // - Validate all classes referenced exist in schema.classes
    // - Validate variable bindings are type-safe

    Ok(())
}

/// Check Order constraint: Λ is ≺-total
/// Validates that hook ordering is deterministic (no cycles)
pub fn check_order(hooks: &[HookDefinition]) -> UnrdfResult<()> {
    // Check for duplicate hook IDs (violates ≺-total ordering)
    let mut seen_ids = HashSet::new();
    for hook in hooks {
        let hook_id = hook.id.clone();
        if seen_ids.contains(&hook_id) {
            return Err(UnrdfError::OrderViolation(format!(
                "Duplicate hook ID '{}' violates ≺-total ordering",
                hook_id
            )));
        }
        seen_ids.insert(hook_id);
    }

    // Planned for v1.0:
    // - Verify hook dependencies form a DAG (no cycles)
    // - Verify ordering is deterministic (same hooks always produce same order)

    Ok(())
}

/// Check Invariant constraint: preserve(Q)
/// Validates that hook preserves invariants
pub fn check_invariant(hook: &HookDefinition, _invariants: &Invariants) -> UnrdfResult<()> {
    // Planned for v1.0:
    // - Validate hook query doesn't violate any invariants
    // - Validate hook execution preserves invariant predicates
    // - Validate hook doesn't introduce contradictions

    // For now, basic validation that hook is well-formed
    let query = extract_query_from_hook(hook)?;

    if query.trim().is_empty() {
        return Err(UnrdfError::InvariantViolation(format!(
            "Hook {} has empty query, cannot preserve invariants",
            hook.id
        )));
    }

    Ok(())
}

/// Extract SPARQL query from hook definition
fn extract_query_from_hook(hook: &HookDefinition) -> UnrdfResult<String> {
    if let Some(when) = hook.definition.get("when") {
        if let Some(query) = when.get("query") {
            if let Some(query_str) = query.as_str() {
                return Ok(query_str.to_string());
            }
        }
    }

    Err(UnrdfError::ConstitutionViolation(format!(
        "Hook {} does not contain a valid SPARQL query in definition.when.query",
        hook.id
    )))
}
