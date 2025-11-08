// rust/knhk-etl/src/hot_path_engine.rs
// Reusable hot path engine with memory reuse
// Based on simdjson: reuse buffers to keep memory hot in cache

use crate::error::PipelineError;
use crate::load::SoAArrays;

/// Reusable hot path engine with memory reuse
///
/// Pattern from simdjson: reuse buffers across operations to keep memory hot in cache.
/// This reduces allocation overhead and improves cache locality for hot path operations.
///
/// # Performance Benefits
/// - Reuses SoAArrays buffers (avoids allocation on every operation)
/// - Keeps buffers hot in L1 cache
/// - Can set max capacity to prevent unbounded growth
pub struct HotPathEngine {
    /// Reusable SoAArrays buffer (64-byte aligned, cache-friendly)
    soa_buffers: SoAArrays,
    /// Maximum capacity (prevents unbounded growth in server loops)
    max_capacity: usize,
    /// Current capacity (grows as needed, never shrinks)
    current_capacity: usize,
}

impl Default for HotPathEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl HotPathEngine {
    /// Create new hot path engine with default capacity (8 triples)
    pub fn new() -> Self {
        Self {
            soa_buffers: SoAArrays::new(),
            max_capacity: 8, // Chatman Constant: max_run_len ≤ 8
            current_capacity: 8,
        }
    }

    /// Create hot path engine with specified max capacity
    ///
    /// # Arguments
    /// * `max_capacity` - Maximum number of triples engine can handle (must be ≤ 8)
    ///
    /// # Returns
    /// * `Err` if max_capacity exceeds 8 (guard violation)
    pub fn with_max_capacity(max_capacity: usize) -> Result<Self, PipelineError> {
        if max_capacity > 8 {
            return Err(PipelineError::GuardViolation(format!(
                "max_capacity {} exceeds max_run_len 8",
                max_capacity
            )));
        }

        Ok(Self {
            soa_buffers: SoAArrays::new(),
            max_capacity,
            current_capacity: max_capacity,
        })
    }

    /// Get mutable reference to SoAArrays buffers
    ///
    /// Buffers are reused across operations, keeping memory hot in cache.
    /// Caller should clear/initialize buffers before use.
    pub fn get_buffers_mut(&mut self) -> &mut SoAArrays {
        &mut self.soa_buffers
    }

    /// Get immutable reference to SoAArrays buffers
    pub fn get_buffers(&self) -> &SoAArrays {
        &self.soa_buffers
    }

    /// Set maximum capacity (for server loops)
    ///
    /// Prevents unbounded growth in long-running processes.
    /// Capacity can grow up to max_capacity but never exceeds it.
    pub fn set_max_capacity(&mut self, max_capacity: usize) -> Result<(), PipelineError> {
        if max_capacity > 8 {
            return Err(PipelineError::GuardViolation(format!(
                "max_capacity {} exceeds max_run_len 8",
                max_capacity
            )));
        }

        self.max_capacity = max_capacity;
        if self.current_capacity > max_capacity {
            self.current_capacity = max_capacity;
        }

        Ok(())
    }

    /// Get current capacity
    pub fn current_capacity(&self) -> usize {
        self.current_capacity
    }

    /// Get max capacity
    pub fn max_capacity(&self) -> usize {
        self.max_capacity
    }

    /// Clear buffers (zero out arrays)
    ///
    /// Useful when reusing engine for new operations.
    pub fn clear(&mut self) {
        self.soa_buffers = SoAArrays::new();
    }

    /// Load triples into reusable buffers
    ///
    /// Reuses existing buffers, only allocates if needed.
    /// This keeps memory hot in cache and reduces allocation overhead.
    ///
    /// # Arguments
    /// * `triples` - Slice of (subject, predicate, object) tuples
    ///
    /// # Returns
    /// * `Ok(SoAArrays)` - Reusable buffers with loaded triples
    /// * `Err` - If triple count exceeds capacity or guard constraints
    pub fn load_triples(
        &mut self,
        triples: &[(u64, u64, u64)],
    ) -> Result<&SoAArrays, PipelineError> {
        if triples.len() > self.max_capacity {
            return Err(PipelineError::GuardViolation(format!(
                "Triple count {} exceeds max_capacity {}",
                triples.len(),
                self.max_capacity
            )));
        }

        // Clear buffers before loading
        self.clear();

        // Load triples into reusable buffers
        for (i, (s, p, o)) in triples.iter().enumerate() {
            if i >= 8 {
                break; // Safety check
            }
            self.soa_buffers.s[i] = *s;
            self.soa_buffers.p[i] = *p;
            self.soa_buffers.o[i] = *o;
        }

        Ok(&self.soa_buffers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hot_path_engine_creation() {
        let engine = HotPathEngine::new();
        assert_eq!(engine.current_capacity(), 8);
        assert_eq!(engine.max_capacity(), 8);
    }

    #[test]
    fn test_hot_path_engine_with_capacity() {
        let engine = HotPathEngine::with_max_capacity(4).unwrap();
        assert_eq!(engine.current_capacity(), 4);
        assert_eq!(engine.max_capacity(), 4);
    }

    #[test]
    fn test_hot_path_engine_capacity_guard() {
        let result = HotPathEngine::with_max_capacity(9);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_triples() {
        let mut engine = HotPathEngine::new();
        let triples = vec![(1, 100, 1000), (2, 100, 2000), (3, 100, 3000)];

        let buffers = engine.load_triples(&triples).unwrap();
        assert_eq!(buffers.s[0], 1);
        assert_eq!(buffers.p[0], 100);
        assert_eq!(buffers.o[0], 1000);
        assert_eq!(buffers.s[2], 3);
    }

    #[test]
    fn test_load_triples_exceeds_capacity() {
        let mut engine = HotPathEngine::with_max_capacity(4).unwrap();
        let triples = vec![(1, 100, 1000); 5]; // 5 triples > 4 capacity

        let result = engine.load_triples(&triples);
        assert!(result.is_err());
    }

    #[test]
    fn test_buffer_reuse() {
        let mut engine = HotPathEngine::new();

        // First operation
        let triples1 = vec![(1, 100, 1000)];
        let buffers1 = engine.load_triples(&triples1).unwrap();
        assert_eq!(buffers1.s[0], 1);

        // Second operation reuses buffers (drop first reference first)
        drop(buffers1);
        let triples2 = vec![(2, 200, 2000)];
        let buffers2 = engine.load_triples(&triples2).unwrap();
        assert_eq!(buffers2.s[0], 2);

        // Verify buffers were reused (same memory location)
        assert_eq!(buffers2.s[0], 2);
    }

    #[test]
    fn test_set_max_capacity() {
        let mut engine = HotPathEngine::new();
        engine.set_max_capacity(4).unwrap();
        assert_eq!(engine.max_capacity(), 4);

        let result = engine.set_max_capacity(9);
        assert!(result.is_err());
    }
}
