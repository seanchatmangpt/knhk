fn main() {
    // TRIZ Principle 2 (Taking Out): Make C compilation completely optional
    // knhk-patterns can work without C compiler - pure Rust implementation

    #[cfg(feature = "c-patterns")]
    {
        use std::path::Path;
        let c_file = "../knhk-hot/src/workflow_patterns.c";

        if Path::new(c_file).exists() {
            println!("cargo:warning=Building with C pattern optimization");
            match cc::Build::new()
                .file(c_file)
                .include("../knhk-hot/src")
                .opt_level(3)
                .flag("-march=native")
                .flag("-fno-strict-aliasing")
                .warnings(false)
                .try_compile("workflow_patterns")
            {
                Ok(_) => {
                    println!("cargo:warning=C pattern compilation successful");
                }
                Err(e) => {
                    println!("cargo:warning=C pattern compilation failed: {}", e);
                    println!("cargo:warning=Using pure Rust patterns");
                }
            }
            println!("cargo:rerun-if-changed={}", c_file);
            println!("cargo:rerun-if-changed=../knhk-hot/src/workflow_patterns.h");
        }
    }

    #[cfg(not(feature = "c-patterns"))]
    {
        println!("cargo:warning=Pure Rust patterns (no C compiler needed)");
    }
}
