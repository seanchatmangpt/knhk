// knhk-hot v1.0 — Rust hot API (FFI-safe, ≤8 ticks)
// Law: A = μ(O). Contracts: O ⊨ Σ, run.len ≤ 8, SoA aligned 64B.

#![allow(non_camel_case_types)]

pub const TICK_BUDGET: u32 = 8;
pub const NROWS: usize = 8;
pub const ALIGN: usize = 64;

// Alignment helpers (SoA must be 64B-aligned)
#[repr(align(64))]
pub struct Aligned<T>(pub T);

// Core types (KGC: Σ, Λ, τ are enforced by wrapper; μ runs in C)
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Run {
    pub pred: u64,   // P id
    pub off: u64,    // SoA offset
    pub len: u64,    // must be ≤ 8 (guarded)
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
#[allow(non_snake_case)] // RDF naming convention: S(ubject), P(redicate), O(bject)
pub struct Ctx {
    pub S: *const u64,
    pub P: *const u64,
    pub O: *const u64,
    pub run: Run,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default)]
pub enum Op {
    #[default]
    AskSp           = 1,  // Default variant for Default trait
    CountSpGe       = 2,
    AskSpo          = 3,
    CountSpLe       = 5,
    CountSpEq       = 6,
    AskOp           = 7,
    UniqueSp        = 8,
    CountOpGe       = 9,
    CountOpLe       = 10,
    CountOpEq       = 11,
    CompareOEQ      = 12,
    CompareOGT      = 13,
    CompareOLT      = 14,
    CompareOGE      = 15,
    CompareOLE      = 16,
    Construct8      = 32, // fixed-template emit, run.len ≤ 8
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
#[allow(non_snake_case)] // RDF naming: out_S/P/O match C API convention
pub struct Ir {
    pub op: Op,
    pub s: u64,
    pub p: u64,
    pub o: u64,
    pub k: u64,
    // For Construct8 only (preallocated; length = NROWS)
    pub out_S: *mut u64,
    pub out_P: *mut u64,
    pub out_O: *mut u64,
    pub out_mask: u64, // lanes written bitmask (μ sets)
    // CONSTRUCT8 pattern hint for branchless routing (set by warm path)
    pub construct8_pattern_hint: u8, // knhk_construct8_pattern_t (0 = GENERIC, 1 = ALL_NONZERO, 2-9 = LEN1-LEN8)
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Receipt {
    pub cycle_id: u64,   // Beat cycle ID (from knhk_beat_next())
    pub shard_id: u64,   // Shard identifier
    pub hook_id: u64,    // Hook identifier
    pub ticks: u32,      // Estimated/legacy ticks (for compatibility)
    pub actual_ticks: u32, // PMU-measured actual ticks (≤8 enforced by τ law)
    pub lanes: u32,      // SIMD lanes used
    pub span_id: u64,    // OTEL-compatible span ID
    pub a_hash: u64,     // hash(A) = hash(μ(O)) fragment
}

// FFI (μ-hot in C)
#[link(name = "knhk")]
extern "C" {
    // Core evaluation functions
    pub fn knhk_init_ctx(ctx: *mut Ctx, S: *const u64, P: *const u64, O: *const u64);
    pub fn knhk_pin_run(ctx: *mut Ctx, run: Run);
    pub fn knhk_eval_bool(ctx: *const Ctx, ir: *mut Ir, rcpt: *mut Receipt) -> i32;
    pub fn knhk_eval_construct8(ctx: *const Ctx, ir: *mut Ir, rcpt: *mut Receipt) -> i32;
    pub fn knhk_eval_batch8(ctx: *const Ctx, irs: *mut Ir, n: usize, rcpts: *mut Receipt) -> i32;
    
    // 8-Beat system functions
    pub fn knhk_beat_init();
    pub fn knhk_beat_next() -> u64;
    pub fn knhk_beat_tick(cycle: u64) -> u64;
    pub fn knhk_beat_pulse(cycle: u64) -> u64;
    pub fn knhk_beat_current() -> u64;
    
    // Fiber execution functions
    pub fn knhk_fiber_execute(
        ctx: *const Ctx,
        ir: *mut Ir,
        tick: u64,
        cycle_id: u64,
        shard_id: u64,
        hook_id: u64,
        receipt: *mut Receipt,
    ) -> i32; // Returns knhk_fiber_result_t: 0=SUCCESS, 1=PARKED, -1=ERROR
}

// Safe wrapper (Σ, Λ, τ, H guards)
pub struct Engine {
    ctx: Ctx,
}

impl Engine {
    /// Σ: arrays must be 64B-aligned, len = NROWS.
    /// # Safety
    /// Caller must ensure s, p, o point to valid 64B-aligned arrays of length NROWS
    pub unsafe fn new(s: *const u64, p: *const u64, o: *const u64) -> Self {
        let mut ctx = Ctx {
            S: std::ptr::null(),
            P: std::ptr::null(),
            O: std::ptr::null(),
            run: Run { pred: 0, off: 0, len: 0 },
        };
        knhk_init_ctx(&mut ctx, s, p, o);
        Self { ctx }
    }

    /// Guard H: run.len ≤ 8. Violations are rejected.
    pub fn pin_run(&mut self, run: Run) -> Result<(), &'static str> {
        if run.len > NROWS as u64 {
            return Err("H: run.len > 8 blocked");
        }
        unsafe { knhk_pin_run(&mut self.ctx, run) };
        Ok(())
    }

    /// μ: boolean reflex, τ ≤ 8. Returns bool and fills rcpt.
    pub fn eval_bool(&self, ir: &mut Ir, rcpt: &mut Receipt) -> bool {
        let r = unsafe { knhk_eval_bool(&self.ctx as *const Ctx, ir as *mut Ir, rcpt as *mut Receipt) };
        r != 0
    }

    /// μ: fixed-template emit, τ ≤ 8. Returns lanes written and fills rcpt.
    pub fn eval_construct8(&self, ir: &mut Ir, rcpt: &mut Receipt) -> usize {
        let n = unsafe { knhk_eval_construct8(&self.ctx as *const Ctx, ir as *mut Ir, rcpt as *mut Receipt) };
        n as usize
    }

    /// Λ: batch of ≤ 8 hooks in ≺-total order. τ holds per hook. rcpts len ≥ n.
    pub fn eval_batch8(&self, irs: &mut [Ir; NROWS], n: usize, rcpts: &mut [Receipt; NROWS]) -> usize {
        if n > NROWS {
            return 0;
        }
        let got = unsafe {
            knhk_eval_batch8(&self.ctx as *const Ctx, irs.as_mut_ptr(), n, rcpts.as_mut_ptr())
        };
        got as usize
    }

    pub fn ctx(&self) -> &Ctx {
        &self.ctx
    }
}

// 8-Beat system safe wrappers
pub mod beat {
    use super::*;
    
    /// Initialize beat scheduler (call once at startup)
    pub fn init() {
        unsafe {
            knhk_beat_init();
        }
    }
    
    /// Advance to next cycle and return cycle value
    /// Branchless: single atomic operation
    pub fn next() -> u64 {
        unsafe {
            knhk_beat_next()
        }
    }
    
    /// Extract tick from cycle (0..7)
    /// Branchless: bitwise mask operation
    pub fn tick(cycle: u64) -> u64 {
        unsafe {
            knhk_beat_tick(cycle)
        }
    }
    
    /// Compute pulse signal (1 when tick==0, else 0)
    /// Branchless: mask-based, no conditional branches
    pub fn pulse(cycle: u64) -> u64 {
        unsafe {
            knhk_beat_pulse(cycle)
        }
    }
    
    /// Get current cycle without incrementing
    pub fn current() -> u64 {
        unsafe {
            knhk_beat_current()
        }
    }
}

// Fiber execution safe wrappers
pub mod fiber {
    use super::*;
    
    /// Fiber execution result
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum FiberResult {
        Success,
        Parked,
        Error,
    }
    
    impl From<i32> for FiberResult {
        fn from(value: i32) -> Self {
            match value {
                0 => FiberResult::Success,
                1 => FiberResult::Parked,
                _ => FiberResult::Error,
            }
        }
    }
    
    /// Execute μ on ≤8 items at tick slot
    /// Returns FiberResult and fills receipt with provenance information
    pub fn execute(
        ctx: &Ctx,
        ir: &mut Ir,
        tick: u64,
        cycle_id: u64,
        shard_id: u64,
        hook_id: u64,
        receipt: &mut Receipt,
    ) -> FiberResult {
        unsafe {
            let result = knhk_fiber_execute(
                ctx as *const Ctx,
                ir as *mut Ir,
                tick,
                cycle_id,
                shard_id,
                hook_id,
                receipt as *mut Receipt,
            );
            result.into()
        }
    }
}

// Receipt merge (Π ⊕)
impl Receipt {
    pub fn merge(a: Receipt, b: Receipt) -> Receipt {
        Receipt {
            // Preserve identifiers from first receipt (deterministic ordering)
            cycle_id: a.cycle_id,
            shard_id: a.shard_id,
            hook_id: a.hook_id,
            // Merge metrics: max ticks (both estimated and actual)
            ticks: a.ticks.max(b.ticks),
            actual_ticks: a.actual_ticks.max(b.actual_ticks),
            lanes: a.lanes + b.lanes,
            // Merge provenance: XOR (⊕ monoid)
            span_id: a.span_id ^ b.span_id,
            a_hash: a.a_hash ^ b.a_hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt_merge() {
        let a = Receipt { cycle_id: 42, shard_id: 1, hook_id: 100, ticks: 4, actual_ticks: 3, lanes: 8, span_id: 0x1234, a_hash: 0x5678 };
        let b = Receipt { cycle_id: 43, shard_id: 2, hook_id: 200, ticks: 6, actual_ticks: 5, lanes: 8, span_id: 0xabcd, a_hash: 0xef00 };
        let merged = Receipt::merge(a, b);
        // Preserve IDs from first receipt
        assert_eq!(merged.cycle_id, 42);
        assert_eq!(merged.shard_id, 1);
        assert_eq!(merged.hook_id, 100);
        // Merge metrics
        assert_eq!(merged.ticks, 6);
        assert_eq!(merged.actual_ticks, 5);
        assert_eq!(merged.lanes, 16);
        assert_eq!(merged.span_id, 0x1234 ^ 0xabcd);
        assert_eq!(merged.a_hash, 0x5678 ^ 0xef00);
    }
}
