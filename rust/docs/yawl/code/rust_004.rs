// Warm path: Generate deterministic blank node IDs
fn generate_bnode_id(solution_id: u64, bnode_label: &str) -> u64 {
    // Deterministic: hash(solution_id, bnode_label)
    // Ensures same solution → same blank node ID
    hash64(solution_id, bnode_label)
}