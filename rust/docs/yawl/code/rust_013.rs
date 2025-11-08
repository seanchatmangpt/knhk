// Rust AOT: Generate CONSTRUCT8 template
pub fn generate_construct8_template(template: &str) -> Construct8Template {
    let triples = parse_template_triples(template);
    
    assert!(triples.len() <= 8, "CONSTRUCT8 template must have ≤8 triples");
    
    Construct8Template {
        triples,
        // Precompute constant values
        constants: extract_constants(&triples),
        // Precompute variable positions
        variable_positions: extract_variable_positions(&triples),
    }
}

// Warm path: Bind variables, prepare hot path input
fn prepare_construct8_input(
    template: &Construct8Template,
    solutions: &[Solution],
) -> Construct8Input {
    // Bind variables from solutions
    // Prepare S, P, O arrays for hot path
    // Precompute constants
    Construct8Input {
        S: bind_subjects(template, solutions),
        P: bind_predicates(template, solutions),  // Usually constant
        O: bind_objects(template, solutions),
        len: solutions.len().min(8),
    }
}

// Hot path: Execute CONSTRUCT8 (≤8 ticks)
fn construct8_execute(input: &Construct8Input, output: &mut Construct8Output) {
    knhk_construct8_emit_8(
        input.S.as_ptr(),
        input.off,
        input.len,
        input.p_const,  // Usually constant
        input.o_const,  // May be constant
        output.out_S.as_mut_ptr(),
        output.out_P.as_mut_ptr(),
        output.out_O.as_mut_ptr(),
    );
}