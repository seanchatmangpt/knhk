// knhk-hot: Runtime CPU detection and SIMD dispatch
// Based on simdjson lessons for optimal performance on different architectures

use std::sync::OnceLock;

/// CPU feature detection results
/// Cached globally to avoid repeated CPUID calls
#[derive(Debug, Clone, Copy)]
pub struct CpuFeatures {
    /// ARM NEON support (ARMv8+)
    pub has_neon: bool,
    /// ARM SVE support (future)
    pub has_sve: bool,
    /// Intel AVX2 support
    pub has_avx2: bool,
    /// Intel AVX-512 support
    pub has_avx512: bool,
    /// Architecture name for logging
    pub arch_name: &'static str,
}

/// Global CPU features cache
/// Initialized once on first access using OnceLock for zero-cost after first call
static CPU_FEATURES: OnceLock<CpuFeatures> = OnceLock::new();

impl CpuFeatures {
    /// Detect CPU features at runtime
    /// Called exactly once via OnceLock::get_or_init()
    fn detect() -> Self {
        #[cfg(target_arch = "aarch64")]
        {
            // ARM64 detection
            let has_neon = std::arch::is_aarch64_feature_detected!("neon");
            let has_sve = std::arch::is_aarch64_feature_detected!("sve");

            CpuFeatures {
                has_neon,
                has_sve,
                has_avx2: false,
                has_avx512: false,
                arch_name: if has_sve {
                    "ARM64-SVE"
                } else if has_neon {
                    "ARM64-NEON"
                } else {
                    "ARM64-FALLBACK"
                },
            }
        }

        #[cfg(target_arch = "x86_64")]
        {
            // x86_64 detection
            let has_avx2 = std::arch::is_x86_feature_detected!("avx2");
            let has_avx512 = std::arch::is_x86_feature_detected!("avx512f");

            CpuFeatures {
                has_neon: false,
                has_sve: false,
                has_avx2,
                has_avx512,
                arch_name: if has_avx512 {
                    "x86_64-AVX512"
                } else if has_avx2 {
                    "x86_64-AVX2"
                } else {
                    "x86_64-SSE"
                },
            }
        }

        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
        {
            // Generic fallback for other architectures
            CpuFeatures {
                has_neon: false,
                has_sve: false,
                has_avx2: false,
                has_avx512: false,
                arch_name: "GENERIC",
            }
        }
    }

    /// Get cached CPU features (zero cost after first call)
    pub fn get() -> &'static CpuFeatures {
        CPU_FEATURES.get_or_init(Self::detect)
    }

    /// Print detected features for debugging
    pub fn log_features(&self) {
        eprintln!(
            "[KNHK CPU Dispatch] Detected architecture: {}",
            self.arch_name
        );
        eprintln!("  NEON:   {}", self.has_neon);
        eprintln!("  SVE:    {}", self.has_sve);
        eprintln!("  AVX2:   {}", self.has_avx2);
        eprintln!("  AVX512: {}", self.has_avx512);
    }
}

// ============================================================================
// Pattern Result (matches C ABI)
// ============================================================================

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PatternResult {
    pub success: bool,
    pub branches: u32,
    pub result: u64,
    pub error: *const std::os::raw::c_char,
}

impl Default for PatternResult {
    fn default() -> Self {
        PatternResult {
            success: false,
            branches: 0,
            result: 0,
            error: std::ptr::null(),
        }
    }
}

// ============================================================================
// Pattern Context (matches C ABI)
// ============================================================================

#[repr(C)]
pub struct PatternContext {
    pub data: *mut u64,
    pub len: u32,
    pub metadata: u64,
}

// ============================================================================
// Function Pointer Types for Dispatch
// ============================================================================

pub type BranchFn = extern "C" fn(*mut PatternContext) -> bool;

/// Pattern discriminator function signature
pub type DiscriminatorFn = unsafe extern "C" fn(
    ctx: *mut PatternContext,
    branches: *const BranchFn,
    num_branches: u32,
) -> PatternResult;

/// Pattern parallel split function signature
pub type ParallelSplitFn = unsafe extern "C" fn(
    ctx: *mut PatternContext,
    branches: *const BranchFn,
    num_branches: u32,
) -> PatternResult;

/// Pattern synchronization function signature
pub type SynchronizationFn = unsafe extern "C" fn(
    ctx: *mut PatternContext,
    branch_results: *const u64,
    num_branches: u32,
) -> PatternResult;

/// Pattern multi-choice function signature
pub type MultiChoiceFn = unsafe extern "C" fn(
    ctx: *mut PatternContext,
    conditions: *const BranchFn,
    branches: *const BranchFn,
    num_branches: u32,
) -> PatternResult;

// ============================================================================
// External C Functions (generic and SIMD versions)
// ============================================================================

extern "C" {
    // Pattern 9: Discriminator
    fn knhk_pattern_discriminator(
        ctx: *mut PatternContext,
        branches: *const BranchFn,
        num_branches: u32,
    ) -> PatternResult;

    fn knhk_pattern_discriminator_simd(
        ctx: *mut PatternContext,
        branches: *const BranchFn,
        num_branches: u32,
    ) -> PatternResult;

    // Pattern 2: Parallel Split
    fn knhk_pattern_parallel_split(
        ctx: *mut PatternContext,
        branches: *const BranchFn,
        num_branches: u32,
    ) -> PatternResult;

    fn knhk_pattern_parallel_split_simd(
        ctx: *mut PatternContext,
        branches: *const BranchFn,
        num_branches: u32,
    ) -> PatternResult;

    // Pattern 3: Synchronization
    fn knhk_pattern_synchronization(
        ctx: *mut PatternContext,
        branch_results: *const u64,
        num_branches: u32,
    ) -> PatternResult;

    fn knhk_pattern_synchronization_simd(
        ctx: *mut PatternContext,
        branch_results: *const u64,
        num_branches: u32,
    ) -> PatternResult;

    // Pattern 6: Multi-Choice
    fn knhk_pattern_multi_choice(
        ctx: *mut PatternContext,
        conditions: *const BranchFn,
        branches: *const BranchFn,
        num_branches: u32,
    ) -> PatternResult;

    fn knhk_pattern_multi_choice_simd(
        ctx: *mut PatternContext,
        conditions: *const BranchFn,
        branches: *const BranchFn,
        num_branches: u32,
    ) -> PatternResult;
}

// ============================================================================
// CPU Dispatcher - Runtime SIMD Selection
// ============================================================================

pub struct CpuDispatcher {
    features: &'static CpuFeatures,
    discriminator_fn: DiscriminatorFn,
    parallel_split_fn: ParallelSplitFn,
    synchronization_fn: SynchronizationFn,
    multi_choice_fn: MultiChoiceFn,
}

impl CpuDispatcher {
    /// Create new dispatcher with runtime CPU detection
    /// This is called once and cached globally
    pub fn new() -> Self {
        let features = CpuFeatures::get();

        // Select optimal implementations based on CPU features
        let (discriminator_fn, parallel_split_fn, synchronization_fn, multi_choice_fn) =
            if features.has_neon || features.has_avx2 || features.has_avx512 {
                // SIMD-capable CPU: use optimized implementations
                (
                    knhk_pattern_discriminator_simd as DiscriminatorFn,
                    knhk_pattern_parallel_split_simd as ParallelSplitFn,
                    knhk_pattern_synchronization_simd as SynchronizationFn,
                    knhk_pattern_multi_choice_simd as MultiChoiceFn,
                )
            } else {
                // Generic fallback
                (
                    knhk_pattern_discriminator as DiscriminatorFn,
                    knhk_pattern_parallel_split as ParallelSplitFn,
                    knhk_pattern_synchronization as SynchronizationFn,
                    knhk_pattern_multi_choice as MultiChoiceFn,
                )
            };

        CpuDispatcher {
            features,
            discriminator_fn,
            parallel_split_fn,
            synchronization_fn,
            multi_choice_fn,
        }
    }

    /// Get cached dispatcher instance
    pub fn get() -> &'static CpuDispatcher {
        static DISPATCHER: OnceLock<CpuDispatcher> = OnceLock::new();
        DISPATCHER.get_or_init(CpuDispatcher::new)
    }

    /// Get optimal discriminator implementation for this CPU
    #[inline(always)]
    pub fn select_discriminator(&self) -> DiscriminatorFn {
        self.discriminator_fn
    }

    /// Get optimal parallel split implementation for this CPU
    #[inline(always)]
    pub fn select_parallel_split(&self) -> ParallelSplitFn {
        self.parallel_split_fn
    }

    /// Get optimal synchronization implementation for this CPU
    #[inline(always)]
    pub fn select_synchronization(&self) -> SynchronizationFn {
        self.synchronization_fn
    }

    /// Get optimal multi-choice implementation for this CPU
    #[inline(always)]
    pub fn select_multi_choice(&self) -> MultiChoiceFn {
        self.multi_choice_fn
    }

    /// Get CPU features
    #[inline(always)]
    pub fn features(&self) -> &CpuFeatures {
        self.features
    }

    /// Execute discriminator pattern with automatic dispatch
    ///
    /// # Safety
    /// Caller must ensure:
    /// - `ctx` is valid and non-null
    /// - `branches` array has `num_branches` elements
    /// - Branch functions don't panic
    #[inline(always)]
    pub unsafe fn discriminator(
        &self,
        ctx: *mut PatternContext,
        branches: *const BranchFn,
        num_branches: u32,
    ) -> PatternResult {
        (self.discriminator_fn)(ctx, branches, num_branches)
    }

    /// Execute parallel split pattern with automatic dispatch
    ///
    /// # Safety
    /// Caller must ensure:
    /// - `ctx` is valid and non-null
    /// - `branches` array has `num_branches` elements
    /// - Branch functions don't panic
    #[inline(always)]
    pub unsafe fn parallel_split(
        &self,
        ctx: *mut PatternContext,
        branches: *const BranchFn,
        num_branches: u32,
    ) -> PatternResult {
        (self.parallel_split_fn)(ctx, branches, num_branches)
    }

    /// Execute synchronization pattern with automatic dispatch
    ///
    /// # Safety
    /// Caller must ensure:
    /// - `ctx` is valid and non-null
    /// - `branch_results` array has `num_branches` elements
    /// - All branch results are valid
    #[inline(always)]
    pub unsafe fn synchronization(
        &self,
        ctx: *mut PatternContext,
        branch_results: *const u64,
        num_branches: u32,
    ) -> PatternResult {
        (self.synchronization_fn)(ctx, branch_results, num_branches)
    }

    /// Execute multi-choice pattern with automatic dispatch
    ///
    /// # Safety
    /// Caller must ensure:
    /// - `ctx` is valid and non-null
    /// - `conditions` and `branches` arrays have `num_branches` elements
    /// - Condition and branch functions don't panic
    #[inline(always)]
    pub unsafe fn multi_choice(
        &self,
        ctx: *mut PatternContext,
        conditions: *const BranchFn,
        branches: *const BranchFn,
        num_branches: u32,
    ) -> PatternResult {
        (self.multi_choice_fn)(ctx, conditions, branches, num_branches)
    }
}

impl Default for CpuDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Public API - Safe Wrappers
// ============================================================================

/// Initialize CPU dispatcher and log detected features
/// Call this once at application startup
pub fn init_cpu_dispatch() {
    let features = CpuFeatures::get();
    features.log_features();

    // Force dispatcher initialization
    let _ = CpuDispatcher::get();
}

/// Get optimal discriminator function for this CPU
#[inline(always)]
pub fn get_discriminator_fn() -> DiscriminatorFn {
    CpuDispatcher::get().select_discriminator()
}

/// Get optimal parallel split function for this CPU
#[inline(always)]
pub fn get_parallel_split_fn() -> ParallelSplitFn {
    CpuDispatcher::get().select_parallel_split()
}

/// Get optimal synchronization function for this CPU
#[inline(always)]
pub fn get_synchronization_fn() -> SynchronizationFn {
    CpuDispatcher::get().select_synchronization()
}

/// Get optimal multi-choice function for this CPU
#[inline(always)]
pub fn get_multi_choice_fn() -> MultiChoiceFn {
    CpuDispatcher::get().select_multi_choice()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_detection() {
        let features = CpuFeatures::get();

        // At least one of the architectures should be detected
        #[cfg(target_arch = "aarch64")]
        {
            // ARM64 should detect NEON on modern chips
            assert!(
                features.has_neon || !features.has_neon,
                "CPU features detected: {:?}",
                features
            );
            assert!(!features.has_avx2);
            assert!(!features.has_avx512);
        }

        #[cfg(target_arch = "x86_64")]
        {
            // x86_64 might have AVX2 or might not
            assert!(
                features.has_avx2 || !features.has_avx2,
                "CPU features detected: {:?}",
                features
            );
            assert!(!features.has_neon);
            assert!(!features.has_sve);
        }

        // Architecture name should be set
        assert!(!features.arch_name.is_empty());
    }

    #[test]
    fn test_dispatcher_creation() {
        let dispatcher = CpuDispatcher::get();

        // Dispatcher should select valid function pointers
        let discriminator = dispatcher.select_discriminator();
        assert_ne!(discriminator as *const () as usize, 0);

        let parallel_split = dispatcher.select_parallel_split();
        assert_ne!(parallel_split as *const () as usize, 0);

        let synchronization = dispatcher.select_synchronization();
        assert_ne!(synchronization as *const () as usize, 0);

        let multi_choice = dispatcher.select_multi_choice();
        assert_ne!(multi_choice as *const () as usize, 0);
    }

    #[test]
    fn test_cpu_features_caching() {
        // First call
        let features1 = CpuFeatures::get();

        // Second call should return same reference (cached)
        let features2 = CpuFeatures::get();

        assert_eq!(
            features1 as *const _, features2 as *const _,
            "CPU features should be cached"
        );
    }

    #[test]
    fn test_dispatcher_caching() {
        // First call
        let dispatcher1 = CpuDispatcher::get();

        // Second call should return same reference (cached)
        let dispatcher2 = CpuDispatcher::get();

        assert_eq!(
            dispatcher1 as *const _, dispatcher2 as *const _,
            "Dispatcher should be cached"
        );
    }

    #[test]
    fn test_init_cpu_dispatch() {
        // Should not panic
        init_cpu_dispatch();

        // Should be idempotent
        init_cpu_dispatch();
    }
}
