fn main() {
    use std::path::Path;

    // TRIZ Principle 2 (Taking Out): Make C compilation OPTIONAL
    // Only attempt C compilation if:
    // 1. Feature flag "c-optimization" is enabled AND
    // 2. C source files exist AND
    // 3. C compiler is available

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let c_files_exist = Path::new(&format!("{}/src/workflow_patterns.c", manifest_dir)).exists()
        && Path::new(&format!("{}/src/ring_buffer.c", manifest_dir)).exists();

    let c_optimization_enabled = cfg!(feature = "c-optimization");

    #[cfg(feature = "c-optimization")]
    {
        if c_files_exist {
            println!("cargo:warning=Building with C optimization (hot path)");

            // Attempt C compilation - if it fails, we'll use Rust fallback
            match cc::Build::new()
                .file("src/workflow_patterns.c")
                .file("src/ring_buffer.c")
                .file("src/simd_predicates.c")
                .opt_level(3)
                .flag("-march=native")
                .flag("-fno-strict-aliasing")
                .warnings(false)
                .try_compile("workflow_patterns")
            {
                Ok(_) => {
                    println!("cargo:warning=C compilation successful");

                    // Try to link to KNHK C library if it exists
                    let c_lib_dir = format!("{}/../../c", manifest_dir);
                    let lib_path = format!("{}/libknhk.a", c_lib_dir);

                    if Path::new(&lib_path).exists() {
                        println!("cargo:rustc-link-search=native={}", c_lib_dir);
                        println!("cargo:rustc-link-lib=static=knhk");
                        println!("cargo:warning=Linked to libknhk.a");
                    } else {
                        println!("cargo:warning=libknhk.a not found, using workflow_patterns.a only");
                    }

                    println!("cargo:rustc-cfg=feature=\"c_compiled\"");
                }
                Err(e) => {
                    println!("cargo:warning=C compilation failed: {}", e);
                    println!("cargo:warning=Falling back to pure Rust implementation");
                }
            }
        } else {
            println!("cargo:warning=C source files not found, using pure Rust implementation");
        }
    }

    #[cfg(not(feature = "c-optimization"))]
    {
        println!("cargo:warning=C optimization disabled (feature flag not set)");
        println!("cargo:warning=Using pure Rust implementation (JTBD accomplishable)");
    }

    // Rerun if files change
    if c_files_exist {
        println!("cargo:rerun-if-changed=src/workflow_patterns.c");
        println!("cargo:rerun-if-changed=src/workflow_patterns.h");
        println!("cargo:rerun-if-changed=src/ring_buffer.c");
        println!("cargo:rerun-if-changed=src/simd_predicates.c");
        println!("cargo:rerun-if-changed=src/simd_predicates.h");
    }

    let c_lib_dir = format!("{}/../../c", manifest_dir);
    let lib_path = format!("{}/libknhk.a", c_lib_dir);
    println!("cargo:rerun-if-changed={}", lib_path);
}
