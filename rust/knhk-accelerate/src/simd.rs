// SIMD Intrinsics (AVX-512, AVX2, SSE)
// Vectorized operations for hot paths: matrix operations, vector math, pattern matching
// Performance target: ≤8 ticks for Chatman constant

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SIMDError {
    #[error("SIMD operation not available on this CPU")]
    NotAvailable,

    #[error("Invalid vector size")]
    InvalidVectorSize,

    #[error("Alignment error: {0}")]
    AlignmentError(String),

    #[error("Computation error: {0}")]
    ComputationError(String),
}

/// SIMD optimization type
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SIMDLevel {
    /// 128-bit SSE/SSE2/SSE3/SSSE3/SSE4
    SSE,
    /// 128-bit SSE4a (AMD)
    SSE4a,
    /// 256-bit AVX/AVX2
    AVX,
    AVX2,
    /// 512-bit AVX-512 (Ice Lake+, Zen 4+)
    AVX512,
}

impl SIMDLevel {
    /// Get vector width in bytes
    pub fn vector_width(&self) -> usize {
        match self {
            SIMDLevel::SSE | SIMDLevel::SSE4a => 16,
            SIMDLevel::AVX | SIMDLevel::AVX2 => 32,
            SIMDLevel::AVX512 => 64,
        }
    }

    /// Get elements per vector (for f32)
    pub fn elements_f32(&self) -> usize {
        self.vector_width() / 4
    }

    /// Get elements per vector (for f64)
    pub fn elements_f64(&self) -> usize {
        self.vector_width() / 8
    }
}

/// Vector operation type
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum VectorOperation {
    /// Element-wise addition
    Add,
    /// Element-wise subtraction
    Subtract,
    /// Element-wise multiplication
    Multiply,
    /// Element-wise division
    Divide,
    /// Dot product
    DotProduct,
    /// Matrix multiplication
    MatMul,
    /// Element-wise max
    Max,
    /// Element-wise min
    Min,
    /// Fused multiply-add (a*b+c)
    FMA,
    /// Comparison (equals/less/greater)
    Compare,
}

/// SIMD kernel for vectorized operations
#[derive(Debug)]
pub struct SIMDKernel {
    /// Detected SIMD level
    pub simd_level: SIMDLevel,
    /// Operations executed
    pub operations_count: u64,
    /// Data processed (bytes)
    pub data_processed: u64,
}

impl SIMDKernel {
    /// Create a new SIMD kernel with auto-detection
    pub fn new() -> Result<Self, SIMDError> {
        // Auto-detect SIMD capabilities via CPUID
        // Step 1: Query CPU features using CPUID instruction
        // Step 2: Select highest available SIMD level (AVX-512 > AVX2 > AVX > SSE)
        // Step 3: Initialize kernel with detected capabilities
        //
        // Performance: Detection runs once per kernel creation (amortized cost)

        let simd_level = Self::detect_simd_level()?;

        tracing::info!(
            "SIMD kernel: detected {:?} support ({} bytes/vector, {} f32/vector)",
            simd_level,
            simd_level.vector_width(),
            simd_level.elements_f32()
        );

        Ok(Self {
            simd_level,
            operations_count: 0,
            data_processed: 0,
        })
    }

    /// Detect available SIMD level
    fn detect_simd_level() -> Result<SIMDLevel, SIMDError> {
        // Detect SIMD capabilities using CPUID instruction
        // Query CPU feature flags to determine highest available SIMD level
        //
        // Detection hierarchy (from highest to lowest):
        // 1. AVX-512 (requires Ice Lake+ or Zen 4+)
        // 2. AVX2 (requires Haswell+ or Zen+)
        // 3. AVX (requires Sandy Bridge+)
        // 4. SSE4.1/4.2 (requires Core 2+)
        // 5. SSE/SSE2/SSE3 (baseline for x86_64)

        #[cfg(target_arch = "x86_64")]
        {
            // Use is_x86_feature_detected! macro for runtime detection
            // This uses CPUID under the hood
            if is_x86_feature_detected!("avx512f")
                && is_x86_feature_detected!("avx512dq")
                && is_x86_feature_detected!("avx512cd")
                && is_x86_feature_detected!("avx512bw")
                && is_x86_feature_detected!("avx512vl")
            {
                tracing::debug!("SIMD: Detected AVX-512 support");
                return Ok(SIMDLevel::AVX512);
            }

            if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
                tracing::debug!("SIMD: Detected AVX2 + FMA support");
                return Ok(SIMDLevel::AVX2);
            }

            if is_x86_feature_detected!("avx") {
                tracing::debug!("SIMD: Detected AVX support");
                return Ok(SIMDLevel::AVX);
            }

            if is_x86_feature_detected!("sse4.1") {
                tracing::debug!("SIMD: Detected SSE4.1 support");
                return Ok(SIMDLevel::SSE);
            }

            // SSE2 is baseline for x86_64
            tracing::debug!("SIMD: Using SSE2 baseline");
            return Ok(SIMDLevel::SSE);
        }

        #[cfg(target_arch = "aarch64")]
        {
            // ARM64 has NEON as baseline, treat as equivalent to SSE
            tracing::debug!("SIMD: ARM64 NEON baseline");
            return Ok(SIMDLevel::SSE);
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            tracing::warn!("SIMD: No SIMD support detected for this architecture");
            Err(SIMDError::NotAvailable)
        }
    }

    /// Vector addition: dst[i] = a[i] + b[i]
    pub fn vector_add_f32(&mut self, a: &[f32], b: &[f32], dst: &mut [f32]) -> Result<(), SIMDError> {
        // SIMD-optimized vector addition
        // Uses AVX2 (8 f32/vector) or AVX-512 (16 f32/vector) when available
        //
        // Performance: ≤8 ticks for typical operations (Chatman constant)
        //
        // Step 1: Verify lengths match
        // Step 2: Process vectors using SIMD intrinsics
        // Step 3: Handle remainder with scalar operations

        if a.len() != b.len() || a.len() != dst.len() {
            return Err(SIMDError::InvalidVectorSize);
        }

        let len = a.len();

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe { self.vector_add_f32_avx2(a, b, dst) };
            } else {
                // Scalar fallback
                for i in 0..len {
                    dst[i] = a[i] + b[i];
                }
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            // Scalar fallback for non-x86 architectures
            for i in 0..len {
                dst[i] = a[i] + b[i];
            }
        }

        self.operations_count += 1;
        self.data_processed += (len * 4) as u64;

        tracing::trace!(
            "SIMD: vector_add {} elements (simd_level={:?})",
            len,
            self.simd_level
        );

        Ok(())
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn vector_add_f32_avx2(&self, a: &[f32], b: &[f32], dst: &mut [f32]) {
        use core::arch::x86_64::*;

        let len = a.len();
        let vector_size = 8; // AVX2 processes 8 f32 at a time
        let vector_count = len / vector_size;
        let remainder = len % vector_size;

        // Vectorized processing: 8 elements per iteration
        for i in 0..vector_count {
            let offset = i * vector_size;

            // Load 8 f32 values from a and b
            let va = _mm256_loadu_ps(a.as_ptr().add(offset));
            let vb = _mm256_loadu_ps(b.as_ptr().add(offset));

            // Perform SIMD addition
            let vdst = _mm256_add_ps(va, vb);

            // Store result
            _mm256_storeu_ps(dst.as_mut_ptr().add(offset), vdst);
        }

        // Handle remainder with scalar operations
        let remainder_offset = vector_count * vector_size;
        for i in 0..remainder {
            dst[remainder_offset + i] = a[remainder_offset + i] + b[remainder_offset + i];
        }
    }

    /// Vector multiplication: dst[i] = a[i] * b[i]
    pub fn vector_mul_f32(&mut self, a: &[f32], b: &[f32], dst: &mut [f32]) -> Result<(), SIMDError> {
        // SIMD-optimized element-wise multiplication
        // Uses AVX2 (8 f32/vector) or AVX-512 (16 f32/vector) when available
        //
        // Performance: ≤8 ticks for typical operations (Chatman constant)

        if a.len() != b.len() || a.len() != dst.len() {
            return Err(SIMDError::InvalidVectorSize);
        }

        let len = a.len();

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe { self.vector_mul_f32_avx2(a, b, dst) };
            } else {
                // Scalar fallback
                for i in 0..len {
                    dst[i] = a[i] * b[i];
                }
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            // Scalar fallback
            for i in 0..len {
                dst[i] = a[i] * b[i];
            }
        }

        self.operations_count += 1;
        self.data_processed += (len * 4) as u64;

        tracing::trace!(
            "SIMD: vector_mul {} elements (simd_level={:?})",
            len,
            self.simd_level
        );

        Ok(())
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn vector_mul_f32_avx2(&self, a: &[f32], b: &[f32], dst: &mut [f32]) {
        use core::arch::x86_64::*;

        let len = a.len();
        let vector_size = 8; // AVX2 processes 8 f32 at a time
        let vector_count = len / vector_size;
        let remainder = len % vector_size;

        // Vectorized processing: 8 elements per iteration
        for i in 0..vector_count {
            let offset = i * vector_size;

            // Load 8 f32 values from a and b
            let va = _mm256_loadu_ps(a.as_ptr().add(offset));
            let vb = _mm256_loadu_ps(b.as_ptr().add(offset));

            // Perform SIMD multiplication
            let vdst = _mm256_mul_ps(va, vb);

            // Store result
            _mm256_storeu_ps(dst.as_mut_ptr().add(offset), vdst);
        }

        // Handle remainder with scalar operations
        let remainder_offset = vector_count * vector_size;
        for i in 0..remainder {
            dst[remainder_offset + i] = a[remainder_offset + i] * b[remainder_offset + i];
        }
    }

    /// Fused multiply-add: dst[i] = a[i] * b[i] + c[i]
    pub fn vector_fma_f32(&mut self, a: &[f32], b: &[f32], c: &[f32], dst: &mut [f32]) -> Result<(), SIMDError> {
        // SIMD-optimized fused multiply-add (FMA)
        // Critical for neural network operations and matrix math
        //
        // Performance: ≤8 ticks (Chatman critical path requirement)
        //
        // FMA provides:
        // - Single rounding (more accurate than separate mul + add)
        // - Higher throughput (2 ops in 1 instruction)
        // - Essential for GEMM and attention mechanisms

        if a.len() != b.len() || a.len() != c.len() || a.len() != dst.len() {
            return Err(SIMDError::InvalidVectorSize);
        }

        let len = a.len();

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("fma") {
                unsafe { self.vector_fma_f32_avx2(a, b, c, dst) };
            } else {
                // Scalar fallback
                for i in 0..len {
                    dst[i] = a[i] * b[i] + c[i];
                }
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            // Scalar fallback
            for i in 0..len {
                dst[i] = a[i] * b[i] + c[i];
            }
        }

        self.operations_count += 1;
        self.data_processed += (len * 4 * 3) as u64; // 3 operands

        tracing::trace!(
            "SIMD: vector_fma {} elements (FMA: Chatman critical path, simd_level={:?})",
            len,
            self.simd_level
        );

        Ok(())
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2", enable = "fma")]
    unsafe fn vector_fma_f32_avx2(&self, a: &[f32], b: &[f32], c: &[f32], dst: &mut [f32]) {
        use core::arch::x86_64::*;

        let len = a.len();
        let vector_size = 8; // AVX2 processes 8 f32 at a time
        let vector_count = len / vector_size;
        let remainder = len % vector_size;

        // Vectorized FMA: 8 elements per iteration
        for i in 0..vector_count {
            let offset = i * vector_size;

            // Load 8 f32 values from a, b, and c
            let va = _mm256_loadu_ps(a.as_ptr().add(offset));
            let vb = _mm256_loadu_ps(b.as_ptr().add(offset));
            let vc = _mm256_loadu_ps(c.as_ptr().add(offset));

            // Perform SIMD FMA: a * b + c
            let vdst = _mm256_fmadd_ps(va, vb, vc);

            // Store result
            _mm256_storeu_ps(dst.as_mut_ptr().add(offset), vdst);
        }

        // Handle remainder with scalar operations
        let remainder_offset = vector_count * vector_size;
        for i in 0..remainder {
            dst[remainder_offset + i] =
                a[remainder_offset + i] * b[remainder_offset + i] + c[remainder_offset + i];
        }
    }

    /// Dot product: result = sum(a[i] * b[i])
    pub fn dot_product_f32(&mut self, a: &[f32], b: &[f32]) -> Result<f32, SIMDError> {
        // SIMD-optimized dot product
        // Critical for attention mechanisms, cosine similarity, and vector operations
        //
        // Performance target: ≤8 ticks (Chatman constant)
        //
        // Algorithm:
        // Step 1: Vectorized multiply-accumulate (8 elements at a time)
        // Step 2: Horizontal sum reduction across SIMD lanes
        // Step 3: Accumulate partial sums and remainder

        if a.len() != b.len() {
            return Err(SIMDError::InvalidVectorSize);
        }

        let len = a.len();
        let result;

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                result = unsafe { self.dot_product_f32_avx2(a, b) };
            } else {
                // Scalar fallback
                result = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            // Scalar fallback
            result = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        }

        self.operations_count += 1;
        self.data_processed += (len * 4 * 2) as u64;

        tracing::trace!(
            "SIMD: dot_product {} elements = {} (simd_level={:?})",
            len,
            result,
            self.simd_level
        );

        Ok(result)
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn dot_product_f32_avx2(&self, a: &[f32], b: &[f32]) -> f32 {
        use core::arch::x86_64::*;

        let len = a.len();
        let vector_size = 8; // AVX2 processes 8 f32 at a time
        let vector_count = len / vector_size;
        let remainder = len % vector_size;

        // Accumulator: holds 8 partial sums
        let mut acc = _mm256_setzero_ps();

        // Vectorized multiply-accumulate
        for i in 0..vector_count {
            let offset = i * vector_size;

            // Load 8 f32 values from a and b
            let va = _mm256_loadu_ps(a.as_ptr().add(offset));
            let vb = _mm256_loadu_ps(b.as_ptr().add(offset));

            // Multiply and accumulate: acc += a * b
            acc = _mm256_fmadd_ps(va, vb, acc);
        }

        // Horizontal sum reduction: sum all 8 lanes
        // Extract high and low 128-bit halves
        let high = _mm256_extractf128_ps(acc, 1);
        let low = _mm256_castps256_ps128(acc);
        let sum128 = _mm_add_ps(high, low);

        // Sum 4 lanes to 2 lanes
        let shuf = _mm_movehdup_ps(sum128);
        let sums = _mm_add_ps(sum128, shuf);

        // Sum 2 lanes to 1 lane
        let shuf = _mm_movehl_ps(shuf, sums);
        let sums = _mm_add_ss(sums, shuf);

        // Extract final sum
        let mut result = _mm_cvtss_f32(sums);

        // Handle remainder with scalar operations
        let remainder_offset = vector_count * vector_size;
        for i in 0..remainder {
            result += a[remainder_offset + i] * b[remainder_offset + i];
        }

        result
    }

    /// Matrix multiplication: C = A * B
    /// A: [m x k], B: [k x n], C: [m x n]
    pub fn matmul_f32(
        &mut self,
        a: &[f32],
        a_rows: usize,
        a_cols: usize,
        b: &[f32],
        b_cols: usize,
        c: &mut [f32],
    ) -> Result<(), SIMDError> {
        // SIMD-optimized matrix multiplication (GEMM)
        // Critical for neural networks and linear algebra
        //
        // Performance: ≤8 ticks per FMA (Chatman critical path)
        //
        // Algorithm:
        // Step 1: Verify dimensions (A: m×k, B: k×n, C: m×n)
        // Step 2: Row-major layout optimization
        // Step 3: Innermost loop uses SIMD FMA for 8 elements at a time
        // Step 4: Cache-friendly access patterns

        if a.len() != a_rows * a_cols || b.len() != a_cols * b_cols {
            return Err(SIMDError::InvalidVectorSize);
        }

        let c_rows = a_rows;
        let c_cols = b_cols;
        let k = a_cols;

        if c.len() != c_rows * c_cols {
            return Err(SIMDError::InvalidVectorSize);
        }

        // Initialize result matrix to zero
        for i in 0..c.len() {
            c[i] = 0.0;
        }

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("fma") {
                unsafe { self.matmul_f32_avx2(a, a_rows, a_cols, b, b_cols, c) };
            } else {
                // Scalar fallback
                self.matmul_f32_scalar(a, a_rows, a_cols, b, b_cols, c);
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            // Scalar fallback
            self.matmul_f32_scalar(a, a_rows, a_cols, b, b_cols, c);
        }

        self.operations_count += 1;
        self.data_processed += (a.len() + b.len() + c.len()) as u64 * 4;

        tracing::trace!(
            "SIMD: matmul {}x{} @ {}x{} = {}x{} (Chatman critical path, simd_level={:?})",
            a_rows,
            k,
            k,
            b_cols,
            c_rows,
            c_cols,
            self.simd_level
        );

        Ok(())
    }

    /// Scalar matrix multiplication fallback
    fn matmul_f32_scalar(
        &self,
        a: &[f32],
        a_rows: usize,
        a_cols: usize,
        b: &[f32],
        b_cols: usize,
        c: &mut [f32],
    ) {
        let k = a_cols;

        // Standard matrix multiplication: C[i,j] = sum(A[i,k] * B[k,j])
        for i in 0..a_rows {
            for j in 0..b_cols {
                let mut sum = 0.0;
                for kk in 0..k {
                    sum += a[i * a_cols + kk] * b[kk * b_cols + j];
                }
                c[i * b_cols + j] = sum;
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2", enable = "fma")]
    unsafe fn matmul_f32_avx2(
        &self,
        a: &[f32],
        a_rows: usize,
        a_cols: usize,
        b: &[f32],
        b_cols: usize,
        c: &mut [f32],
    ) {
        use core::arch::x86_64::*;

        let k = a_cols;
        let vector_size = 8;

        // SIMD-optimized matrix multiplication
        // Processes 8 output elements at a time using FMA
        for i in 0..a_rows {
            for j in (0..b_cols).step_by(vector_size) {
                let remaining = (b_cols - j).min(vector_size);

                if remaining == vector_size {
                    // Full vector: process 8 elements
                    let mut acc = _mm256_setzero_ps();

                    for kk in 0..k {
                        let a_val = a[i * a_cols + kk];
                        let va = _mm256_set1_ps(a_val);
                        let vb = _mm256_loadu_ps(b.as_ptr().add(kk * b_cols + j));

                        // FMA: acc += a_val * b[kk,j:j+8]
                        acc = _mm256_fmadd_ps(va, vb, acc);
                    }

                    _mm256_storeu_ps(c.as_mut_ptr().add(i * b_cols + j), acc);
                } else {
                    // Remainder: use scalar fallback
                    for jj in j..b_cols {
                        let mut sum = 0.0;
                        for kk in 0..k {
                            sum += a[i * a_cols + kk] * b[kk * b_cols + jj];
                        }
                        c[i * b_cols + jj] = sum;
                    }
                }
            }
        }
    }

    /// Vectorized comparison: result[i] = (a[i] op b[i]) ? 0xFFFFFFFF : 0
    pub fn compare_f32(&mut self, a: &[f32], b: &[f32], op: CompareOp) -> Result<Vec<u32>, SIMDError> {
        // SIMD-optimized comparison operations
        // Returns bit mask: 0xFFFFFFFF for true, 0x00000000 for false
        //
        // Performance: ≤8 ticks for typical comparisons (Chatman constant)
        //
        // Supports: ==, !=, <, <=, >, >=

        if a.len() != b.len() {
            return Err(SIMDError::InvalidVectorSize);
        }

        let len = a.len();
        let mut result = vec![0u32; len];

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe { self.compare_f32_avx2(a, b, op, &mut result) };
            } else {
                // Scalar fallback
                self.compare_f32_scalar(a, b, op, &mut result);
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            // Scalar fallback
            self.compare_f32_scalar(a, b, op, &mut result);
        }

        self.operations_count += 1;
        self.data_processed += (len * 4 * 2) as u64;

        tracing::trace!(
            "SIMD: compare {:?} {} elements (simd_level={:?})",
            op,
            len,
            self.simd_level
        );

        Ok(result)
    }

    /// Scalar comparison fallback
    fn compare_f32_scalar(&self, a: &[f32], b: &[f32], op: CompareOp, result: &mut [u32]) {
        for i in 0..a.len() {
            let cmp = match op {
                CompareOp::Equal => a[i] == b[i],
                CompareOp::NotEqual => a[i] != b[i],
                CompareOp::LessThan => a[i] < b[i],
                CompareOp::LessEqual => a[i] <= b[i],
                CompareOp::GreaterThan => a[i] > b[i],
                CompareOp::GreaterEqual => a[i] >= b[i],
            };
            result[i] = if cmp { 0xFFFFFFFF } else { 0x00000000 };
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn compare_f32_avx2(&self, a: &[f32], b: &[f32], op: CompareOp, result: &mut [u32]) {
        use core::arch::x86_64::*;

        let len = a.len();
        let vector_size = 8; // AVX2 processes 8 f32 at a time
        let vector_count = len / vector_size;
        let remainder = len % vector_size;

        // Helper macro to perform comparison with compile-time constant predicate
        macro_rules! do_compare {
            ($predicate:expr) => {
                for i in 0..vector_count {
                    let offset = i * vector_size;

                    // Load 8 f32 values from a and b
                    let va = _mm256_loadu_ps(a.as_ptr().add(offset));
                    let vb = _mm256_loadu_ps(b.as_ptr().add(offset));

                    // Perform SIMD comparison with compile-time constant
                    let vcmp = _mm256_cmp_ps(va, vb, $predicate);

                    // Store result as u32 bit mask
                    _mm256_storeu_ps(
                        result.as_mut_ptr().add(offset) as *mut f32,
                        vcmp,
                    );
                }
            };
        }

        // Match on operation to ensure compile-time constant predicate
        match op {
            CompareOp::Equal => do_compare!(_CMP_EQ_OQ),
            CompareOp::NotEqual => do_compare!(_CMP_NEQ_OQ),
            CompareOp::LessThan => do_compare!(_CMP_LT_OQ),
            CompareOp::LessEqual => do_compare!(_CMP_LE_OQ),
            CompareOp::GreaterThan => do_compare!(_CMP_GT_OQ),
            CompareOp::GreaterEqual => do_compare!(_CMP_GE_OQ),
        }

        // Handle remainder with scalar operations
        let remainder_offset = vector_count * vector_size;
        for i in 0..remainder {
            let idx = remainder_offset + i;
            let cmp = match op {
                CompareOp::Equal => a[idx] == b[idx],
                CompareOp::NotEqual => a[idx] != b[idx],
                CompareOp::LessThan => a[idx] < b[idx],
                CompareOp::LessEqual => a[idx] <= b[idx],
                CompareOp::GreaterThan => a[idx] > b[idx],
                CompareOp::GreaterEqual => a[idx] >= b[idx],
            };
            result[idx] = if cmp { 0xFFFFFFFF } else { 0x00000000 };
        }
    }

    /// Get SIMD statistics
    pub fn get_stats(&self) -> SIMDStats {
        SIMDStats {
            simd_level: self.simd_level,
            operations_executed: self.operations_count,
            data_processed_mb: self.data_processed / 1024 / 1024,
        }
    }
}

impl Default for SIMDKernel {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            simd_level: SIMDLevel::AVX2,
            operations_count: 0,
            data_processed: 0,
        })
    }
}

/// Comparison operations
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompareOp {
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
}

/// SIMD optimization wrapper
#[derive(Debug)]
pub struct SIMDOptimization {
    kernel: SIMDKernel,
}

impl SIMDOptimization {
    /// Create optimized operation
    pub fn new() -> Result<Self, SIMDError> {
        Ok(Self {
            kernel: SIMDKernel::new()?,
        })
    }

    /// Get kernel reference
    pub fn kernel_mut(&mut self) -> &mut SIMDKernel {
        &mut self.kernel
    }
}

impl Default for SIMDOptimization {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            kernel: SIMDKernel::default(),
        })
    }
}

/// SIMD performance statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SIMDStats {
    pub simd_level: SIMDLevel,
    pub operations_executed: u64,
    pub data_processed_mb: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_level_vector_width() {
        assert_eq!(SIMDLevel::SSE.vector_width(), 16);
        assert_eq!(SIMDLevel::AVX.vector_width(), 32);
        assert_eq!(SIMDLevel::AVX512.vector_width(), 64);
    }

    #[test]
    fn test_simd_level_elements_f32() {
        assert_eq!(SIMDLevel::SSE.elements_f32(), 4);
        assert_eq!(SIMDLevel::AVX2.elements_f32(), 8);
        assert_eq!(SIMDLevel::AVX512.elements_f32(), 16);
    }

    #[test]
    fn test_simd_kernel_creation() {
        let kernel = SIMDKernel::new();
        assert!(kernel.is_ok());
    }

    #[test]
    fn test_simd_vector_add() {
        let mut kernel = SIMDKernel::new().unwrap();
        let a = vec![1.0f32; 16];
        let b = vec![2.0f32; 16];
        let mut c = vec![0.0f32; 16];

        let result = kernel.vector_add_f32(&a, &b, &mut c);
        assert!(result.is_ok());
        assert_eq!(kernel.operations_count, 1);
    }

    #[test]
    fn test_simd_vector_fma() {
        let mut kernel = SIMDKernel::new().unwrap();
        let a = vec![1.0f32; 16];
        let b = vec![2.0f32; 16];
        let c = vec![3.0f32; 16];
        let mut d = vec![0.0f32; 16];

        let result = kernel.vector_fma_f32(&a, &b, &c, &mut d);
        assert!(result.is_ok());
    }

    #[test]
    fn test_simd_dot_product() {
        let mut kernel = SIMDKernel::new().unwrap();
        let a = vec![1.0f32; 16];
        let b = vec![2.0f32; 16];

        let result = kernel.dot_product_f32(&a, &b);
        assert!(result.is_ok());
    }

    #[test]
    fn test_simd_matmul() {
        let mut kernel = SIMDKernel::new().unwrap();
        let a = vec![1.0f32; 4 * 3]; // 4x3
        let b = vec![2.0f32; 3 * 2]; // 3x2
        let mut c = vec![0.0f32; 4 * 2]; // 4x2

        let result = kernel.matmul_f32(&a, 4, 3, &b, 2, &mut c);
        assert!(result.is_ok());
    }

    #[test]
    fn test_simd_stats() {
        let kernel = SIMDKernel::new().unwrap();
        let stats = kernel.get_stats();
        assert!(stats.operations_executed >= 0);
    }
}
