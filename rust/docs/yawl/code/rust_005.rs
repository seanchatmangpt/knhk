// Rust AOT: Analyze GRAPH clause
pub fn analyze_graph_clause(where: &str) -> GraphAccess {
    if let Some(graph_iri) = extract_constant_graph(where) {
        GraphAccess::Constant(graph_iri)  // Hardcode graph IRI
    } else {
        GraphAccess::Variable  // Variable graph (rare)
    }
}

// Warm path: Pre-select graph
fn select_graph(ctx: &Context, graph_iri: u64) -> Option<GraphHandle> {
    ctx.graphs.get(&graph_iri)  // O(1) lookup
}

// Hot path: Use pre-selected graph handle
fn construct8_from_graph(
    graph: &GraphHandle,  // Pre-selected graph
    // ... other args
) {
    // Direct access to graph data (no lookup in hot path)
}