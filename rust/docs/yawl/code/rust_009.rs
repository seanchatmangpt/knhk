// Warm path: Apply OFFSET before hot path
fn apply_offset(solutions: &mut Vec<Solution>, offset: usize) {
    solutions.drain(..offset);
}

// Hot path: Process offset solutions (no offset checking)
fn construct8_from_offset_solutions(
    solutions: &[Solution],  // Already offset
    // ... other args
) {
    // Process all solutions (offset already applied)
}