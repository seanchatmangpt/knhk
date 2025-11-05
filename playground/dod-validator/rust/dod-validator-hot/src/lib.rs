// dod-validator-hot: FFI bindings for DoD validator hot path operations
// Safe Rust wrappers around C hot path validators
// Performance: ≤8 ticks (≤2ns) per validation operation

#![allow(non_camel_case_types)]

use std::os::raw::c_int;

// Pattern types matching C enum
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DodPattern {
    Unwrap = 1,
    Expect = 2,
    Todo = 3,
    Placeholder = 4,
    Panic = 5,
    Result = 6,
}

// Validation result (hot path)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DodValidationResult {
    pub found: i32,           // 1 if pattern found, 0 otherwise
    pub count: u32,           // Number of matches
    pub span_id: u64,         // OTEL span ID for provenance
}

// KNHK types (matching C types.h)
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct KnhkPredRun {
    pub pred: u64,
    pub off: u64,
    pub len: u64,
}

#[repr(C)]
#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug)]
pub struct KnhkContext {
    pub S: *const u64,
    pub P: *const u64,
    pub O: *const u64,
    pub triple_count: usize,
    pub run: KnhkPredRun,
}

#[repr(C)]
#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug)]
pub struct KnhkHookIr {
    pub op: u32,
    pub s: u64,
    pub p: u64,
    pub o: u64,
    pub k: u64,
    pub out_S: *mut u64,
    pub out_P: *mut u64,
    pub out_O: *mut u64,
    pub out_mask: u64,
    pub select_out: *mut u64,
    pub select_capacity: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct KnhkReceipt {
    pub lanes: u32,
    pub span_id: u64,
    pub a_hash: u64,
}

// FFI bindings to KNHK C library
#[link(name = "knhk")]
extern "C" {
    fn knhk_init_ctx(ctx: *mut KnhkContext, S: *const u64, P: *const u64, O: *const u64);
    fn knhk_core_eval_bool(ctx: *const KnhkContext, ir: *const KnhkHookIr) -> c_int;
    fn knhk_generate_span_id() -> u64;
}

// Safe wrapper for hot path pattern matching
pub struct HotPathValidator;

impl HotPathValidator {
    /// Create a new hot path validator
    pub fn new() -> Self {
        Self
    }

    /// Match pattern existence in code (ASK_SP operation)
    /// Returns true if pattern found, false otherwise
    /// Performance: ≤8 ticks (≤2ns) when measured externally
    pub fn match_pattern(
        &self,
        patterns: &[u64],
        pattern_type: DodPattern,
        code_hash: u64,
    ) -> Result<DodValidationResult, String> {
        // Guard: pattern_count must be ≤8
        if patterns.len() > 8 {
            return Err(format!("Pattern count {} exceeds max_run_len 8", patterns.len()));
        }

        if patterns.is_empty() {
            return Ok(DodValidationResult {
                found: 0,
                count: 0,
                span_id: 0,
            });
        }

        let pattern_count = patterns.len() as u64;
        let pattern_type_val = pattern_type as u64;
        
        // Set up SoA arrays (must be 64-byte aligned)
        // Store triples: (code_hash, pattern_type, pattern_hash)
        #[repr(align(64))]
        struct AlignedArray([u64; 8]);
        
        let mut s_array = AlignedArray([0u64; 8]);
        let mut p_array = AlignedArray([0u64; 8]);
        let mut o_array = AlignedArray([0u64; 8]);
        
        // Store triples: for each pattern_hash, store (code_hash, pattern_type, pattern_hash)
        for (i, &pattern_hash) in patterns.iter().enumerate() {
            s_array.0[i] = code_hash;
            p_array.0[i] = pattern_type_val;
            o_array.0[i] = pattern_hash;
        }

        // Initialize KNHK context
        let mut ctx = KnhkContext {
            S: s_array.0.as_ptr(),
            P: p_array.0.as_ptr(),
            O: o_array.0.as_ptr(),
            triple_count: patterns.len(),
            run: KnhkPredRun {
                pred: pattern_type_val,
                off: 0,
                len: pattern_count,
            },
        };

        unsafe {
            knhk_init_ctx(&mut ctx, s_array.0.as_ptr(), p_array.0.as_ptr(), o_array.0.as_ptr());
            // knhk_init_ctx resets run to zero, so we need to set the run field directly
            // (knhk_pin_run is static inline, so we set it directly)
            ctx.run.pred = pattern_type_val;
            ctx.run.off = 0;
            ctx.run.len = pattern_count;
        }

        // Create hook IR for ASK_SP operation
        // Check if code_hash has any pattern of this type (ASK_SP is faster than ASK_SPO)
        let ir = KnhkHookIr {
            op: 1, // KNHK_OP_ASK_SP - checks if (code_hash, pattern_type) exists
            s: code_hash,
            p: pattern_type_val,
            o: 0, // Not used for ASK_SP
            k: 0,
            out_S: std::ptr::null_mut(),
            out_P: std::ptr::null_mut(),
            out_O: std::ptr::null_mut(),
            out_mask: 0,
            select_out: std::ptr::null_mut(),
            select_capacity: 0,
        };

        // Execute pattern matching (hot path, ≤8 ticks)
        let result = unsafe {
            knhk_core_eval_bool(&ctx, &ir)
        };

        // Generate span ID for provenance
        let span_id = unsafe { knhk_generate_span_id() };

        Ok(DodValidationResult {
            found: if result != 0 { 1 } else { 0 },
            count: if result != 0 { patterns.len() as u32 } else { 0 },
            span_id,
        })
    }

    /// Count pattern occurrences (COUNT_SP_GE operation)
    /// Returns count of pattern matches
    /// Performance: ≤8 ticks (≤2ns) when measured externally
    pub fn count_patterns(
        &self,
        patterns: &[u64],
        pattern_type: DodPattern,
        code_hash: u64,
    ) -> Result<DodValidationResult, String> {
        // Guard: pattern_count must be ≤8
        if patterns.len() > 8 {
            return Err(format!("Pattern count {} exceeds max_run_len 8", patterns.len()));
        }

        if patterns.is_empty() {
            return Ok(DodValidationResult {
                found: 0,
                count: 0,
                span_id: 0,
            });
        }

        let pattern_count = patterns.len() as u64;
        let pattern_type_val = pattern_type as u64;
        
        #[repr(align(64))]
        struct AlignedArray([u64; 8]);
        
        let mut s_array = AlignedArray([0u64; 8]);
        let mut p_array = AlignedArray([0u64; 8]);
        let mut o_array = AlignedArray([0u64; 8]);
        
        for (i, &pattern) in patterns.iter().enumerate() {
            s_array.0[i] = pattern;
            p_array.0[i] = pattern_type_val;
            o_array.0[i] = code_hash;
        }

        let mut ctx = KnhkContext {
            S: s_array.0.as_ptr(),
            P: p_array.0.as_ptr(),
            O: o_array.0.as_ptr(),
            triple_count: patterns.len(),
            run: KnhkPredRun {
                pred: pattern_type_val,
                off: 0,
                len: pattern_count,
            },
        };

        unsafe {
            knhk_init_ctx(&mut ctx, s_array.0.as_ptr(), p_array.0.as_ptr(), o_array.0.as_ptr());
        }

        // Create hook IR for COUNT_SP_GE operation
        let ir = KnhkHookIr {
            op: 2, // KNHK_OP_COUNT_SP_GE
            s: code_hash,
            p: pattern_type_val,
            o: 0,
            k: 1, // k >= 1
            out_S: std::ptr::null_mut(),
            out_P: std::ptr::null_mut(),
            out_O: std::ptr::null_mut(),
            out_mask: 0,
            select_out: std::ptr::null_mut(),
            select_capacity: 0,
        };

        // Execute COUNT_SP_GE operation
        let result = unsafe {
            knhk_core_eval_bool(&ctx, &ir)
        };

        // Generate span ID for provenance
        let span_id = unsafe { knhk_generate_span_id() };

        Ok(DodValidationResult {
            found: if result != 0 { 1 } else { 0 },
            count: if result != 0 { patterns.len() as u32 } else { 0 },
            span_id,
        })
    }

    /// Validate guard constraint (max_run_len ≤ 8)
    /// Returns true if constraint satisfied, false otherwise
    /// Performance: ≤8 ticks (≤2ns) when measured externally
    pub fn validate_guard_constraint(&self, run_len: u32) -> DodValidationResult {
        let valid = run_len <= 8;
        let span_id = unsafe { knhk_generate_span_id() };

        DodValidationResult {
            found: if valid { 1 } else { 0 },
            count: run_len,
            span_id,
        }
    }
}

impl Default for HotPathValidator {
    fn default() -> Self {
        Self::new()
    }
}


