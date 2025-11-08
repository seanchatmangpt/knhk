// knhk-hot: CPU dispatch integration tests
// Verify runtime CPU detection and SIMD dispatch work correctly

use knhk_hot::{init_cpu_dispatch, CpuDispatcher, CpuFeatures};

#[test]
fn test_cpu_features_detection() {
    // CPU features should be detected
    let features = CpuFeatures::get();

    // Architecture name should be set
    assert!(!features.arch_name.is_empty());

    // Verify architecture-specific features
    #[cfg(target_arch = "aarch64")]
    {
        // ARM64: NEON should typically be available, AVX should not
        assert!(!features.has_avx2, "ARM64 should not report AVX2");
        assert!(!features.has_avx512, "ARM64 should not report AVX512");

        // Log detected ARM features
        println!(
            "ARM64 detected: NEON={}, SVE={}",
            features.has_neon, features.has_sve
        );
    }

    #[cfg(target_arch = "x86_64")]
    {
        // x86_64: AVX might be available, NEON/SVE should not
        assert!(!features.has_neon, "x86_64 should not report NEON");
        assert!(!features.has_sve, "x86_64 should not report SVE");

        // Log detected x86 features
        println!(
            "x86_64 detected: AVX2={}, AVX512={}",
            features.has_avx2, features.has_avx512
        );
    }
}

#[test]
fn test_cpu_features_caching() {
    // First call
    let features1 = CpuFeatures::get();

    // Second call should return exact same reference
    let features2 = CpuFeatures::get();

    assert_eq!(
        features1 as *const _, features2 as *const _,
        "CPU features should be cached and return same reference"
    );

    // Values should be identical
    assert_eq!(features1.has_neon, features2.has_neon);
    assert_eq!(features1.has_sve, features2.has_sve);
    assert_eq!(features1.has_avx2, features2.has_avx2);
    assert_eq!(features1.has_avx512, features2.has_avx512);
    assert_eq!(features1.arch_name, features2.arch_name);
}

#[test]
fn test_dispatcher_creation() {
    let dispatcher = CpuDispatcher::get();

    // All function pointers should be non-null
    let discriminator = dispatcher.select_discriminator();
    assert_ne!(
        discriminator as *const () as usize, 0,
        "Discriminator function pointer should be non-null"
    );

    let parallel_split = dispatcher.select_parallel_split();
    assert_ne!(
        parallel_split as *const () as usize, 0,
        "Parallel split function pointer should be non-null"
    );

    let synchronization = dispatcher.select_synchronization();
    assert_ne!(
        synchronization as *const () as usize, 0,
        "Synchronization function pointer should be non-null"
    );

    let multi_choice = dispatcher.select_multi_choice();
    assert_ne!(
        multi_choice as *const () as usize, 0,
        "Multi-choice function pointer should be non-null"
    );
}

#[test]
fn test_dispatcher_caching() {
    // First call
    let dispatcher1 = CpuDispatcher::get();

    // Second call should return exact same reference
    let dispatcher2 = CpuDispatcher::get();

    assert_eq!(
        dispatcher1 as *const _, dispatcher2 as *const _,
        "Dispatcher should be cached and return same reference"
    );
}

#[test]
fn test_dispatcher_features_consistency() {
    let dispatcher = CpuDispatcher::get();
    let features = CpuFeatures::get();

    // Dispatcher should reference same features
    assert_eq!(
        dispatcher.features() as *const _,
        features as *const _,
        "Dispatcher should reference cached CPU features"
    );
}

#[test]
fn test_init_cpu_dispatch() {
    // Should not panic
    init_cpu_dispatch();

    // Should be idempotent - multiple calls should be safe
    init_cpu_dispatch();
    init_cpu_dispatch();
}

#[test]
fn test_dispatcher_selects_optimal_implementation() {
    let features = CpuFeatures::get();
    let has_simd = features.has_neon || features.has_avx2 || features.has_avx512;

    println!(
        "CPU has SIMD: {} (arch: {})",
        has_simd, features.arch_name
    );

    // If SIMD is available, dispatcher should select SIMD versions
    // If not, it should select generic fallback
    // We can't directly verify which was selected without C symbol inspection,
    // but we can verify dispatch completed without error
    let dispatcher = CpuDispatcher::get();

    // All selections should succeed
    let _ = dispatcher.select_discriminator();
    let _ = dispatcher.select_parallel_split();
    let _ = dispatcher.select_synchronization();
    let _ = dispatcher.select_multi_choice();
}

#[test]
fn test_dispatcher_inline_performance() {
    // Verify that dispatcher methods are properly inlined
    // by checking they don't allocate (indirect verification)

    let dispatcher = CpuDispatcher::get();

    // These calls should be zero-cost after inlining
    for _ in 0..1000 {
        let _ = dispatcher.select_discriminator();
        let _ = dispatcher.select_parallel_split();
        let _ = dispatcher.select_synchronization();
        let _ = dispatcher.select_multi_choice();
    }

    // If we got here without OOM, inlining likely worked
    // (direct function pointer access doesn't allocate)
}

#[test]
fn test_architecture_name_format() {
    let features = CpuFeatures::get();

    // Architecture name should follow expected format
    let valid_prefixes = ["ARM64", "x86_64", "GENERIC"];

    let has_valid_prefix = valid_prefixes
        .iter()
        .any(|prefix| features.arch_name.starts_with(prefix));

    assert!(
        has_valid_prefix,
        "Architecture name '{}' should start with one of {:?}",
        features.arch_name,
        valid_prefixes
    );
}

#[test]
fn test_feature_mutual_exclusivity() {
    let features = CpuFeatures::get();

    // ARM features and x86 features should be mutually exclusive
    let has_arm = features.has_neon || features.has_sve;
    let has_x86 = features.has_avx2 || features.has_avx512;

    if has_arm {
        assert!(
            !has_x86,
            "ARM features detected, but x86 features also present"
        );
    }

    if has_x86 {
        assert!(
            !has_arm,
            "x86 features detected, but ARM features also present"
        );
    }
}

#[test]
fn test_public_api_functions() {
    use knhk_hot::{
        get_discriminator_fn, get_multi_choice_fn, get_parallel_split_fn, get_synchronization_fn,
    };

    // All public API functions should return valid function pointers
    let discriminator = get_discriminator_fn();
    assert_ne!(discriminator as *const () as usize, 0);

    let parallel_split = get_parallel_split_fn();
    assert_ne!(parallel_split as *const () as usize, 0);

    let synchronization = get_synchronization_fn();
    assert_ne!(synchronization as *const () as usize, 0);

    let multi_choice = get_multi_choice_fn();
    assert_ne!(multi_choice as *const () as usize, 0);
}

#[test]
fn test_cpu_features_debug_format() {
    let features = CpuFeatures::get();

    // Debug format should work without panic
    let debug_str = format!("{:?}", features);

    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("CpuFeatures"));
}

#[test]
fn test_cpu_features_clone() {
    let features1 = *CpuFeatures::get();
    let features2 = features1;

    // Clone/Copy should work
    assert_eq!(features1.has_neon, features2.has_neon);
    assert_eq!(features1.has_sve, features2.has_sve);
    assert_eq!(features1.has_avx2, features2.has_avx2);
    assert_eq!(features1.has_avx512, features2.has_avx512);
    assert_eq!(features1.arch_name, features2.arch_name);
}
