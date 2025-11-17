// knhk-kernel: Platform-specific unsafe operations
// This module contains ONLY unavoidable unsafe code that has no safe alternative
// All unsafe code here is documented and justified

/// SAFETY JUSTIFICATION:
/// - RDTSC intrinsics: No safe alternative exists for cycle-accurate timing
/// - CPU affinity: No safe alternative exists for thread pinning
/// - SIMD intrinsics: No safe alternative exists for vectorized operations
///
/// These operations are marked #[allow(unsafe_code)] because they are
/// fundamental platform capabilities with no safe Rust equivalent.

#[allow(unsafe_code)]
pub mod unsafe_ops {
    /// Read Time Stamp Counter (x86-64 only)
    /// SAFETY: RDTSC is a read-only instruction with no side effects
    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    pub fn read_tsc() -> u64 {
        unsafe { std::arch::x86_64::_rdtsc() }
    }

    /// Read TSC with serialization
    /// SAFETY: CPUID and RDTSC are read-only instructions
    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    pub fn read_tsc_serialized() -> u64 {
        unsafe {
            std::arch::x86_64::__cpuid(0);
            std::arch::x86_64::_rdtsc()
        }
    }

    /// Read TSC with memory fence
    /// SAFETY: Memory fence and RDTSC are safe operations
    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    pub fn read_tsc_fenced() -> u64 {
        unsafe {
            std::arch::x86_64::_mm_mfence();
            let tsc = std::arch::x86_64::_rdtsc();
            std::arch::x86_64::_mm_mfence();
            tsc
        }
    }

    /// Pin thread to CPU core
    /// SAFETY: pthread_setaffinity_np is a C FFI call
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    #[inline]
    pub fn pin_to_cpu(cpu_id: usize) -> Result<(), String> {
        unsafe {
            let mut cpu_set: libc::cpu_set_t = std::mem::zeroed();
            libc::CPU_SET(cpu_id, &mut cpu_set);

            let result = libc::pthread_setaffinity_np(
                libc::pthread_self(),
                std::mem::size_of::<libc::cpu_set_t>(),
                &cpu_set,
            );

            if result == 0 {
                Ok(())
            } else {
                Err(format!("Failed to pin to CPU {}: errno {}", cpu_id, result))
            }
        }
    }

    /// Fallback for non-x86_64 platforms
    #[cfg(not(target_arch = "x86_64"))]
    #[inline(always)]
    pub fn read_tsc() -> u64 {
        std::time::Instant::now().elapsed().as_nanos() as u64
    }

    #[cfg(not(target_arch = "x86_64"))]
    #[inline(always)]
    pub fn read_tsc_serialized() -> u64 {
        read_tsc()
    }

    #[cfg(not(target_arch = "x86_64"))]
    #[inline(always)]
    pub fn read_tsc_fenced() -> u64 {
        read_tsc()
    }

    #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
    #[inline]
    pub fn pin_to_cpu(_cpu_id: usize) -> Result<(), String> {
        Err("CPU pinning not supported on this platform".to_string())
    }
}
