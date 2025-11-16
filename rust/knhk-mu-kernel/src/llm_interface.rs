//! LLM Interface (μ_cold - only for ΔΣ generation)

/// LLM generates ΔΣ proposals, never executes in hot path
pub mod llm {
    use alloc::vec::Vec;
    use alloc::vec;
    use crate::overlay::DeltaSigma;

    /// Generate overlay proposals from LLM
    pub fn generate_overlay_proposals(_context: &str) -> Vec<DeltaSigma> {
        // Would call LLM API to generate ΔΣ
        // LLMs are pattern generators, not executors
        vec![]
    }
}
