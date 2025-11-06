// knhk-warm CONSTRUCT8 implementation
// Routes CONSTRUCT8 operations to warm path with ≤500ms budget
// Production-ready implementation with proper error handling

use crate::error::WarmPathError;
use crate::WarmPathResult;
use knhk_hot::{Ctx, Ir, Op, Receipt};

/// CONSTRUCT8 warm path executor
pub struct WarmPathConstruct8;

impl WarmPathConstruct8 {
    /// Execute CONSTRUCT8 operation in warm path
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
    /// * Budget: ≤500 µs
    /// * SLO: ≤1 ms (p99)
    /// * Validates guard constraints (max_run_len ≤ 8)
    /// * Measures execution time for observability
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

        // Measure execution time (in microseconds for W1 budget)
        let start_time = Self::get_current_time_us();
        
        // Execute CONSTRUCT8 via hot path C code
        // Note: CONSTRUCT8 exceeds 8-tick budget but is still fast enough for warm path
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

