// knhk-warm CONSTRUCT8 implementation
// Routes CONSTRUCT8 operations to warm path with ≤500ms budget
// AOT precomputation: pattern detection, constant pre-broadcast, length specialization
// Production-ready implementation with proper error handling

use crate::error::WarmPathError;
use crate::WarmPathResult;
use knhk_hot::{Ctx, Ir, Op, Receipt};

/// CONSTRUCT8 pattern detection for AOT optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Construct8Pattern {
    /// All subjects are non-zero (common in enterprise: authorization, compliance)
    AllNonZero,
    /// All subjects are zero (empty run, early return)
    AllZero,
    /// Mixed pattern (requires mask generation)
    Mixed,
}

/// CONSTRUCT8 warm path executor with AOT precomputation
pub struct WarmPathConstruct8;

impl WarmPathConstruct8 {
    /// Detect CONSTRUCT8 pattern for AOT optimization
    /// 
    /// Analyzes subject array to determine if all-nonzero, all-zero, or mixed pattern.
    /// This enables routing to specialized hot path functions.
    fn detect_pattern(ctx: &Ctx) -> Construct8Pattern {
        if ctx.run.len == 0 {
            return Construct8Pattern::AllZero;
        }
        
        let s_slice = &ctx.S[ctx.run.off..ctx.run.off + ctx.run.len];
        let all_zero = s_slice.iter().all(|&s| s == 0);
        let all_nonzero = s_slice.iter().all(|&s| s != 0);
        
        if all_zero {
            Construct8Pattern::AllZero
        } else if all_nonzero {
            Construct8Pattern::AllNonZero
        } else {
            Construct8Pattern::Mixed
        }
    }
    
    /// Execute CONSTRUCT8 operation in warm path with AOT precomputation
    /// 
    /// # Arguments
    /// * `ctx` - Hot path context (SoA arrays)
    /// * `ir` - Hook IR with CONSTRUCT8 operation
    /// 
    /// # Returns
    /// * `Ok(WarmPathResult)` - Success with timing and result metrics
    /// * `Err(WarmPathError)` - Error with descriptive message
    /// 
    /// # Performance
    /// * Budget: ≤500 µs (warm path)
    /// * Target: ≤8 ticks (hot path, via AOT optimization)
    /// * SLO: ≤1 ms (p99)
    /// * Validates guard constraints (max_run_len ≤ 8)
    /// * AOT optimizations: pattern detection, length specialization, constant pre-broadcast
    pub fn execute(
        ctx: &Ctx,
        ir: &mut Ir,
    ) -> Result<WarmPathResult, WarmPathError> {
        // Validate inputs
        if ir.op != Op::Construct8 {
            return Err(WarmPathError::InvalidInput(
                "Operation is not CONSTRUCT8".to_string()
            ));
        }

        // Validate guard constraints
        if ctx.run.len > 8 {
            return Err(WarmPathError::GuardViolation(
                format!("Run length {} exceeds max_run_len 8", ctx.run.len)
            ));
        }

        // AOT precomputation: Detect pattern for specialized routing
        let pattern = Self::detect_pattern(ctx);
        
        // Early return for all-zero pattern (0 ticks in hot path)
        if pattern == Construct8Pattern::AllZero {
            ir.out_mask = 0;
            ir.construct8_pattern_hint = 0; // KNHK_CONSTRUCT8_PATTERN_GENERIC
            return Ok(WarmPathResult::new(
                false,
                0,
                0,
                0,
            ));
        }

        // Set pattern hint for branchless routing to specialized functions
        // Pattern hint enables hot path to route to optimized variants without branches
        match pattern {
            Construct8Pattern::AllNonZero => {
                // Route to all-nonzero specialized function (skips mask generation)
                ir.construct8_pattern_hint = 1; // KNHK_CONSTRUCT8_PATTERN_ALL_NONZERO
            }
            Construct8Pattern::Mixed => {
                // Route based on length for length-specialized variants
                // Length-specialized variants have compile-time constant len_mask_bits
                match ctx.run.len {
                    1 => ir.construct8_pattern_hint = 2, // KNHK_CONSTRUCT8_PATTERN_LEN1
                    2 => ir.construct8_pattern_hint = 3, // KNHK_CONSTRUCT8_PATTERN_LEN2
                    3 => ir.construct8_pattern_hint = 4, // KNHK_CONSTRUCT8_PATTERN_LEN3
                    4 => ir.construct8_pattern_hint = 5, // KNHK_CONSTRUCT8_PATTERN_LEN4
                    5 => ir.construct8_pattern_hint = 6, // KNHK_CONSTRUCT8_PATTERN_LEN5
                    6 => ir.construct8_pattern_hint = 7, // KNHK_CONSTRUCT8_PATTERN_LEN6
                    7 => ir.construct8_pattern_hint = 8, // KNHK_CONSTRUCT8_PATTERN_LEN7
                    8 => ir.construct8_pattern_hint = 9, // KNHK_CONSTRUCT8_PATTERN_LEN8
                    _ => ir.construct8_pattern_hint = 0, // KNHK_CONSTRUCT8_PATTERN_GENERIC (fallback)
                }
            }
            Construct8Pattern::AllZero => {
                // Already handled above, but set hint for completeness
                ir.construct8_pattern_hint = 0;
            }
        }

        // Measure execution time (in microseconds for W1 budget)
        let start_time = Self::get_current_time_us();
        
        // Execute CONSTRUCT8 via hot path C code with branchless routing
        // Hot path uses dispatch table indexed by construct8_pattern_hint
        // Routes to specialized functions: all-nonzero (skips mask), len1-len8 (compile-time constants)
        let mut rcpt = Receipt::default();
        let result = unsafe {
            knhk_hot::knhk_eval_construct8(ctx, ir, &mut rcpt)
        };

        let end_time = Self::get_current_time_us();
        let latency_us = end_time.saturating_sub(start_time);

        // Check timeout (500µs budget)
        if latency_us > 500 {
            return Err(WarmPathError::TimeoutExceeded(
                format!("CONSTRUCT8 exceeded 500µs budget: {}µs", latency_us)
            ));
        }

        Ok(WarmPathResult::new(
            result > 0,
            latency_us / 1000, // Convert to milliseconds for result
            result as usize,
            rcpt.span_id,
        ))
    }

    fn get_current_time_us() -> u64 {
        #[cfg(feature = "std")]
        {
            // For timing measurements, use high-resolution timer if available
            // Fallback to system time
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_micros() as u64)
                .unwrap_or(0)
        }
        #[cfg(not(feature = "std"))]
        {
            // no_std: Timing measurement requires external time source
            // For no_std builds, timing is measured externally by the Rust framework
            // This is a known limitation for no_std builds
            0
        }
    }
}

