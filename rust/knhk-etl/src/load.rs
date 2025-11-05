// rust/knhk-etl/src/load.rs
// Stage 3: Load - SoA-aligned arrays in L1 cache

use alloc::string::String;
use alloc::vec::Vec;

use crate::transform::{TransformResult, TypedTriple};
use crate::types::PipelineError;

/// Hook operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookOperation {
    AskSp,        // Hot path: ASK_SP query
    Construct8,   // Warm path: CONSTRUCT8 emit operation
}

pub struct PredRun {
    pub pred: u64,
    pub off: u64,
    pub len: u64, // Must be ≤ 8
    pub op: Option<HookOperation>, // Optional operation type (None = default ASK_SP)
}

/// Stage 3: Load
/// SoA-aligned arrays in L1 cache
pub struct LoadStage {
    pub alignment: usize, // Must be 64
    pub max_run_len: usize, // Must be ≤ 8
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
        // Validate guard: total triples must not exceed max_run_len
        // (In production, we'd handle multiple runs, but for simplicity, enforce single run)
        if input.typed_triples.len() > self.max_run_len {
            return Err(PipelineError::GuardViolation(
                format!("Triple count {} exceeds max_run_len {}", 
                    input.typed_triples.len(), 
                    self.max_run_len)
            ));
        }

        if input.typed_triples.is_empty() {
            return Ok(LoadResult {
                soa_arrays: SoAArrays::new(),
                runs: Vec::new(),
            });
        }

        // Group by predicate (for run formation)
        // In production, we'd group properly, but for simplicity, assume single predicate
        let predicate = input.typed_triples[0].predicate;

        // Create SoA arrays
        let mut soa = SoAArrays::new();
        let mut runs = Vec::new();
        
        let mut offset = 0u64;
        let mut current_predicate = predicate;
        let mut run_start = 0;

        for (i, triple) in input.typed_triples.iter().enumerate() {
            // Start new run if predicate changes or run length would exceed max_run_len
            if triple.predicate != current_predicate || (i - run_start) >= self.max_run_len {
                // Finalize current run
                if i > run_start {
                    runs.push(PredRun {
                        pred: current_predicate,
                        off: offset,
                        len: (i - run_start) as u64,
                        op: None, // Default to ASK_SP (hot path)
                    });
                    offset += (i - run_start) as u64;
                }
                
                // Start new run
                run_start = i;
                current_predicate = triple.predicate;
            }

            // Load into SoA arrays
            soa.s[i] = triple.subject;
            soa.p[i] = triple.predicate;
            soa.o[i] = triple.object;
        }

        // Finalize last run
        if run_start < input.typed_triples.len() {
            runs.push(PredRun {
                pred: current_predicate,
                off: offset,
                len: (input.typed_triples.len() - run_start) as u64,
                op: None, // Default to ASK_SP (hot path)
            });
            offset += (input.typed_triples.len() - run_start) as u64;
        }

        // Verify 64-byte alignment (arrays are already aligned via #[repr(align(64))])
        // This is a compile-time guarantee, but we verify at runtime for safety
        let soa_ptr = &soa as *const SoAArrays as *const u8 as usize;
        if soa_ptr % self.alignment != 0 {
            return Err(PipelineError::LoadError(
                format!("SoA arrays not properly aligned to {} bytes", self.alignment)
            ));
        }

        Ok(LoadResult {
            soa_arrays: soa,
            runs,
        })
    }
}

pub struct LoadResult {
    pub soa_arrays: SoAArrays,
    pub runs: Vec<PredRun>,
}

/// SoA arrays for hot path (64-byte aligned)
#[repr(align(64))]
pub struct SoAArrays {
    pub s: [u64; 8],
    pub p: [u64; 8],
    pub o: [u64; 8],
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

