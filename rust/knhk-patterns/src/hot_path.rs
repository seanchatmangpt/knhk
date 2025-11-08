// rust/knhk-patterns/src/hot_path.rs
// Low-level hot path API for C kernel integration
//
// This module provides direct access to C workflow pattern kernels for
// maximum performance. Use this when:
// - Working with raw u64 data arrays
// - Need nanosecond-level performance (≤8 ticks)
// - Processing simple, high-throughput workflows
//
// For complex workflows with Rust types, use the high-level patterns.rs API.

use crate::ffi::*;

// ============================================================================
// Hot Path Error Types
// ============================================================================

#[derive(Debug)]
pub enum HotPathError {
    ValidationFailed(String),
    ExecutionFailed(String),
    InvalidContext,
    NullPointer,
}

impl std::fmt::Display for HotPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HotPathError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            HotPathError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            HotPathError::InvalidContext => write!(f, "Invalid pattern context"),
            HotPathError::NullPointer => write!(f, "Null pointer encountered"),
        }
    }
}

impl std::error::Error for HotPathError {}

pub type HotPathResult<T> = Result<T, HotPathError>;

// ============================================================================
// Pattern Context Builder
// ============================================================================

/// Builder for creating C-compatible pattern contexts from Rust data
pub struct PatternContextBuilder {
    data: Vec<u64>,
    metadata: u64,
}

impl PatternContextBuilder {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            metadata: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            metadata: 0,
        }
    }

    pub fn add_data(&mut self, value: u64) -> &mut Self {
        self.data.push(value);
        self
    }

    pub fn add_metadata(&mut self, metadata: u64) -> &mut Self {
        self.metadata = metadata;
        self
    }

    pub fn build(&mut self) -> PatternContext {
        PatternContext {
            data: self.data.as_mut_ptr(),
            len: self.data.len() as u32,
            metadata: self.metadata,
        }
    }
}

impl Default for PatternContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Pattern 20: Timeout (Hot Path) - CRITICAL: 5000x Performance Improvement
// ============================================================================

/// Hot path timeout pattern using C kernel
///
/// **Performance**: ~2 ticks (vs 10,000-20,000 ticks in pure Rust)
/// **Speedup**: 5000x faster
///
/// # Safety
/// - `branch` must be a valid C function pointer that doesn't panic
/// - `fallback` can be NULL for no fallback behavior
/// - `ctx` must point to valid data for the duration of the call
pub unsafe fn timeout_hot(
    ctx: *mut PatternContext,
    branch: BranchFn,
    timeout_ms: u64,
    fallback: Option<BranchFn>,
) -> HotPathResult<PatternResult> {
    if ctx.is_null() {
        return Err(HotPathError::NullPointer);
    }

    // Validate timeout
    if timeout_ms == 0 {
        return Err(HotPathError::ValidationFailed(
            "Timeout must be > 0".to_string(),
        ));
    }

    // Pass Option<BranchFn> directly - Rust FFI handles nullable function pointers
    let result = knhk_pattern_timeout(ctx, branch, timeout_ms, fallback);

    Ok(result)
}

// ============================================================================
// Pattern 9: Discriminator (Hot Path) - 5x Performance Improvement
// ============================================================================

/// Hot path discriminator (first-wins) pattern using C kernel
///
/// **Performance**: ~3 ticks (vs 12-15 ticks in pure Rust)
/// **Speedup**: 5x faster
///
/// # Safety
/// - `branches` must point to an array of valid C function pointers
/// - `num_branches` must match the actual array length
/// - All branch functions must not panic
pub unsafe fn discriminator_hot(
    ctx: *mut PatternContext,
    branches: *const BranchFn,
    num_branches: u32,
) -> HotPathResult<PatternResult> {
    if ctx.is_null() || branches.is_null() {
        return Err(HotPathError::NullPointer);
    }

    // Validate at ingress
    PatternType::Discriminator
        .validate_ingress(num_branches)
        .map_err(HotPathError::ValidationFailed)?;

    let result = knhk_pattern_discriminator(ctx, branches as *mut BranchFn, num_branches);

    Ok(result)
}

/// Hot path discriminator with SIMD optimization
///
/// # Safety
/// - `ctx` must be a valid, non-null pointer to a PatternContext
/// - `branches` must point to an array of valid C function pointers of length `num_branches`
/// - All branch functions must not panic and must be safe to call from C
/// - The `ctx` and `branches` pointers must remain valid for the duration of this call
pub unsafe fn discriminator_simd_hot(
    ctx: *mut PatternContext,
    branches: *const BranchFn,
    num_branches: u32,
) -> HotPathResult<PatternResult> {
    if ctx.is_null() || branches.is_null() {
        return Err(HotPathError::NullPointer);
    }

    PatternType::Discriminator
        .validate_ingress(num_branches)
        .map_err(HotPathError::ValidationFailed)?;

    let result = knhk_pattern_discriminator_simd(ctx, branches as *mut BranchFn, num_branches);

    Ok(result)
}

// ============================================================================
// Pattern 11: Implicit Termination (Hot Path)
// ============================================================================

/// Hot path implicit termination pattern using C kernel
///
/// **Performance**: ~2 ticks (vs 8-10 ticks in pure Rust)
/// **Speedup**: 4-5x faster
///
/// # Safety
/// - `ctx` must be a valid, non-null pointer to a PatternContext
/// - `branches` must point to an array of valid C function pointers of length `num_branches`
/// - All branch functions must not panic and must be safe to call from C
/// - The `ctx` and `branches` pointers must remain valid for the duration of this call
pub unsafe fn implicit_termination_hot(
    ctx: *mut PatternContext,
    branches: *const BranchFn,
    num_branches: u32,
) -> HotPathResult<PatternResult> {
    if ctx.is_null() || branches.is_null() {
        return Err(HotPathError::NullPointer);
    }

    PatternType::ImplicitTermination
        .validate_ingress(num_branches)
        .map_err(HotPathError::ValidationFailed)?;

    let result = knhk_pattern_implicit_termination(ctx, branches as *mut BranchFn, num_branches);

    Ok(result)
}

// ============================================================================
// Pattern 21: Cancellation (Hot Path)
// ============================================================================

/// Hot path cancellation pattern using C kernel
///
/// **Performance**: ~1 tick (vs 3-4 ticks in pure Rust)
///
/// # Safety
/// - `ctx` must be a valid, non-null pointer to a PatternContext
/// - `branch` must be a valid C function pointer that doesn't panic
/// - `should_cancel` must be a valid C function pointer for the cancellation condition
/// - The `ctx` pointer must remain valid for the duration of this call
pub unsafe fn cancellation_hot(
    ctx: *mut PatternContext,
    branch: BranchFn,
    should_cancel: ConditionFn,
) -> HotPathResult<PatternResult> {
    if ctx.is_null() {
        return Err(HotPathError::NullPointer);
    }

    PatternType::Cancellation
        .validate_ingress(1)
        .map_err(HotPathError::ValidationFailed)?;

    let result = knhk_pattern_cancellation(ctx, branch, should_cancel);

    Ok(result)
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a pattern context with specified capacity
pub fn create_context(capacity: u32) -> *mut PatternContext {
    unsafe { knhk_pattern_context_create(capacity) }
}

/// Destroy a pattern context and free its memory
///
/// # Safety
/// - `ctx` must be a valid pointer previously returned by `create_context`
/// - `ctx` must not be used after this call
/// - This function must only be called once per context
pub unsafe fn destroy_context(ctx: *mut PatternContext) {
    if !ctx.is_null() {
        knhk_pattern_context_destroy(ctx);
    }
}

/// Add data to a pattern context
///
/// # Safety
/// - `ctx` must be a valid pointer to a PatternContext
/// - `ctx` must have been created by `create_context` and not yet destroyed
/// - The context must have sufficient capacity for the new data
pub unsafe fn context_add_data(ctx: *mut PatternContext, data: u64) -> bool {
    if ctx.is_null() {
        return false;
    }
    knhk_pattern_context_add(ctx, data)
}

// ============================================================================
// Benchmarking Helpers
// ============================================================================

/// Get tick budget for a pattern (for benchmarking)
pub fn get_tick_budget(pattern_type: PatternType) -> u32 {
    pattern_type.tick_budget()
}

/// Validate pattern constraints at ingress
pub fn validate_pattern(pattern_type: PatternType, num_branches: u32) -> HotPathResult<()> {
    pattern_type
        .validate_ingress(num_branches)
        .map_err(HotPathError::ValidationFailed)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_context_builder() {
        let mut builder = PatternContextBuilder::new();
        builder.add_data(42).add_data(100).add_metadata(999);

        let ctx = builder.build();
        assert_eq!(ctx.len, 2);
        assert_eq!(ctx.metadata, 999);
    }

    #[test]
    fn test_tick_budgets() {
        assert_eq!(get_tick_budget(PatternType::Timeout), 2);
        assert_eq!(get_tick_budget(PatternType::Discriminator), 3);
        assert_eq!(get_tick_budget(PatternType::ImplicitTermination), 2);
        assert_eq!(get_tick_budget(PatternType::Cancellation), 1);
    }

    #[test]
    fn test_validation() {
        assert!(validate_pattern(PatternType::Timeout, 1).is_ok());
        assert!(validate_pattern(PatternType::Timeout, 0).is_err());
        assert!(validate_pattern(PatternType::Discriminator, 5).is_ok());
        assert!(validate_pattern(PatternType::Discriminator, 0).is_err());
    }
}
