// Warm path: Pre-validate solutions
fn pre_validate_solutions(solutions: &mut Vec<Solution>, template: &Construct8Template) {
    solutions.retain(|s| {
        // Validate each solution against template
        validate_solution_against_template(s, template)
    });
}

// Hot path: Process pre-validated solutions (no validation)
fn construct8_from_validated_solutions(
    solutions: &[Solution],  // Pre-validated
    // ... other args
) {
    // All solutions are valid (no validation in hot path)
}