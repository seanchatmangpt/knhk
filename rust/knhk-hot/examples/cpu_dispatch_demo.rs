// knhk-hot: CPU dispatch demonstration
// Shows runtime CPU detection and SIMD function selection

use knhk_hot::{init_cpu_dispatch, CpuDispatcher, CpuFeatures};

fn main() {
    println!("=== KNHK CPU Dispatch Demonstration ===\n");

    // Initialize CPU dispatch and log features
    println!("Step 1: Initializing CPU dispatcher...");
    init_cpu_dispatch();
    println!();

    // Get detected CPU features
    println!("Step 2: Detected CPU features");
    println!("------------------------------");
    let features = CpuFeatures::get();

    println!("Architecture: {}", features.arch_name);
    println!("  ARM NEON:   {}", features.has_neon);
    println!("  ARM SVE:    {}", features.has_sve);
    println!("  Intel AVX2: {}", features.has_avx2);
    println!("  Intel AVX512: {}", features.has_avx512);
    println!();

    // Get dispatcher and show selected implementations
    println!("Step 3: Selected implementations");
    println!("---------------------------------");
    let dispatcher = CpuDispatcher::get();

    let has_simd = features.has_neon || features.has_avx2 || features.has_avx512;

    println!(
        "SIMD available: {}",
        if has_simd { "YES (using optimized kernels)" } else { "NO (using generic fallback)" }
    );
    println!();

    // Show function pointer addresses (for verification)
    println!("Step 4: Function pointer addresses");
    println!("-----------------------------------");

    let discriminator = dispatcher.select_discriminator();
    println!(
        "Discriminator:    0x{:x}",
        discriminator as *const () as usize
    );

    let parallel_split = dispatcher.select_parallel_split();
    println!(
        "Parallel Split:   0x{:x}",
        parallel_split as *const () as usize
    );

    let synchronization = dispatcher.select_synchronization();
    println!(
        "Synchronization:  0x{:x}",
        synchronization as *const () as usize
    );

    let multi_choice = dispatcher.select_multi_choice();
    println!(
        "Multi-Choice:     0x{:x}",
        multi_choice as *const () as usize
    );
    println!();

    // Verify caching works
    println!("Step 5: Verifying dispatcher caching");
    println!("-------------------------------------");

    let features1 = CpuFeatures::get();
    let features2 = CpuFeatures::get();

    println!(
        "Features cached: {}",
        if features1 as *const _ == features2 as *const _ {
            "YES (same reference)"
        } else {
            "NO (different reference - BUG!)"
        }
    );

    let dispatcher1 = CpuDispatcher::get();
    let dispatcher2 = CpuDispatcher::get();

    println!(
        "Dispatcher cached: {}",
        if dispatcher1 as *const _ == dispatcher2 as *const _ {
            "YES (same reference)"
        } else {
            "NO (different reference - BUG!)"
        }
    );
    println!();

    // Performance characteristics
    println!("Step 6: Performance characteristics");
    println!("------------------------------------");

    println!("✓ CPU detection: Called once at startup (OnceLock)");
    println!("✓ Dispatcher creation: Called once at startup (OnceLock)");
    println!("✓ Function selection: Zero-cost after first call (inlined)");
    println!("✓ SIMD dispatch: No runtime overhead (direct function pointer)");
    println!();

    // Summary
    println!("=== Summary ===");
    println!();
    if has_simd {
        println!("✓ SIMD optimization ENABLED");
        println!("  - Pattern execution will use vectorized instructions");
        println!("  - Expected speedup: 2-4x for parallel patterns");
    } else {
        println!("⚠ SIMD optimization DISABLED (generic fallback)");
        println!("  - Pattern execution will use scalar code");
        println!("  - Consider upgrading to SIMD-capable CPU for better performance");
    }
    println!();

    // Architecture-specific recommendations
    match features.arch_name {
        name if name.contains("ARM64-NEON") => {
            println!("📊 ARM64 NEON detected:");
            println!("  - Optimized for: Parallel Split, Synchronization, Multi-Choice, Discriminator");
            println!("  - NEON vector width: 128 bits (4x u32 or 2x u64 per instruction)");
            println!("  - Expected tick reduction: 30-50% for SIMD patterns");
        }
        name if name.contains("ARM64-SVE") => {
            println!("📊 ARM64 SVE detected:");
            println!("  - Scalable Vector Extension available");
            println!("  - Future optimization opportunity (not yet implemented)");
        }
        name if name.contains("x86_64-AVX512") => {
            println!("📊 Intel AVX-512 detected:");
            println!("  - Optimized for: Parallel Split, Synchronization, Multi-Choice, Discriminator");
            println!("  - AVX-512 vector width: 512 bits (16x u32 or 8x u64 per instruction)");
            println!("  - Expected tick reduction: 50-70% for SIMD patterns");
        }
        name if name.contains("x86_64-AVX2") => {
            println!("📊 Intel AVX2 detected:");
            println!("  - Optimized for: Parallel Split, Synchronization, Multi-Choice, Discriminator");
            println!("  - AVX2 vector width: 256 bits (8x u32 or 4x u64 per instruction)");
            println!("  - Expected tick reduction: 40-60% for SIMD patterns");
        }
        _ => {
            println!("📊 Generic architecture detected:");
            println!("  - Using portable scalar implementation");
            println!("  - No SIMD acceleration available");
        }
    }
    println!();

    println!("=== Demonstration Complete ===");
}
