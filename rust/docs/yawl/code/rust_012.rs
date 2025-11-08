// Warm path: Pre-select UNION branch
fn pre_select_union_branch(ctx: &Context, union: &UnionPattern) -> Vec<Solution> {
    // Evaluate each branch, select best branch
    // Return solutions from selected branch
}

// Hot path: Process pre-selected branch (no UNION evaluation)
fn construct8_from_union_solutions(
    solutions: &[Solution],  // Pre-selected from UNION
    // ... other args
) {
    // Process solutions directly (no UNION in hot path)
}