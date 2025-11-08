// Rust warm path: Precompute ground triples
pub struct ConstructTemplate {
    pub ground_triples: Vec<(u64, u64, u64)>,  // (s, p, o) constants
    pub template_triples: Vec<TemplateTriple>, // Variable triples
}

// Warm path: Emit ground triples once
fn emit_ground_triples(template: &ConstructTemplate, output: &mut Vec<Quad>) {
    for (s, p, o) in &template.ground_triples {
        output.push(Quad { s: *s, p: *p, o: *o, g: 0 });
    }
}

// Hot path: Only emit variable triples
fn construct8_variable_triples(...) {
    // Only processes variable triples (≤8)
}