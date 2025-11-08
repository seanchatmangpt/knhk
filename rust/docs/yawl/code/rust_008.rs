// Warm path: Apply LIMIT before hot path
fn apply_limit(solutions: &mut Vec<Solution>, limit: usize) {
    solutions.truncate(limit);
}

// Hot path: Process limited solutions (no limit checking)
fn construct8_from_limited_solutions(
    solutions: &[Solution],  // Already limited
    // ... other args
) {
    // Process all solutions (limit already applied)
}