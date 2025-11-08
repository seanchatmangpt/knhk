// Warm path: Pre-filter solutions
fn pre_filter_solutions(solutions: &mut Vec<Solution>, filter: &FilterExpr) {
    solutions.retain(|s| evaluate_filter(s, filter));
}

// Hot path: Process pre-filtered solutions (no filtering)
fn construct8_from_filtered_solutions(
    solutions: &[Solution],  // Pre-filtered
    // ... other args
) {
    // Process all solutions (filter already applied)
}