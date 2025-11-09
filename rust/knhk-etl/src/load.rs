// rust/knhk-etl/src/load.rs
// Stage 3: Load
// SoA-aligned arrays in L1 cache

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::vec::Vec;

use crate::error::PipelineError;
use crate::transform::{TransformResult, TypedTriple};
use chicago_tdd_tools::guards::GuardValidator;

/// Stage 3: Load
/// SoA-aligned arrays in L1 cache
/// Load stage for converting triples to SoA (Structure of Arrays) format
///
/// Converts transformed triples into SoA format for efficient SIMD processing
/// and hot path execution (≤8 ticks per operation).
pub struct LoadStage {
    pub alignment: usize,   // Must be 64
    pub max_run_len: usize, // Must be ≤ 8
}

impl Default for LoadStage {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadStage {
    pub fn new() -> Self {
        Self {
            alignment: 64,
            max_run_len: 8,
        }
    }

    /// Load triples into SoA arrays
    ///
    /// Production implementation:
    /// 1. Group by predicate (for run formation)
    /// 2. Ensure run.len ≤ 8
    /// 3. Align to 64-byte boundaries
    /// 4. Prepare SoA arrays
    pub fn load(&self, input: TransformResult) -> Result<LoadResult, PipelineError> {
        // Guard validation at ingress: MAX_RUN_LEN ≤ 8
        // Validate before passing to execution paths (defensive checks removed from hot path)
        let guard_validator = GuardValidator::new();
        guard_validator
            .validate_run(&input.typed_triples)
            .map_err(|e| {
                PipelineError::GuardViolation(format!(
                    "Guard constraint violation at Load stage ingress: {}",
                    e
                ))
            })?;

        if input.typed_triples.is_empty() {
            return Ok(LoadResult {
                soa_arrays: SoAArrays::new(),
                runs: Vec::new(),
            });
        }

        // Group triples by predicate (for run formation)
        let mut grouped_by_predicate: BTreeMap<u64, Vec<&TypedTriple>> = BTreeMap::new();
        for triple in &input.typed_triples {
            grouped_by_predicate
                .entry(triple.predicate)
                .or_default()
                .push(triple);
        }

        // Create SoA arrays and runs
        let mut soa = SoAArrays::new();
        let mut runs = Vec::new();
        let mut offset = 0u64;

        for (predicate, triples) in grouped_by_predicate {
            // Guard validation: run length ≤ 8 (validated at ingress, but double-check here for safety)
            guard_validator
                .validate_run_len(triples.len())
                .map_err(|e| {
                    PipelineError::GuardViolation(format!("Predicate run guard violation: {}", e))
                })?;

            // Ensure we don't exceed SoA array capacity
            if offset as usize + triples.len() > 8 {
                return Err(PipelineError::LoadError(
                    "Total triples exceed SoA capacity of 8".to_string(),
                ));
            }

            // Load triples into SoA arrays
            for (i, triple) in triples.iter().enumerate() {
                let idx = offset as usize + i;
                soa.s[idx] = triple.subject;
                soa.p[idx] = triple.predicate;
                soa.o[idx] = triple.object;
            }

            // Create run metadata
            runs.push(PredRun {
                pred: predicate,
                off: offset,
                len: triples.len() as u64,
            });

            offset += triples.len() as u64;
        }

        // Verify 64-byte alignment (arrays are already aligned via #[repr(align(64))])
        // This is a compile-time guarantee, but we verify at runtime for safety
        let soa_ptr = &soa as *const SoAArrays as *const u8 as usize;
        if !soa_ptr.is_multiple_of(self.alignment) {
            return Err(PipelineError::LoadError(format!(
                "SoA arrays not properly aligned to {} bytes",
                self.alignment
            )));
        }

        Ok(LoadResult {
            soa_arrays: soa,
            runs,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LoadResult {
    pub soa_arrays: SoAArrays,
    pub runs: Vec<PredRun>,
}

/// SoA arrays for hot path (64-byte aligned)
#[repr(align(64))]
#[derive(Debug, Clone)]
pub struct SoAArrays {
    pub s: [u64; 8],
    pub p: [u64; 8],
    pub o: [u64; 8],
}

impl Default for SoAArrays {
    fn default() -> Self {
        Self::new()
    }
}

impl SoAArrays {
    pub fn new() -> Self {
        Self {
            s: [0; 8],
            p: [0; 8],
            o: [0; 8],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PredRun {
    pub pred: u64,
    pub off: u64,
    pub len: u64, // Must be ≤ 8
}
