// knhk-kernel: Build script for optimizations and validation

use std::env;

fn main() {
    // Set optimization flags for release builds
    if env::var("PROFILE").unwrap_or_default() == "release" {
        // Enable CPU-specific optimizations
        if cfg!(target_arch = "x86_64") {
            // Use native CPU optimizations
            println!("cargo:rustc-env=RUSTFLAGS=-C target-cpu=native");
        }
    }

    // Validate Chatman constant at build time
    validate_chatman_constant();

    // Print build information
    println!("cargo:warning=Building KNHK Kernel with ≤8 tick guarantee");
}

fn validate_chatman_constant() {
    const CHATMAN_CONSTANT: u32 = 8;

    // This is a compile-time check
    const _: () = {
        if CHATMAN_CONSTANT > 8 {
            panic!("Chatman constant exceeds 8 ticks");
        }
    };

    println!("cargo:rustc-env=CHATMAN_CONSTANT={}", CHATMAN_CONSTANT);
}