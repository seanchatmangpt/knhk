// Warm path: Pre-filter graphs based on conditions
fn prefilter_graphs(ctx: &Context, conditions: &GraphConditions) -> Vec<GraphHandle> {
    ctx.graphs
        .iter()
        .filter(|(_, graph)| matches_conditions(graph, conditions))
        .map(|(_, graph)| graph.clone())
        .collect()
}

// Hot path: Process only pre-filtered graphs
fn construct8_from_filtered_graphs(
    graphs: &[GraphHandle],  // Pre-filtered graphs
    // ... other args
) {
    // Process graphs sequentially (no filtering in hot path)
}