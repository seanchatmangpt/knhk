pub fn preprocess_construct8(
    analysis: &TemplateAnalysis,
    ctx: &Context,
    solutions: &[Solution],
) -> Construct8Input {
    // 1. Emit ground triples (once)
    let mut output = emit_ground_triples(&analysis.ground_triples);
    
    // 2. Apply solution modifiers
    let mut processed_solutions = apply_modifiers(solutions, &analysis.modifiers);
    
    // 3. Pre-validate solutions
    let validated_solutions = pre_validate(&processed_solutions, &analysis);
    
    // 4. Pre-bind variables
    let bound_data = pre_bind_variables(&validated_solutions, &analysis.variables);
    
    // 5. Pre-allocate blank nodes
    let bnode_ids = pre_allocate_blank_nodes(&analysis.blank_nodes, validated_solutions.len());
    
    // 6. Prepare hot path input
    Construct8Input {
        S: bound_data.subjects,
        P: bound_data.predicates,
        O: bound_data.objects,
        len: validated_solutions.len().min(8),
        p_const: analysis.constants.predicate,
        o_const: analysis.constants.object,
        bnode_ids,
    }
}